use crate::gmap::{GMap, TileType};
use crate::models::{Avatar, Position};
use rltk::{Algorithm2D, Point};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Avatar>();
    let map = ecs.fetch::<GMap>();

    for (_player, has_pos) in (&mut players, &mut positions).join() {
        let new_pos = Point::new(has_pos.point.x + delta_x, has_pos.point.y + delta_y);

        if !map.in_bounds(new_pos) {
            return;
        }

        if map.get_cell(new_pos) != Some(TileType::Wall) {
            has_pos.point = new_pos;
        }
    }
}
