use crate::commons::v2i::V2I;
use crate::gmap::{Cell, GMap, GMapTile};
use crate::models::{ObjectsType, Player, Position};
use crate::utils::find_objects_at;
use crate::view::window::Window;
use log::debug;
use rltk::{Algorithm2D, Point};
use specs::prelude::*;
use specs_derive::*;
use std::borrow::Borrow;
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
    let grids = ecs.read_storage::<GMap>();
    let avatars = ecs.fetch::<Player>();

    for (avatar_id, pos) in (avatars.get_avatarset(), &mut positions).join() {
        let map = grids.borrow().get(pos.grid_id).unwrap();

        let new_pos = pos.point.translate(delta_x, delta_y);
        match map.get_grid().get_at(&new_pos) {
            Some(cell) if !cell.tile.is_opaque() => {
                debug!("{:?} move to position {:?}", avatar_id, new_pos);
                pos.point = new_pos;
            }
            _ => {
                debug!(
                    "{:?} try to move to invalid position {:?}",
                    avatar_id, new_pos
                );
            }
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
            let at_list = find_objects_at(&entities, &objects, &positions, pos);

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
