use crate::cfg::Cfg;
use crate::game_log::GameLog;
use crate::models::Player;
use crate::view::cockpit_window::CockpitWindowState;
use crate::view::game_window::GameWindowState;
use crate::view::window::{Window, WindowManage};
use crate::{actions, ai, health, ship, view, visibility_system};
use hecs::World;
use rand::prelude::StdRng;
use rand::SeedableRng;
use rltk::BTerm as Rltk;

pub struct State {
    pub cfg: Cfg,
    pub ecs: World,
    // pub window: Window,
    pub window_manage: WindowManage,
    pub player: Player,
    // pub cockpit_window: CockpitWindowState,
    // pub shoot_window_state: ShootWindowState,
    pub logs: GameLog,
    pub rng: StdRng,
}

impl State {
    pub fn new(cfg: Cfg) -> Self {
        let mut world = World::new();
        let player_id = world.reserve_entity();

        let mut state = State {
            cfg,
            ecs: world,
            // window: Window::MainMenu,
            window_manage: WindowManage::default(),
            player: Player::new(player_id),
            // cockpit_window: Default::default(),
            // shoot_window_state: Default::default(),
            logs: Default::default(),
            rng: SeedableRng::seed_from_u64(0),
        };
        state.clear();
        state
    }

    pub fn clear(&mut self) {
        let mut world = World::new();
        let player_id = world.reserve_entity();
        self.ecs = world;
        self.player = Player::new(player_id);
        self.logs = Default::default();
    }

    pub fn run_game_loop_systems(&mut self) {
        visibility_system::run(&self.ecs);
        ai::run_ai_mob_system(&mut self.ecs, self.player.get_avatar_id());
        actions::run_available_actions_system(&mut self.ecs);
        actions::run_actions_system(
            &mut self.ecs,
            &mut self.window_manage,
            &mut self.logs,
            &mut self.rng,
            self.player.get_avatar_id(),
        );
        ship::systems::run(&mut self.ecs, &mut self.logs);
        health::run_health_system(&mut self.ecs, &mut self.logs);
    }
}

impl rltk::GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.run_game_loop_systems();

        match self.window_manage.get_window() {
            Window::World => view::game_window::run_window(self, ctx),
            Window::Cockpit { cockpit_id } => {
                view::cockpit_window::draw_cockpit(self, ctx, cockpit_id);
            }
            Window::MainMenu => {
                view::main_menu_window::run(self, ctx);
            }
            Window::Lose => {
                todo!();
            }
        }
    }
}
