use crate::cfg::Cfg;
use crate::game_log::GameLog;
use crate::models::Player;
use crate::view;
use crate::view::cockpit_window::CockpitWindowState;
use crate::view::window::Window;
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
                crate::run_systems(self, ctx);
                view::draw_world(self, ctx);
            }

            Window::Cockpit { cockpit_id } => {
                crate::run_systems(self, ctx);
                view::draw_cockpit(self, ctx, cockpit_id);
            }
        }
    }
}
