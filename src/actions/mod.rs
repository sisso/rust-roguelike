use crate::gmap::{GMap, TileType};
use crate::models::{Avatar, Position};
use rltk::{Algorithm2D, Point};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Avatar>();
    let map = ecs.fetch::<GMap>();

    for (_player, has_pos) in (&mut players, &mut positions).join() {
        let new_pos = Point::new(has_pos.point.x + delta_x, has_pos.point.y + delta_y);

        if !map.in_bounds(new_pos) {
            return;
        }

        let destination_idx = map.point2d_to_index(new_pos);
        if map.get_cell(destination_idx).tile != TileType::Wall {
            has_pos.point.x = min(map.width - 1, max(0, has_pos.point.x + delta_x));
            has_pos.point.y = min(map.height - 1, max(0, has_pos.point.y + delta_y));
        }
    }
}
