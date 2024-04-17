use crate::models::{Dir, ObjectsType, Player, Position};
use log::debug;

use crate::gridref::GridRef;
use specs::prelude::*;
use specs_derive::*;

pub mod actions_system;
pub mod avatar_actions_system;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Interact,
    Move(Dir),
}

#[derive(Debug, Clone, Component)]
pub struct EntityActions {
    /// list of actions that a entity can do
    pub actions: Vec<Action>,
    /// what the entity is assigned to do
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
    let grids = ecs.read_storage::<GridRef>();
    let player = ecs.fetch::<Player>();

    for (avatar_id, pos) in (player.get_avatarset(), &mut positions).join() {
        let map = GridRef::find_area(&grids, pos.grid_id).unwrap();

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

pub fn set_current_action(ecs: &mut World, action: Action) {
    let player = ecs.fetch::<Player>();
    let mut actions = ecs.write_storage::<EntityActions>();
    for (_, entity_action) in (player.get_avatarset(), &mut actions).join() {
        entity_action.current = Some(action.clone());
    }
}

pub fn get_available_actions(objects_at_cell: &Vec<(Entity, ObjectsType)>) -> Vec<Action> {
    let mut actions = vec![];

    for (_, kind) in objects_at_cell {
        match kind {
            ObjectsType::Cockpit => {
                actions.push(Action::Interact);
            }
            _ => {}
        }
    }

    actions
}
