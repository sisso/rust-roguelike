use crate::gmap::{GMap, TileType};
use crate::models::{ObjectsType, Player, Position};
use crate::utils::find_objects_at;
use crate::view::window::Window;
use rltk::{Algorithm2D, Point};
use specs::prelude::*;
use specs_derive::*;
use std::cmp::{max, min};

pub mod actions_system;
pub mod avatar_actions_system;

#[derive(Debug, Clone)]
pub enum Action {
    CheckCockpit,
}

#[derive(Debug, Clone, Component)]
pub struct EntityActions {
    pub actions: Vec<Action>,
    pub current: Option<Action>,
}

impl EntityActions {
    pub fn new() -> Self {
        EntityActions {
            actions: vec![],
            current: None,
        }
    }
}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let avatar = ecs.fetch::<Player>();
    let map = ecs.fetch::<GMap>();

    for (_player, has_pos) in (avatar.get_avatarset(), &mut positions).join() {
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

// TOOD: merge with get_available_actions
pub fn try_interact(ecs: &mut World) {
    let mut apply = vec![];

    {
        let entities = ecs.entities();
        let positions = ecs.read_storage::<Position>();
        let avatar = ecs.fetch::<Player>();
        let objects = ecs.read_storage::<ObjectsType>();

        'outer: for (_, pos) in (avatar.get_avatarset(), &positions).join() {
            let at_list =
                find_objects_at(&entities, &objects, &positions, pos.point.x, pos.point.y);

            for (_e, t) in at_list {
                match t {
                    ObjectsType::Cockpit => {
                        apply.push(move |ecs: &mut World| {
                            ecs.insert(Window::Cockpit);
                        });
                        break 'outer;
                    }
                    _ => {}
                }
            }
        }
    }

    for a in apply {
        a(ecs);
    }
}

pub fn get_available_actions(objects_at_cell: &Vec<(Entity, ObjectsType)>) -> Vec<Action> {
    let mut actions = vec![];

    for (_, kind) in objects_at_cell {
        match kind {
            ObjectsType::Cockpit => {
                actions.push(Action::CheckCockpit);
            }
            _ => {}
        }
    }

    actions
}
