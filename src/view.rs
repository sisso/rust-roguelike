pub mod camera;
pub mod cockpit_window;
pub mod game_window;
pub mod lose_window;
pub mod main_menu_window;
pub mod window;

use crate::actions::{Action, EntityActions};
use crate::area::{Area, Tile};
use crate::commons::grid::BaseGrid;
use crate::commons::recti::RectI;
use crate::commons::v2i::V2I;
use crate::gridref::AreaRef;
use crate::health::{Health, Hp};
use crate::inventory::Inventory;
use crate::models::{Label, ObjKind, Position};
use crate::state::State;
use crate::view::camera::Camera;
use crate::view::game_window::SubWindow;
use crate::visibility::{Visibility, VisibilityMemory};
use crate::{cfg, utils};
use hecs::{Entity, World};
use rltk::{Rltk, TextAlign, VirtualKeyCode, RGB};
use std::collections::HashSet;

pub type Color = (u8, u8, u8);

#[derive(Debug, Clone)]
pub struct ScreenLayout {
    screen_width: i32,
    screen_height: i32,
    left_column_width: i32,
    bottom_bar_height: i32,
    info_box_width: i32,
}

impl ScreenLayout {
    pub fn new(screen_width: i32, screen_height: i32) -> Self {
        let left_column_width = 30;
        let bottom_bar_height = 9;
        let info_box_width = left_column_width;

        ScreenLayout {
            screen_width,
            screen_height,
            left_column_width,
            bottom_bar_height,
            info_box_width,
        }
    }

    pub fn get_main_area_rect(&self) -> RectI {
        RectI::new(
            self.left_column_width,
            0,
            self.screen_width - self.left_column_width,
            self.screen_height - self.bottom_bar_height,
        )
    }

    pub fn get_left_rect(&self) -> RectI {
        RectI::new(
            0,
            0,
            self.info_box_width - 1,
            self.screen_height - self.bottom_bar_height - 1,
        )
    }

    pub fn get_logs_rect(&self) -> RectI {
        RectI::new(
            self.info_box_width,
            self.screen_height - self.bottom_bar_height,
            self.screen_width - self.info_box_width - 1,
            self.bottom_bar_height - 1,
        )
    }

    pub fn get_info_rect(&self) -> RectI {
        RectI::new(
            0,
            self.screen_height - self.bottom_bar_height,
            self.info_box_width - 1,
            self.bottom_bar_height - 1,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    // bigger show above lower
    pub priority: i32,
}

pub fn get_window_rect() -> RectI {
    RectI::new(0, 0, cfg::SCREEN_W, cfg::SCREEN_H)
}

pub fn draw_rect(ctx: &mut Rltk, rect: RectI, fg: Color, bg: Color, title: Option<&str>) {
    ctx.draw_box(
        rect.get_x(),
        rect.get_y(),
        rect.get_width(),
        rect.get_height(),
        fg,
        bg,
    );

    if let Some(title) = title {
        ctx.print_color(
            rect.get_x() + 2,
            rect.get_y(),
            rltk::GRAY,
            rltk::BLACK,
            title,
        );
    }
}

fn draw_map_and_objects(state: &State, ctx: &mut Rltk) {
    let avatar_id = state.player.get_avatar_id();

    let mut query = state
        .ecs
        .query_one::<(&Visibility, &Position, &VisibilityMemory)>(avatar_id)
        .expect("player avatar not found");
    let (visibility, avatar_pos, memory) = query.get().expect("player not found");

    // TODO: add camera to state
    let camera = Camera::from_center(*avatar_pos, state.screen_layout.get_main_area_rect());

    // draw
    let map = AreaRef::resolve_area(&state.ecs, avatar_pos.grid_id).expect("area not found");
    draw_map(
        &camera,
        &visibility.visible_tiles,
        memory.know_tiles.get(&avatar_pos.grid_id),
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
        let cell = gmap.get_grid().get_at_opt(c.world_pos);
        let tile = cell.map(|c| c.tile()).unwrap_or_default();

        // calculate real tile
        let (mut fg, mut bg, mut ch) = match tile {
            Tile::Ground => (rltk::LIGHT_GRAY, rltk::BLACK, '.'),
            Tile::Floor => (rltk::LIGHT_GREEN, rltk::BLACK, '.'),
            Tile::Wall => (rltk::GREEN, rltk::BLACK, '#'),
            Tile::Space => (rltk::BLACK, rltk::BLACK, '?'),
            Tile::OutOfMap => (rltk::BLACK, rltk::GRAY, '%'),
        };

        // replace non visible tiles
        if visible_cells
            .iter()
            .find(|p| c.world_pos.x == p.x && c.world_pos.y == p.y)
            .is_none()
        {
            if know_cells
                .map(|i| i.contains(&c.world_pos.into()))
                .unwrap_or(false)
            {
                // if is know
                fg = rltk::GRAY;
            } else {
                // unknown
                fg = rltk::BLACK;
                bg = rltk::BLACK;
                ch = '?';
            }
        }

        ctx.set(
            c.screen_pos.x,
            c.screen_pos.y,
            fg,
            bg,
            ch as rltk::FontCharType,
        );
    }
}

fn draw_map_objects(camera: &Camera, visible_cells: &Vec<V2I>, ecs: &World, ctx: &mut Rltk) {
    let mut query = ecs.query::<(&Position, &Renderable)>();
    // find objects in the grid
    let mut objects = query
        .into_iter()
        .filter(|(_, (pos, _))| pos.grid_id == camera.grid_id)
        .map(|(_, c)| c)
        .collect::<Vec<_>>();
    objects.sort_by(|&a, &b| a.1.priority.cmp(&b.1.priority));

    for (pos, render) in objects {
        if camera.in_world(pos.point) {
            let screen_point = camera.world_to_screen(pos.point);

            if visible_cells
                .iter()
                .find(|p| p.x == pos.point.x && p.y == pos.point.y)
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

/// Default screen include map and gui
pub fn draw_default(state: &State, ctx: &mut Rltk) {
    draw_map_and_objects(state, ctx);
    draw_info_box(&state, ctx);
    draw_log_box(state, ctx);
    draw_character_box(state, ctx);
}

fn draw_character_box(state: &State, ctx: &mut Rltk) {
    let avatar_id = state.player.get_avatar_id();

    let entity = state.ecs.entity(avatar_id).unwrap();
    let actions = entity.get::<&EntityActions>().unwrap();
    let health = entity.get::<&Health>().unwrap();
    let inventory = entity.get::<&Inventory>().unwrap();
    let items_labels = utils::find_labels(&state.ecs, inventory.items.iter());

    let rect = state.screen_layout.get_left_rect();
    let actions = map_actions_to_keys(&actions.available);

    draw_rect(ctx, rect, rltk::WHITE, rltk::BLACK, Some("Player"));

    let text_x = rect.get_x() + 1;
    let mut text_y = rect.get_y() + 2;

    ctx.printer(
        text_x,
        text_y,
        format!(
            "#[gray]HP: #[red]{}#[gray]/#[red]{}",
            health.hp, health.max_hp
        ),
        TextAlign::Left,
        None,
    );
    text_y += 1;

    // available actions
    text_y += 1;
    ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, "Actions");
    text_y += 1;
    ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, "---------");
    text_y += 1;
    for vc in actions {
        ctx.print_color(text_x, text_y, rltk::RED, rltk::BLACK, vc.ch);
        ctx.print_color(text_x + 1, text_y, rltk::GRAY, rltk::BLACK, " - ");
        ctx.print_color(text_x + 4, text_y, rltk::GRAY, rltk::BLACK, vc.label);
        text_y += 1;
    }

    // inventory
    text_y += 1;
    ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, "Inventory");
    text_y += 1;
    ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, "---------");
    text_y += 1;
    for item in items_labels {
        ctx.print_color(text_x, text_y, rltk::BLUE, rltk::BLACK, &item.name);
        text_y += 1;
    }
}

fn draw_info_box(state: &State, ctx: &mut Rltk) {
    let player_id = state.player.get_avatar_id();
    let player_pos = utils::get_position(&state.ecs, player_id).unwrap();

    // get current cell or info marker position
    let info_pos = match &state.window_manage.game_state.sub_window {
        SubWindow::Fire { point } => player_pos.with_point(*point),
        SubWindow::Info { point } => player_pos.with_point(*point),
        _ => {
            let avatar_id = state.player.get_avatar_id();
            let pos = utils::get_position(&state.ecs, avatar_id).unwrap();
            pos
        }
    };

    // draw box
    let rect = state.screen_layout.get_info_rect();
    draw_rect(ctx, rect, rltk::GRAY, rltk::BLACK, Some("Info"));

    let text_x = rect.get_x() + 1;
    let mut text_y = rect.get_y() + 1;

    // confirm the position is a player know position
    let is_know = {
        let mut q = state.ecs.query_one::<&VisibilityMemory>(player_id).unwrap();
        q.get().unwrap().is_know(info_pos)
    };

    // write info
    if is_know {
        // get cell tile
        let gmap = AreaRef::resolve_area(&state.ecs, player_pos.grid_id).unwrap();
        let current_cell = gmap
            .get_grid()
            .get_at_opt(info_pos.point)
            .map(|c| c.tile())
            .unwrap_or_default();
        let tile_str = match current_cell {
            Tile::Ground => "ground",
            Tile::Floor => "floor",
            Tile::Wall => "?",
            Tile::Space => "space",
            Tile::OutOfMap => "oom",
        };

        // get objects at cell
        let objects = utils::find_objects_at_with_label(&state.ecs, info_pos);

        ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, tile_str);
        text_y += 1;

        for (obj_id, kind, label) in objects {
            if obj_id == player_id {
                continue;
            }

            let kind_str = match kind {
                ObjKind::Door { .. } => "door",
                ObjKind::Engine => "engine",
                ObjKind::Cockpit => "cockpit",
                ObjKind::Player => "player",
                ObjKind::Mob => "mob",
                ObjKind::Item => "item",
            };

            let text = if kind_str == label.name {
                format!("{}", kind_str)
            } else {
                format!("{} ({})", label.name, kind_str)
            };
            ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, text);
            text_y += 1;
        }
    } else {
        ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, "?");
    }
}

fn draw_log_box(state: &State, ctx: &mut Rltk) {
    let rect = state.screen_layout.get_logs_rect();
    draw_rect(ctx, rect, rltk::GRAY, rltk::BLACK, Some("Log"));

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
            Action::Interact(_) => ('i', "check cockpit"),
            Action::Pickup(_) => ('p', "pick up"),
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

fn draw_character_box_content(
    ctx: &mut Rltk,
    rect: RectI,
    objects: &Vec<(Entity, ObjKind, Label)>,
    actions: &Vec<(char, &str)>,
    player_health: (Hp, Hp),
) {
    draw_rect(ctx, rect, rltk::WHITE, rltk::BLACK, Some("Player"));

    let text_x = rect.get_x() + 1;
    let mut text_y = rect.get_y() + 1;

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

    for (_, k, label) in objects {
        ctx.print_color(text_x, text_y, rltk::GRAY, rltk::BLACK, &label.name);
        text_y += 1;
    }

    for (chr, action) in actions {
        ctx.print_color(text_x, text_y, rltk::RED, rltk::BLACK, chr);
        ctx.print_color(text_x + 1, text_y, rltk::GRAY, rltk::BLACK, " - ");
        ctx.print_color(text_x + 4, text_y, rltk::GRAY, rltk::BLACK, action);
        text_y += 1;
    }
}

pub fn read_key_direction(ctx: &mut Rltk) -> Option<V2I> {
    match ctx.key {
        Some(VirtualKeyCode::Left) | Some(VirtualKeyCode::A) => Some(V2I::new(-1, 0)),
        Some(VirtualKeyCode::Right) | Some(VirtualKeyCode::D) => Some(V2I::new(1, 0)),
        Some(VirtualKeyCode::Up) | Some(VirtualKeyCode::W) => Some(V2I::new(0, -1)),
        Some(VirtualKeyCode::Down) | Some(VirtualKeyCode::S) => Some(V2I::new(0, 1)),
        Some(VirtualKeyCode::Numpad7) => Some(V2I::new(-1, -1)),
        Some(VirtualKeyCode::Numpad8) => Some(V2I::new(0, -1)),
        Some(VirtualKeyCode::Numpad9) => Some(V2I::new(1, -1)),
        Some(VirtualKeyCode::Numpad4) => Some(V2I::new(-1, 0)),
        Some(VirtualKeyCode::Numpad5) => Some(V2I::new(0, 0)),
        Some(VirtualKeyCode::Numpad6) => Some(V2I::new(1, 0)),
        Some(VirtualKeyCode::Numpad1) => Some(V2I::new(-1, 1)),
        Some(VirtualKeyCode::Numpad2) => Some(V2I::new(0, 1)),
        Some(VirtualKeyCode::Numpad3) => Some(V2I::new(1, 1)),
        _ => None,
    }
}

#[cfg(test)]
mod test {}
