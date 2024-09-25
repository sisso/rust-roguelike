use crate::actions::Action;
use crate::commons::v2i::V2I;
use crate::state::State;
use crate::{actions, ai};
use hecs::{Entity, World};
use rltk::{Rltk, VirtualKeyCode};

pub fn run_main_window(state: &mut State, ctx: &mut Rltk) {
    player_input(state, ctx);
    state.run_game_loop_systems();
    super::draw_game_window(state, ctx);
}

fn set_action_move(ecs: &mut World, avatar_id: Entity, delta_x: i32, delta_y: i32) {
    actions::set_current_action(ecs, avatar_id, Action::Move(V2I::new(delta_x, delta_y)));
    ai::give_turn_to_ai(ecs);
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    let avatar_id = gs.player.get_avatar_id();

    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => set_action_move(&mut gs.ecs, avatar_id, -1, 0),
            VirtualKeyCode::Right => set_action_move(&mut gs.ecs, avatar_id, 1, 0),
            VirtualKeyCode::Up => set_action_move(&mut gs.ecs, avatar_id, 0, -1),
            VirtualKeyCode::Down => set_action_move(&mut gs.ecs, avatar_id, 0, 1),
            VirtualKeyCode::Numpad7 => set_action_move(&mut gs.ecs, avatar_id, -1, -1),
            VirtualKeyCode::Numpad8 => set_action_move(&mut gs.ecs, avatar_id, 0, -1),
            VirtualKeyCode::Numpad9 => set_action_move(&mut gs.ecs, avatar_id, 1, -1),
            VirtualKeyCode::Numpad4 => set_action_move(&mut gs.ecs, avatar_id, -1, 0),
            VirtualKeyCode::Numpad5 => set_action_move(&mut gs.ecs, avatar_id, 0, 0),
            VirtualKeyCode::Numpad6 => set_action_move(&mut gs.ecs, avatar_id, 1, 0),
            VirtualKeyCode::Numpad1 => set_action_move(&mut gs.ecs, avatar_id, -1, 1),
            VirtualKeyCode::Numpad2 => set_action_move(&mut gs.ecs, avatar_id, 0, 1),
            VirtualKeyCode::Numpad3 => set_action_move(&mut gs.ecs, avatar_id, 1, 1),
            VirtualKeyCode::I => {
                actions::set_current_action(&mut gs.ecs, avatar_id, Action::Interact);
            }
            // VirtualKeyCode::W => gs.camera.y -= 1,
            // VirtualKeyCode::A => gs.camera.x -= 1,
            // VirtualKeyCode::D => gs.camera.x += 1,
            // VirtualKeyCode::S => gs.camera.y += 1,
            _ => {}
        },
    }
}
