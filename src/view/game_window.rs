use crate::actions::Action;
use crate::commons::grid::Coord;
use crate::commons::recti::RectI;
use crate::commons::v2i::V2I;
use crate::models::Position;
use crate::state::State;
use crate::view::camera::Camera;
use crate::{actions, ai, cfg, utils, view};
use hecs::{Entity, World};
use rltk::{BTerm, Rltk, VirtualKeyCode};

#[derive(Clone, Debug, Default)]
pub enum SubWindow {
    #[default]
    Normal,
    Fire {
        target: Coord,
    },
    Info {
        target: Coord,
    },
}

#[derive(Debug, Default)]
pub struct GameWindowState {
    pub sub_window: SubWindow,
}

pub fn run_window(state: &mut State, ctx: &mut Rltk) {
    let left_colum_width = 10;
    let bottom_bar_height = 9;

    let game_area = RectI::new(
        left_colum_width,
        0,
        cfg::SCREEN_W - left_colum_width,
        cfg::SCREEN_H - bottom_bar_height,
    );
    match &state.window_manage.game_state.sub_window {
        SubWindow::Normal => {
            process_input(state, ctx);
        }
        SubWindow::Info { target } => {
            process_info_input(state, ctx);
        }
        _ => {}
    }

    view::draw_map_and_objects(state, ctx, game_area);
    view::draw_gui(
        state,
        ctx,
        RectI::new(0, cfg::SCREEN_H - 10, cfg::SCREEN_W, bottom_bar_height),
    );

    match &state.window_manage.game_state.sub_window {
        SubWindow::Info { target } => {
            draw_info_at(state, ctx, target.clone(), game_area);
        }
        _ => {}
    }
}

fn draw_info_at(state: &mut State, ctx: &mut Rltk, pos: Coord, rect: RectI) {
    let player_pos = utils::get_position(&state.ecs, state.player.get_avatar_id()).unwrap();
    // TODO: add camera to state
    let camera = Camera::from_center(player_pos, rect);
    let marker_pos = camera.world_to_screen(pos);
    ctx.print_color(marker_pos.x, marker_pos.y, rltk::GRAY, rltk::BLACK, "X");
}

fn process_info_input(gs: &mut State, ctx: &mut Rltk) {
    if let Some(dir) = view::read_key_direction(ctx) {
        match &mut gs.window_manage.game_state.sub_window {
            SubWindow::Info { target: position } => {
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
    }

    match ctx.key {
        Some(VirtualKeyCode::I) => {
            actions::set_current_action(&mut gs.ecs, avatar_id, Action::Interact);
        }
        Some(VirtualKeyCode::X) => {
            let player_pos = utils::get_position(&gs.ecs, avatar_id).unwrap();
            log::info!("switching game window to info");
            gs.window_manage.game_state.sub_window = SubWindow::Info {
                target: player_pos.point,
            };
        }
        Some(VirtualKeyCode::F) => {
            // actions::set_current_action(&mut gs.ecs, avatar_id, Action::SearchToShoot);
        }
        // VirtualKeyCode::W => gs.camera.y -= 1,
        // VirtualKeyCode::A => gs.camera.x -= 1,
        // VirtualKeyCode::D => gs.camera.x += 1,
        // VirtualKeyCode::S => gs.camera.y += 1,
        _ => {}
    }
}
