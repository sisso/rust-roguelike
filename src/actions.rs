use crate::commons::v2i::V2I;
use crate::game_log::{GameLog, Msg};
use crate::gridref::GridRef;
use crate::health::Health;
use crate::models::{ObjectsKind, Position};
use crate::utils::{find_mobs_at, find_objects_at};
use crate::view::window::Window;
use crate::{mob, utils};
use hecs::{CommandBuffer, Entity, World};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Interact,
    Move(V2I),
}

pub struct WantInteract;
pub struct WantMove {
    dir: V2I,
}

pub struct WantAttack {
    target_id: Entity,
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

pub fn set_current_action(ecs: &mut World, id: Entity, action: Action) {
    ecs.get::<&mut EntityActions>(id)
        .expect("Entity has no actions")
        .requested = Some(action);
}

pub fn get_available_actions(objects_at_cell: &Vec<(Entity, ObjectsKind)>) -> Vec<Action> {
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

fn run_action_assign_system(world: &mut World) {
    let mut buffer = CommandBuffer::new();
    for (e, (actions, pos)) in &mut world.query::<(&mut EntityActions, &Position)>() {
        match actions.requested.take() {
            Some(Action::Interact) => buffer.insert_one(e, WantInteract),
            Some(Action::Move(dir)) => buffer.insert_one(e, WantMove { dir }),
            None => {}
        }
    }
    buffer.run_on(world);
}

pub fn run_actions_system(world: &mut World, window: &mut Window, game_log: &mut GameLog) {
    run_action_assign_system(world);
    run_action_wantinteract_system(world, window);
    run_action_wantmove_system(world, game_log);
    run_action_attack_system(world, game_log);
}

fn run_action_attack_system(world: &mut World, logs: &mut GameLog) {
    let mut buffer = CommandBuffer::new();
    for (agressor_id, (WantAttack { target_id },)) in &mut world.query::<(&WantAttack,)>() {
        buffer.remove_one::<WantAttack>(agressor_id);

        let mut query = world.query_one::<&mut Health>(*target_id).unwrap();
        let target_health = query.get().unwrap();
        target_health.pending_damage.push(1);
        logs.push(Msg::PlayerAttack {});
    }
    buffer.run_on(world);
}

fn run_action_wantmove_system(world: &mut World, game_log: &mut GameLog) {
    let mut query = world.query::<(&WantMove, &Position)>();
    let candidates = query.iter().map(|(id, (WantMove { dir }, _))| (id, *dir));

    let mut buffer = CommandBuffer::new();
    for (id, dir) in candidates {
        buffer.remove_one::<WantMove>(id);

        let mut query = world.query_one::<&Position>(id).unwrap();
        let pos = query.get().unwrap();
        let next_pos = pos.translate_by(dir);

        if can_move_into(world, id, &next_pos) {
            let mob_on_next_cell = find_mobs_at(world, &next_pos);
            if let Some(target_id) = mob_on_next_cell.into_iter().next() {
                buffer.insert_one(id, WantAttack { target_id });
            } else {
                buffer.insert_one(id, next_pos);
                game_log.push(Msg::PlayerMove);
            }
        } else {
            game_log.push(Msg::PlayerFailMove);
        }
    }

    drop(query);
    buffer.run_on(world);
}

fn can_move_into(world: &World, e: Entity, pos: &Position) -> bool {
    let area = GridRef::find_area(world, pos.grid_id).unwrap();
    area.get_grid()
        .get_at(&pos.point)
        .map(|t| t.tile.is_opaque() == false)
        .unwrap_or(false)
}

fn run_action_wantinteract_system(world: &mut World, window: &mut Window) {
    let mut buffer = CommandBuffer::new();
    for (e, (_, pos)) in &mut world.query::<(&WantInteract, &Position)>() {
        let objects_at = find_objects_at(world, &pos);
        match objects_at.into_iter().find(|(_, k)| k.can_interact()) {
            Some((id, _)) => {
                // change window to cockpit
                *window = Window::Cockpit { cockpit_id: id };
            }
            None => log::warn!("{e:?} try to interact but not object has interaction"),
        }

        buffer.remove_one::<WantInteract>(e);
    }
    buffer.run_on(world);
}
