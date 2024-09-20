use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::models::{ObjectsType, Position};
use crate::utils::find_objects_at;
use crate::view::window::Window;
use hecs::{Entity, World};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Interact,
    Move(V2I),
}

#[derive(Debug, Clone)]
pub struct EntityActions {
    /// list of actions that a entity can do
    pub available: Vec<Action>,
    /// what the entity is assigned to do
    pub requested: Option<Action>,
}

impl EntityActions {
    pub fn new() -> Self {
        EntityActions {
            available: vec![],
            requested: None,
        }
    }
}

fn try_move_player(ecs: &mut World, avatar_id: Entity, delta_x: i32, delta_y: i32) {
    let mut pos = ecs
        .get::<&mut Position>(avatar_id)
        .map_err(|err| format!("fail to find avatar_id {avatar_id:?}: {err:?}"))
        .unwrap();

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
        .requested = Some(action);
}

pub fn get_available_actions(objects_at_cell: &Vec<(Entity, ObjectsType)>) -> Vec<Action> {
    let mut actions = vec![];
    for (_, kind) in objects_at_cell {
        if kind.can_interact() {
            actions.push(Action::Interact);
        }
    }
    actions
}

pub fn run_available_actions_system(world: &mut World) {
    for (_, (actions, pos)) in &mut world.query::<(&mut EntityActions, &Position)>() {
        let objects_at = find_objects_at(&world, pos);
        actions.available = get_available_actions(&objects_at);
    }
}

pub fn run_actions_system(world: &mut World, window: &mut Window) {
    let pending_actions: Vec<(Entity, Action, Position)> = world
        .query::<(&mut EntityActions, &Position)>()
        .into_iter()
        .flat_map(|(e, (actions, pos))| match actions.requested.take() {
            Some(action) => Some((e, action, pos.clone())),
            None => None,
        })
        .collect();

    for (e, req_action, pos) in pending_actions {
        match req_action {
            Action::Interact => {
                let objects_at = find_objects_at(&world, &pos);
                match objects_at.into_iter().find(|(_, k)| k.can_interact()) {
                    Some((id, _)) => {
                        // change window to cockpit
                        *window = Window::Cockpit { cockpit_id: id };
                    }
                    None => log::warn!("{e:?} try to interact but not object has interaction"),
                }
            }
            Action::Move(dir) => {
                try_move_player(world, e, dir.x, dir.y);
            }
        }
    }
}
