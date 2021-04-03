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

#[derive(Component, Clone, Debug)]
pub struct Camera {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub struct CameraCell {
    pub screen_point: Point,
    pub point: Point,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            x: 0,
            y: 0,
            w: cfg::SCREEN_W,
            h: cfg::SCREEN_H,
        }
    }

    pub fn fromCenter(p: Point) -> Self {
        let w = cfg::SCREEN_W;
        let h = cfg::SCREEN_H;

        Camera {
            x: p.x - w / 2,
            y: p.y - h / 2,
            w: w,
            h: h,
        }
    }

    pub fn globla_rect(&self) -> Rect {
        Rect::with_size(self.x, self.y, self.w, self.h)
    }

    pub fn global_center(&self) -> Point {
        self.globla_rect().center()
    }

    pub fn global_to_screen(&self, p: Point) -> Point {
        (p.x - self.x, p.y - self.y).into()
    }

    pub fn screen_to_global(&self, p: Point) -> Point {
        (p.x + self.x, p.y + self.y).into()
    }

    pub fn is_global_in(&self, p: Point) -> bool {
        self.globla_rect().point_in_rect(p)
    }

    pub fn list_cells<'a>(&'a self) -> impl Iterator<Item = CameraCell> + 'a {
        CameraIterator {
            camera: self,
            current: 0,
        }
    }
}

struct CameraIterator<'a> {
    camera: &'a Camera,
    current: i32,
}

impl<'a> Iterator for CameraIterator<'a> {
    type Item = CameraCell;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.current % self.camera.w;
        let y = self.current / self.camera.w;

        let screen_point = Point { x, y };
        if y >= self.camera.h {
            return None;
        }

        let point = self.camera.screen_to_global(screen_point);
        let next = Some(CameraCell {
            screen_point: screen_point,
            point: point,
        });

        self.current += 1;
        next
    }
}

pub fn draw_map(
    camera: &Camera,
    visible_cells: &Vec<rltk::Point>,
    know_cells: &HashSet<rltk::Point>,
    map: &GMap,
    ctx: &mut Rltk,
) {
    for cell in camera.list_cells() {
        let tile = if map.in_bounds(cell.point) {
            let index = map.point2d_to_index(cell.point);
            map.cells[index].tile
        } else {
            TileType::OutOfMap
        };

        // calculate real tile
        let (mut fg, mut bg, mut ch) = match tile {
            TileType::Floor => (rltk::LIGHT_GREEN, rltk::BLACK, '.'),
            TileType::Wall => (rltk::GREEN, rltk::BLACK, '#'),
            TileType::Space => (rltk::BLACK, rltk::BLACK, ' '),
            TileType::OutOfMap => (rltk::BLACK, rltk::GRAY, ' '),
        };

        // replace non visible tiles
        if visible_cells.iter().find(|p| **p == cell.point).is_none() {
            if know_cells.contains(&cell.point) {
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
            cell.screen_point.x,
            cell.screen_point.y,
            fg,
            bg,
            ch as rltk::FontCharType,
        );
    }
}

pub fn draw_objects(
    camera: &Camera,
    visible_cells: &Vec<rltk::Point>,
    ecs: &World,
    ctx: &mut Rltk,
) {
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
        TileType::OutOfMap => "oom",
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
            x: 1,
            y: 1,
            w: 4,
            h: 3,
        };

        assert_eq!(Point::new(1, 1), camera.screen_to_global((0, 0).into()));
        assert_eq!(Point::new(6, 5), camera.screen_to_global((5, 4).into()));
        assert_eq!(Point::new(0, 0), camera.global_to_screen((1, 1).into()));
        assert_eq!(Point::new(-1, -1), camera.global_to_screen((0, 0).into()));
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
