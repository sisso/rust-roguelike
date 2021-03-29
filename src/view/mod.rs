use crate::cfg;
use crate::gmap::{GMap, TileType};
use crate::models::{Avatar, ObjectsType, Position};
use crate::State;
use rltk::{Algorithm2D, Point, Rect, Rltk, RGB};
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

pub fn draw_map(
    center: Point,
    visible_cells: &Vec<rltk::Point>,
    know_cells: &HashSet<rltk::Point>,
    map: &GMap,
    ctx: &mut Rltk,
) {
    let sw = cfg::SCREEN_W;
    let sh = cfg::SCREEN_H;

    for i in 0..sw {
        for j in 0..sh {
            let x = center.x - sw / 2 + i;
            let y = center.y - sh / 2 + j;
            let tile = if map.in_bounds((x, y).into()) {
                let index = map.point2d_to_index((x, y).into());
                map.cells[index].tile
            } else {
                TileType::Space
            };

            // calculate real tile
            let (mut fg, mut bg, mut ch) = match tile {
                TileType::Floor => (rltk::LIGHT_GREEN, rltk::BLACK, '.'),
                TileType::Wall => (rltk::GREEN, rltk::BLACK, '#'),
                TileType::Space => (rltk::BLACK, rltk::BLACK, ' '),
            };

            // replace non visible tiles
            if visible_cells
                .iter()
                .find(|p| p.x == x && p.y == y)
                .is_none()
            {
                if know_cells.contains(&rltk::Point { x, y }) {
                    // if is know
                    fg = rltk::GRAY;
                } else {
                    // unknown
                    fg = rltk::BLACK;
                    bg = rltk::BLACK;
                    ch = ' ';
                }
            }

            ctx.set(x, y, fg, bg, ch as rltk::FontCharType);
        }
    }
}

pub fn draw_objects(center: Point, visible_cells: &Vec<rltk::Point>, ecs: &World, ctx: &mut Rltk) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let mut objects = (&positions, &renderables).join().collect::<Vec<_>>();
    objects.sort_by(|&a, &b| a.1.priority.cmp(&b.1.priority));

    let sw = cfg::SCREEN_W;
    let sh = cfg::SCREEN_H;
    let start_x = center.x - sw / 2;
    let start_y = center.y - sh / 2;
    let area = Rect::with_size(start_x, start_y, sw, sh);

    for (pos, render) in objects {
        let point = &pos.point;

        if area.point_in_rect(*point) {
            if visible_cells
                .iter()
                .find(|p| p.x == point.x && p.y == point.y)
                .is_some()
            {
                ctx.set(point.x, point.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}

pub fn draw_gui(state: &State, ctx: &mut Rltk) {
    let avatars = &state.ecs.read_storage::<Avatar>();
    let positions = &state.ecs.read_storage::<Position>();
    let map = &state.ecs.fetch::<GMap>();

    for (avatar, position) in (avatars, positions).join() {
        let tile = map
            .cells
            .get(map.point2d_to_index(position.point))
            .unwrap()
            .tile;

        let objects = find_objects_at(&state.ecs, position.point.x, position.point.y);
        draw_gui_bottom_box(ctx, tile, &objects);
    }
}

fn find_objects_at(ecs: &World, x: i32, y: i32) -> Vec<(Entity, ObjectsType)> {
    let objects = &ecs.read_storage::<ObjectsType>();
    let entities = &ecs.entities();
    let positions = &ecs.read_storage::<Position>();

    let mut result = vec![];
    for (e, o, p) in (entities, objects, positions).join() {
        let p = p.point;
        if p.x == x && p.y == y {
            result.push((e.clone(), o.clone()));
        }
    }
    result
}

fn draw_gui_bottom_box(
    ctx: &mut Rltk,
    current_tile: TileType,
    objects: &Vec<(Entity, ObjectsType)>,
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
        TileType::Floor => "floor",
        TileType::Wall => "?",
        TileType::Space => "space",
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
}
