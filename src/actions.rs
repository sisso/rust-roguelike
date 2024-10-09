use crate::combat;
use crate::combat::{CombatResult, CombatStats};
use crate::commons::grid::BaseGrid;
use crate::commons::v2i::V2I;
use crate::game_log::{GameLog, Msg};
use crate::gridref::GridRef;
use crate::health::Health;
use crate::inventory::Inventory;
use crate::models::{Label, ObjectsKind, Position};
use crate::team::Team;
use crate::utils;
use crate::view::window::{Window, WindowManage};
use hecs::{CommandBuffer, Entity, World};
use rand::rngs::StdRng;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Interact(Entity),
    Move(V2I),
    Pickup(Entity),
}

#[derive(Clone, Debug)]
pub struct WantPickup(Entity);

#[derive(Clone, Debug)]
pub struct WantMove {
    pub(crate) dir: V2I,
}

#[derive(Clone, Debug)]
pub struct WantAttack {
    pub target_id: Entity,
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

pub fn get_available_actions(objects_at_cell: &Vec<(Entity, ObjectsKind, Label)>) -> Vec<Action> {
    let mut actions = vec![];
    for (id, kind, _) in objects_at_cell {
        if kind.can_interact() {
            actions.push(Action::Interact(*id));
        }
        if kind.can_pickup() {
            actions.push(Action::Pickup(*id));
        }
    }
    actions
}

pub fn run_available_actions_system(world: &mut World) {
    for (_, (actions, pos)) in &mut world.query::<(&mut EntityActions, &Position)>() {
        let objects_at = utils::find_objects_at(&world, *pos);
        actions.available = get_available_actions(&objects_at);
    }
}

fn run_action_assign_system(world: &mut World, window_manage: &mut WindowManage) {
    let mut buffer = CommandBuffer::new();
    for (e, (actions, pos)) in &mut world.query::<(&mut EntityActions, &Position)>() {
        match actions.requested.take() {
            Some(Action::Interact(target_id)) => {
                if utils::get_kind(world, target_id).can_interact() {
                    window_manage.set_window(Window::Cockpit {
                        cockpit_id: target_id,
                    });
                } else {
                    log::warn!("{e:?} try to interact but not object has interaction")
                }
            }
            Some(Action::Move(dir)) => buffer.insert_one(e, WantMove { dir }),
            Some(Action::Pickup(id)) => buffer.insert_one(e, WantPickup(id)),
            None => {}
        }
    }
    buffer.run_on(world);
}

pub fn run_actions_system(
    world: &mut World,
    // window: &mut Window,
    window_manage: &mut WindowManage,
    game_log: &mut GameLog,
    rng: &mut StdRng,
    player_id: Entity,
) {
    run_action_assign_system(world, window_manage);
    run_action_wantmove_system(world, game_log, player_id);
    run_action_attack_system(world, game_log, rng, player_id);
    run_action_pickup_system(world, game_log, player_id);
}

fn run_action_pickup_system(world: &mut World, logs: &mut GameLog, player_id: Entity) {
    let mut buffer = CommandBuffer::new();
    for (e, (inventory, action, pos)) in
        &mut world.query::<(&mut Inventory, &WantPickup, &Position)>()
    {
        buffer.remove_one::<WantPickup>(e);

        // get target position
        let Some(target_pos) = utils::get_position(world, action.0) else {
            log::warn!("fail to pickup item, target has no position");
            continue;
        };

        if *pos != target_pos {
            log::warn!("fail to pickup item, item is not at same position");
            continue;
        }

        buffer.remove_one::<Position>(action.0);
        inventory.items.push(action.0);
    }
    buffer.run_on(world);
}

fn run_action_attack_system(
    world: &mut World,
    logs: &mut GameLog,
    rng: &mut StdRng,
    player_id: Entity,
) {
    let mut buffer = CommandBuffer::new();
    for (
        attacker_id,
        (
            WantAttack {
                target_id: defender_id,
            },
            attack_combat_status,
        ),
    ) in &mut world.query::<(&WantAttack, &CombatStats)>()
    {
        buffer.remove_one::<WantAttack>(attacker_id);

        let mut query = world.query_one::<&CombatStats>(*defender_id).unwrap();
        let defender_attack_status = query.get();

        let cs = combat::execute_attack(rng, attack_combat_status, defender_attack_status);

        match cs {
            CombatResult::AttackHit {
                hit_roll,
                hit_require,
                damage_roll,
            } => {
                let mut query = world.query_one::<&mut Health>(*defender_id).unwrap();
                let target_health = query.get().unwrap();
                target_health.pending_damage.push(1);

                if player_id == attacker_id {
                    logs.push(Msg::PlayerAttack {
                        hit_roll,
                        hit_require,
                        damage: damage_roll,
                    });
                } else if *defender_id == player_id {
                    logs.push(Msg::PlayerReceiveAttack {
                        hit_roll,
                        hit_require,
                        damage: damage_roll,
                    });
                }
            }
            CombatResult::Defend {
                hit_roll,
                hit_require,
            } => {
                if player_id == attacker_id {
                    logs.push(Msg::PlayerMissAttack {
                        hit_roll,
                        hit_require,
                    });
                } else if *defender_id == player_id {
                    logs.push(Msg::PlayerReceiveMissAttack {
                        hit_roll,
                        hit_require,
                    });
                }
            }
        }
    }
    buffer.run_on(world);
}

fn run_action_wantmove_system(world: &mut World, game_log: &mut GameLog, player_id: Entity) {
    let mut query = world.query::<(&WantMove, &Position, &Team)>();
    let candidates = query
        .iter()
        .map(|(id, (WantMove { dir }, _, team))| (id, *dir, team));

    let mut buffer = CommandBuffer::new();
    for (id, dir, team) in candidates {
        buffer.remove_one::<WantMove>(id);

        let is_player = id == player_id;

        let mut query = world.query_one::<&Position>(id).unwrap();
        let pos = query.get().unwrap();
        let next_pos = pos.translate_by(dir);

        if can_move_into(world, id, &next_pos) {
            let mob_on_next_cell = utils::find_damageable_at(world, next_pos, *team);
            if let Some(target_id) = mob_on_next_cell.into_iter().next() {
                buffer.insert_one(id, WantAttack { target_id });
            } else {
                buffer.insert_one(id, next_pos);
                if is_player {
                    game_log.push(Msg::PlayerMove);
                }
            }
        } else {
            if is_player {
                game_log.push(Msg::PlayerFailMove);
            }
        }
    }

    drop(query);
    buffer.run_on(world);
}

fn can_move_into(world: &World, e: Entity, pos: &Position) -> bool {
    let area = GridRef::find_area(world, pos.grid_id).unwrap();
    area.get_grid()
        .get_at_opt(pos.point)
        .map(|t| t.tile.is_opaque() == false)
        .unwrap_or(false)
}
