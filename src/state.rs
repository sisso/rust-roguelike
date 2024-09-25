use crate::cfg::Cfg;
use crate::game_log::GameLog;
use crate::models::Player;
use crate::view::cockpit_window::CockpitWindowState;
use crate::view::window::Window;
use crate::{actions, ai, health, ship, view, visibility_system};
use hecs::World;
use rltk::{BTerm as Rltk, BTerm};

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

        let mut state = State {
            cfg,
            ecs: world,
            window: Window::MainMenu,
            player: Player::new(player_id),
            cockpit_window: Default::default(),
            logs: Default::default(),
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
            &mut self.window,
            &mut self.logs,
            self.player.get_avatar_id(),
        );
        ship::systems::run(&mut self.ecs, &mut self.logs);
        health::run_health_system(&mut self.ecs, &mut self.logs);
    }
}

impl rltk::GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let window = self.window;

        match window {
            Window::World => view::game_window::run_main_window(self, ctx),

            Window::Cockpit { cockpit_id } => {
                self.run_game_loop_systems();
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
