use crate::actions::Action;
use crate::commons::grid::Coord;
use crate::commons::v2i::V2I;
use crate::state::State;
use crate::{actions, ai};
use hecs::{Entity, World};
use rltk::{Rltk, VirtualKeyCode};

#[derive(Debug, Default)]
pub struct ShootWindowState {
    pub target: Option<Coord>,
}

pub fn run_main_window(state: &mut State, ctx: &mut Rltk) {
    player_input(state, ctx);
    super::draw_game_window(state, ctx);
}

pub fn run_shoot_window(state: &mut State, ctx: &mut Rltk) {
    player_shoot_input(state, ctx);
    super::draw_game_window(state, ctx);
}

fn player_shoot_input(state: &mut State, ctx: &mut Rltk) {
    todo!()
}

fn set_action_move(ecs: &mut World, avatar_id: Entity, delta_x: i32, delta_y: i32) {
    actions::set_current_action(ecs, avatar_id, Action::Move(V2I::new(delta_x, delta_y)));
    ai::give_turn_to_ai(ecs);
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    let avatar_id = gs.player.get_avatar_id();

    if let Some(dir) = crate::view::read_key_direction(ctx) {
        set_action_move(&mut gs.ecs, avatar_id, dir.x, dir.y);
    }

    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::I => {
                actions::set_current_action(&mut gs.ecs, avatar_id, Action::Interact);
            }
            VirtualKeyCode::F => {
                actions::set_current_action(&mut gs.ecs, avatar_id, Action::SearchToShoot);
            }
            // VirtualKeyCode::W => gs.camera.y -= 1,
            // VirtualKeyCode::A => gs.camera.x -= 1,
            // VirtualKeyCode::D => gs.camera.x += 1,
            // VirtualKeyCode::S => gs.camera.y += 1,
            _ => {}
        },
    }
}
