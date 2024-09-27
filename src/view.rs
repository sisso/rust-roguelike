pub mod camera;
pub mod cockpit_window;
pub mod game_window;
pub mod lose_window;
pub mod main_menu_window;
pub mod window;

use crate::actions::{Action, EntityActions};
use crate::area::{Area, Tile};
use crate::cfg;
use crate::commons::grid::BaseGrid;
use crate::commons::recti::RectI;
use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::health::{Health, Hp};
use crate::models::{ObjectsKind, Position};
use crate::state::State;
use crate::utils::find_objects_at;
use crate::view::camera::Camera;
use crate::visibility::{Visibility, VisibilityMemory};
use crate::P2;
use hecs::{Entity, World};
use rltk::{BTerm, Rect, Rltk, TextAlign, RGB};
use std::collections::HashSet;

pub type Color = (u8, u8, u8);

#[derive(Clone, Debug)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub priority: i32,
}

pub fn get_window_rect() -> RectI {
    RectI::new(0, 0, cfg::SCREEN_W, cfg::SCREEN_H)
}

pub fn draw_rect(ctx: &mut Rltk, rect: &RectI, fg: Color, bg: Color) {
    ctx.draw_box(
        rect.get_x(),
        rect.get_y(),
        rect.get_width(),
        rect.get_height(),
        fg,
        bg,
    );
}

fn draw_map_and_objects(state: &mut State, ctx: &mut Rltk, rect: RectI) {
    let avatar_id = state.player.get_avatar_id();

    let mut query = state
        .ecs
        .query_one::<(&Visibility, &Position, &VisibilityMemory)>(avatar_id)
        .expect("player avatar not found");
    let (visibility, pos, memory) = query.get().expect("player not found");

    let camera = Camera::from_center(pos.clone(), rect);

    // draw
    let map = GridRef::find_area(&state.ecs, pos.grid_id).expect("area not found");
    draw_map(
        &camera,
        &visibility.visible_tiles,
        memory.know_tiles.get(&pos.grid_id),
        &map,
        ctx,
    );
    draw_map_objects(&camera, &visibility.visible_tiles, &state.ecs, ctx);
}

fn draw_map(
    camera: &Camera,
    visible_cells: &Vec<V2I>,
    know_cells: Option<&HashSet<V2I>>,
    gmap: &Area,
    ctx: &mut Rltk,
) {
    for c in camera.list_cells() {
        let cell = gmap.get_grid().get_at_opt(c.point);
        let tile = cell.unwrap_or_default().tile;

        // calculate real tile
        let (mut fg, mut bg, mut ch) = match tile {
            Tile::Ground => (rltk::LIGHT_GRAY, rltk::BLACK, '.'),
            Tile::Floor => (rltk::LIGHT_GREEN, rltk::BLACK, '.'),
            Tile::Wall => (rltk::GREEN, rltk::BLACK, '#'),
            Tile::Space => (rltk::BLACK, rltk::BLACK, ' '),
            Tile::OutOfMap => (rltk::BLACK, rltk::GRAY, ' '),
        };

        // replace non visible tiles
        if visible_cells
            .iter()
            .find(|p| c.point.x == p.x && c.point.y == p.y)
            .is_none()
        {
            if know_cells
                .map(|i| i.contains(&c.point.into()))
                .unwrap_or(false)
            {
                // if is know
                fg = rltk::GRAY;
            } else {
                // unknown
                fg = rltk::BLACK;
                bg = rltk::BLACK;
                ch = ' ';
            }
        }

        ctx.set(
            c.screen_point.x,
            c.screen_point.y,
            fg,
            bg,
            ch as rltk::FontCharType,
        );
    }
}

fn draw_map_objects(camera: &Camera, visible_cells: &Vec<V2I>, ecs: &World, ctx: &mut Rltk) {
    let mut query = ecs.query::<(&Position, &Renderable)>();
    let mut objects = query
        .into_iter()
        .filter(|(_, (pos, _))| Some(pos.grid_id) == camera.grid_id)
        .map(|(_, c)| c)
        .collect::<Vec<_>>();
    objects.sort_by(|&a, &b| a.1.priority.cmp(&b.1.priority));

    for (pos, render) in objects {
        let point = &pos.point;
        let screen_point = camera.global_to_screen(*point);

        if camera.is_global_in(*point) {
            if visible_cells
                .iter()
                .find(|p| p.x == point.x && p.y == point.y)
                .is_some()
            {
                ctx.set(
                    screen_point.x,
                    screen_point.y,
                    render.fg,
                    render.bg,
                    render.glyph,
                );
            }
        }
    }
}

pub fn draw_mouse(_state: &mut State, ctx: &mut Rltk) {
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
}

pub fn draw_game_window(state: &mut State, ctx: &mut Rltk) {
    draw_map_and_objects(state, ctx, RectI::new(0, 0, cfg::SCREEN_W, cfg::SCREEN_H));
    draw_gui(
        state,
        ctx,
        RectI::new(0, cfg::SCREEN_H - 10, cfg::SCREEN_W, 9),
    );
}

fn draw_gui(state: &State, ctx: &mut Rltk, rect: RectI) {
    let width4 = rect.get_width() / 4;
    draw_info_box(
        &state,
        ctx,
        RectI::new(rect.get_x(), rect.get_y(), width4, rect.get_height()),
    );

    draw_log_box(
        state,
        ctx,
        RectI::new(
            rect.get_x() + width4,
            rect.get_y(),
            3 * width4,
            rect.get_height(),
        ),
    );
}

fn draw_info_box(state: &State, ctx: &mut BTerm, rect: RectI) {
    for (_avatar_id, (position, actions, health)) in
        &mut state.ecs.query::<(&Position, &EntityActions, &Health)>()
    {
        let gmap = GridRef::find_area(&state.ecs, position.grid_id).unwrap();

        let tile = gmap
            .get_grid()
            .get_at_opt(position.point)
            .unwrap_or_default();
        let objects_at = find_objects_at(&state.ecs, position);

        draw_gui_bottom_box(
            ctx,
            rect.clone(),
            tile.tile,
            &objects_at,
            &map_actions_to_keys(&actions.available)
                .iter()
                .map(ViewAction::to_tuple)
                .collect::<Vec<_>>(),
            (health.hp, health.max_hp),
        );
    }
}

fn draw_log_box(state: &State, ctx: &mut Rltk, rect: RectI) {
    ctx.draw_box(
        rect.get_x(),
        rect.get_y(),
        rect.get_width(),
        rect.get_height(),
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let x = rect.get_x() + 1;
    let mut y = rect.get_y() + 1;

    let msgs = state.logs.list();
    let free_space = (rect.get_height() - 1) as usize;
    let slice_index = if msgs.len() > free_space {
        msgs.len() - free_space
    } else {
        0
    };

    for log in &msgs[slice_index..] {
        ctx.print_color(x, y, log.fg(), log.bg(), log.to_string());
        y += 1;
    }
}

struct ViewAction {
    ch: char,
    label: &'static str,
}

impl ViewAction {
    fn to_tuple(&self) -> (char, &'static str) {
        (self.ch, self.label)
    }

    fn map_to_keys(action: &Action) -> (char, &'static str) {
        match action {
            Action::Interact => ('i', "check cockpit"),
            _ => ('?', "unknown"),
        }
    }
}

fn map_actions_to_keys(actions: &Vec<Action>) -> Vec<ViewAction> {
    actions
        .iter()
        .enumerate()
        .map(|(_i, action)| {
            let (c, s) = ViewAction::map_to_keys(action);
            ViewAction { ch: c, label: s }
        })
        .collect()
}

fn draw_gui_bottom_box(
    ctx: &mut Rltk,
    rect: RectI,
    current_tile: Tile,
    objects: &Vec<(Entity, ObjectsKind)>,
    actions: &Vec<(char, &str)>,
    player_health: (Hp, Hp),
) {
    let box_x = rect.get_x();
    let box_y = rect.get_y();
    let box_h = rect.get_height();
    let box_w = rect.get_width();
    ctx.draw_box(
        box_x,
        box_y,
        box_w,
        box_h,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let text_x = box_x + 1;
    let mut text_y = box_y + 1;

    ctx.printer(
        text_x,
        text_y,
        format!(
            "#[gray]HP: #[red]{}#[gray]/#[red]{}",
            player_health.0, player_health.1
        ),
        TextAlign::Left,
        None,
    );
    text_y += 1;

    let tile_str = match current_tile {
        Tile::Ground => "ground",
        Tile::Floor => "floor",
        Tile::Wall => "?",
        Tile::Space => "space",
        Tile::OutOfMap => "oom",
    };
    ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, tile_str);
    text_y += 1;

    for (_, k) in objects {
        let obj_str = match k {
            ObjectsKind::Door { .. } => "door",
            ObjectsKind::Cockpit => "cockpit",
            _ => continue,
        };

        ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, obj_str);
        text_y += 1;
    }

    for (chr, action) in actions {
        ctx.print_color(text_x, text_y, rltk::RED, rltk::BLACK, chr);
        ctx.print_color(text_x + 1, text_y, rltk::GRAY, rltk::BLACK, " - ");
        ctx.print_color(text_x + 4, text_y, rltk::GRAY, rltk::BLACK, action);
        text_y += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /*
          0 1 2 3 4 5 ...
        0
        1   * * * * *
        2   * * * * *
        3   * * * * *
        4   * * * * *
        .
        .

    */
    #[test]
    fn test_camera_to_local_and_global() {
        let camera = Camera {
            grid_id: None,
            x: 1,
            y: 1,
            w: 4,
            h: 3,
        };

        assert_eq!(P2::new(1, 1), camera.screen_to_global((0, 0).into()));
        assert_eq!(P2::new(6, 5), camera.screen_to_global((5, 4).into()));
        assert_eq!(P2::new(0, 0), camera.global_to_screen((1, 1).into()));
        assert_eq!(P2::new(-1, -1), camera.global_to_screen((0, 0).into()));
    }
    #[test]
    fn test_camera_iterator() {
        let camera = Camera {
            grid_id: None,
            x: 1,
            y: 1,
            w: 4,
            h: 3,
        };

        let cells = camera.list_cells().collect::<Vec<_>>();
        assert_eq!(12, cells.len());
        assert_eq!(0, cells[4].screen_point.x);
        assert_eq!(1, cells[4].screen_point.y);
        assert_eq!(1, cells[4].point.x);
        assert_eq!(2, cells[4].point.y);
    }
}
