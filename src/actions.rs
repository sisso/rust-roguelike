use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::health::Health;
use crate::mob;
use crate::mob::Mob;
use crate::models::{ObjectsKind, Position};
use crate::utils::find_objects_at;
use crate::view::window::Window;
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
            Some(Action::Interact) => buffer.insert(e, (WantInteract,)),
            Some(Action::Move(dir)) => {
                let mut objects_at = mob::find_mobs_at(world, &pos.translate_by(dir));
                if let Some(target_id) = objects_at.into_iter().into_iter().next() {
                    buffer.insert(e, (WantAttack { target_id },))
                } else {
                    buffer.insert(e, (WantMove { dir },))
                }
            }
            None => {}
        }
    }

    buffer.run_on(world);
}

pub fn run_actions_system(world: &mut World, window: &mut Window) {
    run_action_assign_system(world);
    run_action_wantinteract_system(world, window);
    run_action_wantmove_system(world);
    run_action_attack_system(world);
}

fn run_action_attack_system(world: &mut World) {
    let mut buffer = CommandBuffer::new();
    for (agressor_id, (WantAttack { target_id },)) in &mut world.query::<(&WantAttack,)>() {
        buffer.remove_one::<WantAttack>(agressor_id);

        let mut query = world.query_one::<&mut Health>(*target_id).unwrap();
        let target_health = query.get().unwrap();
        target_health.pending_damage.push(1);
    }
    buffer.run_on(world);
}

fn run_action_wantmove_system(world: &mut World) {
    let candidates = world
        .query::<(&WantMove, &Position)>()
        .iter()
        .map(|(id, (WantMove { dir }, _))| (id, *dir))
        .collect::<Vec<_>>();

    let mut buffer = CommandBuffer::new();
    for (id, dir) in candidates {
        try_move_player(world, id, dir.x, dir.y);
        buffer.remove_one::<WantMove>(id);
    }
    buffer.run_on(world);
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
