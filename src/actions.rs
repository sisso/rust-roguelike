use crate::models::{Dir, ObjectsType, Player, Position};
use hecs::{Entity, World};

use crate::gridref::GridRef;

pub mod actions_system;
pub mod avatar_actions_system;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Interact,
    Move(Dir),
}

#[derive(Debug, Clone)]
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

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World, player: &Player) {
    let avatar_id = player.get_avatar_id();
    let pos = ecs
        .get::<&mut Position>(avatar_id)
        .expect("Player has no position");

    let area = GridRef::find_area(ecs, pos.grid_id).unwrap();

    let new_pos = pos.point.translate(delta_x, delta_y);
    match area.get_grid().get_at(&new_pos) {
        Some(cell) if !cell.tile.is_opaque() => {
            log::debug!("{:?} move to position {:?}", avatar_id, new_pos);
            pos.point = new_pos;
        }
        _ => {
            log::debug!(
                "{:?} try to move to invalid position {:?}",
                avatar_id,
                new_pos
            );
        }
    }
}

pub fn set_current_action(ecs: &mut World, id: Entity, action: Action) {
    ecs.get::<&mut EntityActions>(id)
        .expect("Entity has no actions")
        .current = Some(action);
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
