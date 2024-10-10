use crate::actions::{Action, EntityActions};
use crate::commons::grid::Coord;
use crate::commons::recti::RectI;
use crate::commons::v2i::V2I;
use crate::models::Position;
use crate::state::State;
use crate::view::camera::Camera;
use crate::{actions, ai, cfg, utils, view};
use hecs::{Entity, World};
use log::log;
use rltk::{BTerm, Rltk, VirtualKeyCode};

#[derive(Clone, Debug, Default)]
pub enum SubWindow {
    #[default]
    Normal,
    Fire {
        point: Coord,
    },
    Info {
        point: Coord,
    },
}

#[derive(Debug, Default)]
pub struct GameWindowState {
    pub sub_window: SubWindow,
}

pub fn run_window(state: &mut State, ctx: &mut Rltk) {
    match &state.window_manage.game_state.sub_window {
        SubWindow::Normal => {
            process_input(state, ctx);
        }
        SubWindow::Info { point: target } => {
            process_info_input(state, ctx);
        }
        _ => {}
    }

    view::draw_default(state, ctx);

    match &state.window_manage.game_state.sub_window {
        SubWindow::Info { point: target } => {
            draw_info_marker(state, ctx, *target);
        }
        _ => {}
    }
}

fn draw_info_marker(state: &mut State, ctx: &mut Rltk, point: Coord) {
    let player_pos = utils::get_position(&state.ecs, state.player.get_avatar_id()).unwrap();
    // TODO: add camera to state
    let camera = Camera::from_center(player_pos, state.screen_layout.get_main_area_rect());
    let marker_point = camera.world_to_screen(point);
    ctx.print_color(marker_point.x, marker_point.y, rltk::GRAY, rltk::BLACK, "X");
}

fn process_info_input(gs: &mut State, ctx: &mut Rltk) {
    if let Some(dir) = view::read_key_direction(ctx) {
        match &mut gs.window_manage.game_state.sub_window {
            SubWindow::Info { point: position } => {
                *position = *position + dir;
            }
            _ => {
                log::warn!("invalid game state subwindow");
            }
        }
    }

    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::X | VirtualKeyCode::Escape => {
                log::info!("switching game window to normal");
                gs.window_manage.game_state.sub_window = SubWindow::Normal;
            }
            _ => {}
        },
    }
}

fn set_action_move(ecs: &mut World, avatar_id: Entity, delta_x: i32, delta_y: i32) {
    actions::set_current_action(ecs, avatar_id, Action::Move(V2I::new(delta_x, delta_y)));
    ai::give_turn_to_ai(ecs);
}

fn process_input(gs: &mut State, ctx: &mut Rltk) {
    let avatar_id = gs.player.get_avatar_id();

    if let Some(dir) = view::read_key_direction(ctx) {
        set_action_move(&mut gs.ecs, avatar_id, dir.x, dir.y);
        return;
    }

    let available_actions = actions::get_available_actions(gs, avatar_id);

    match ctx.key {
        Some(VirtualKeyCode::I) => {
            let action = available_actions
                .into_iter()
                .find(|i| i.is_interact())
                .unwrap();
            actions::set_current_action(&mut gs.ecs, avatar_id, action);
        }
        Some(VirtualKeyCode::X) => {
            let player_pos = utils::get_position(&gs.ecs, avatar_id).unwrap();
            log::info!("switching game window to info at {:?}", player_pos);
            gs.window_manage.game_state.sub_window = SubWindow::Info {
                point: player_pos.point,
            };
        }
        Some(VirtualKeyCode::P) => {
            let action = available_actions
                .into_iter()
                .find(|i| i.is_pickup())
                .unwrap();
            actions::set_current_action(&mut gs.ecs, avatar_id, action);
        }
        // VirtualKeyCode::W => gs.camera.y -= 1,
        // VirtualKeyCode::A => gs.camera.x -= 1,
        // VirtualKeyCode::D => gs.camera.x += 1,
        // VirtualKeyCode::S => gs.camera.y += 1,
        _ => {}
    }
}
