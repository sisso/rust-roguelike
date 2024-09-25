use crate::cfg::Cfg;
use crate::game_log::GameLog;
use crate::models::Player;
use crate::view::cockpit_window::CockpitWindowState;
use crate::view::window::Window;
use crate::{actions, ai, health, mob, ship, view, visibility_system};
use hecs::World;
use log::Level::Debug;
use rltk::BTerm as Rltk;

pub struct State {
    pub cfg: Cfg,
    pub ecs: World,
    pub window: Window,
    pub player: Player,
    pub cockpit_window: CockpitWindowState,
    pub logs: GameLog,
}

impl State {
    pub fn new(cfg: Cfg) -> Self {
        let mut world = World::new();
        let player_id = world.reserve_entity();

        State {
            cfg,
            ecs: world,
            window: Window::World,
            player: Player::new(player_id),
            cockpit_window: Default::default(),
            logs: Default::default(),
        }
    }
}

impl rltk::GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let window = self.window;

        match window {
            Window::World => {
                view::player_input(self, ctx);
                run_game_loop_systems(self);
                view::draw_world(self, ctx);
            }

            Window::Cockpit { cockpit_id } => {
                run_game_loop_systems(self);
                view::draw_cockpit(self, ctx, cockpit_id);
            }
        }
    }
}

pub fn run_game_loop_systems(st: &mut State) {
    visibility_system::run(&st.ecs);
    ai::run_ai_mob_system(&mut st.ecs, st.player.get_avatar_id());
    actions::run_available_actions_system(&mut st.ecs);
    actions::run_actions_system(
        &mut st.ecs,
        &mut st.window,
        &mut st.logs,
        st.player.get_avatar_id(),
    );
    ship::systems::run(&mut st.ecs, &mut st.logs);
    health::run_health_system(&mut st.ecs, &mut st.logs);
}
