use crate::gmap::{GMap, TileType};
use crate::models::Position;
use rltk::{Rltk, RGB};
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
    visible_cells: &Vec<rltk::Point>,
    know_cells: &HashSet<rltk::Point>,
    map: &GMap,
    ctx: &mut Rltk,
) {
    let mut y = 0;
    let mut x = 0;

    for cell in &map.cells {
        // calculate real tile
        let (mut fg, mut bg, mut ch) = match cell.tile {
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

        // Move the coordinates
        x += 1;
        if x >= map.width {
            x = 0;
            y += 1;
        }
    }
}

pub fn draw_objects(visible_cells: &Vec<rltk::Point>, ecs: &World, ctx: &mut Rltk) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let mut objects = (&positions, &renderables).join().collect::<Vec<_>>();
    objects.sort_by(|&a, &b| a.1.priority.cmp(&b.1.priority));
    for (pos, render) in objects {
        if visible_cells
            .iter()
            .find(|p| p.x == pos.x && p.y == pos.y)
            .is_some()
        {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
