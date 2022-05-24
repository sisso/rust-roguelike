pub mod camera;
pub mod cockpit_window;
pub mod window;

use crate::actions::{Action, EntityActions};
use crate::gmap::{GMap, GMapTile};
use crate::models::{ObjectsType, Player, Position};
use crate::utils::find_objects_at;
use crate::view::camera::Camera;
use crate::{actions, cfg};
use crate::{State, P2};
use rltk::{Algorithm2D, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::*;
use std::collections::HashSet;

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub know_tiles: HashSet<rltk::Point>,
    pub range: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub priority: i32,
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => actions::try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => actions::try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => actions::try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => actions::try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad7 => actions::try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad8 => actions::try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad9 => actions::try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad4 => actions::try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad5 => actions::try_move_player(0, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad6 => actions::try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad1 => actions::try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad2 => actions::try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad3 => actions::try_move_player(1, 1, &mut gs.ecs),
            VirtualKeyCode::I => actions::try_interact(&mut gs.ecs),
            // VirtualKeyCode::W => gs.camera.y -= 1,
            // VirtualKeyCode::A => gs.camera.x -= 1,
            // VirtualKeyCode::D => gs.camera.x += 1,
            // VirtualKeyCode::S => gs.camera.y += 1,
            _ => {}
        },
    }
}

// pub fn view_input(state: &mut State, ctx: &mut Rltk) {
//     let chkey = match ctx.key {
//         Some(VirtualKeyCode::I) => 'i',
//         _ => return,
//     };
//
//     let avatars = &state.ecs.read_storage::<Avatar>();
//     let positions = &state.ecs.read_storage::<Position>();
//     let actions_st = &mut state.ecs.write_storage::<EntityActions>();
//
//     for (avatar, position, actions) in (avatars, positions, actions_st).join() {
//         let view_actions = map_actions_to_keys(&actions.actions);
//
//         match view_actions.iter().find(|va| va.ch == chkey) {
//             Some(va) => {
//                 actions.current = Some(va.action.clone());
//             }
//             _ => {}
//         }
//     }
// }

pub fn draw_mouse(_state: &mut State, ctx: &mut Rltk) {
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
}

pub fn draw_map_and_objects(state: &mut State, ctx: &mut Rltk) {
    // merge all visible and know tiles from player
    let viewshed = state.ecs.read_storage::<Viewshed>();
    let avatars = state.ecs.fetch::<Player>();
    let positions = state.ecs.read_storage::<Position>();
    let views = (&viewshed, avatars.get_avatarset(), &positions)
        .join()
        .collect::<Vec<_>>();
    let (v, _, pos) = views.iter().next().unwrap();

    let camera = Camera::from_center(pos.point);

    // draw
    let grids = &state.ecs.read_storage::<GMap>();
    let map = grids.get(pos.grid_id).unwrap();
    draw_map(&camera, &v.visible_tiles, &v.know_tiles, &map, ctx);
    draw_objects(&camera, &v.visible_tiles, &state.ecs, ctx);
}

impl Into<rltk::Point> for P2 {
    fn into(self) -> rltk::Point {
        rltk::Point {
            x: self.x,
            y: self.y,
        }
    }
}

fn draw_map(
    camera: &Camera,
    visible_cells: &Vec<rltk::Point>,
    know_cells: &HashSet<rltk::Point>,
    gmap: &GMap,
    ctx: &mut Rltk,
) {
    for c in camera.list_cells() {
        let cell = gmap.get_grid().get_at(&c.point);
        let tile = cell.unwrap_or_default().tile;

        // calculate real tile
        let (mut fg, mut bg, mut ch) = match tile {
            GMapTile::Ground => (rltk::LIGHT_GRAY, rltk::BLACK, '.'),
            GMapTile::Floor => (rltk::LIGHT_GREEN, rltk::BLACK, '.'),
            GMapTile::Wall => (rltk::GREEN, rltk::BLACK, '#'),
            GMapTile::Space => (rltk::BLACK, rltk::BLACK, ' '),
            GMapTile::OutOfMap => (rltk::BLACK, rltk::GRAY, ' '),
        };

        // replace non visible tiles
        if visible_cells
            .iter()
            .find(|p| c.point.x == p.x && c.point.y == p.y)
            .is_none()
        {
            if know_cells.contains(&c.point.into()) {
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

fn draw_objects(camera: &Camera, visible_cells: &Vec<rltk::Point>, ecs: &World, ctx: &mut Rltk) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let mut objects = (&positions, &renderables).join().collect::<Vec<_>>();
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

pub fn draw_gui(state: &State, ctx: &mut Rltk) {
    let entities = &state.ecs.entities();
    let objects = &state.ecs.read_storage::<ObjectsType>();
    let avatars = &state.ecs.read_storage::<Player>();
    let positions = &state.ecs.read_storage::<Position>();
    let actions_st = &state.ecs.read_storage::<EntityActions>();

    for (_avatar_id, position, actions) in (avatars, positions, actions_st).join() {
        let grids = &state.ecs.read_storage::<GMap>();
        let gmap = grids.get(position.grid_id).unwrap();

        let tile = gmap.get_grid().get_at(&position.point).unwrap_or_default();
        let objects_at = find_objects_at(entities, objects, positions, position);

        draw_gui_bottom_box(
            ctx,
            tile.tile,
            &objects_at,
            &map_actions_to_keys(&actions.actions)
                .iter()
                .map(ViewAction::to_tuple)
                .collect::<Vec<_>>(),
        );
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
            Action::CheckCockpit => ('i', "check cockpit"),
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
    current_tile: GMapTile,
    objects: &Vec<(Entity, ObjectsType)>,
    actions: &Vec<(char, &str)>,
) {
    let box_h = 6;
    let box_x = 0;
    let box_y = cfg::SCREEN_H - box_h - 1;
    let box_w = cfg::SCREEN_W - 1;
    ctx.draw_box(
        box_x,
        box_y,
        box_w,
        box_h,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let inner_box_x = box_x + 1;
    let inner_box_y = box_y + 1;
    let tile_str = match current_tile {
        GMapTile::Ground => "ground",
        GMapTile::Floor => "floor",
        GMapTile::Wall => "?",
        GMapTile::Space => "space",
        GMapTile::OutOfMap => "oom",
    };
    ctx.print_color(inner_box_x, inner_box_y, rltk::GRAY, rltk::BLACK, tile_str);

    let mut j = inner_box_y + 1;
    for (_, k) in objects {
        let obj_str = match k {
            ObjectsType::Door { .. } => "door",
            ObjectsType::Cockpit => "cockpit",
            _ => continue,
        };

        ctx.print_color(inner_box_x, j, rltk::GRAY, rltk::BLACK, obj_str);
        j += 1;
    }

    {
        let x = inner_box_x;
        let mut y = inner_box_y + 2;
        for (chr, action) in actions {
            ctx.print_color(x, y, rltk::RED, rltk::BLACK, chr);
            ctx.print_color(x + 1, y, rltk::GRAY, rltk::BLACK, " - ");
            ctx.print_color(x + 4, y, rltk::GRAY, rltk::BLACK, action);
            y += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rltk::Point;

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
