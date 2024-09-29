use crate::view::cockpit_window::CockpitWindowState;
use crate::view::game_window::GameWindowState;
use hecs::Entity;

#[derive(Copy, Clone, Debug, Default)]
pub enum Window {
    #[default]
    MainMenu,
    Lose,
    World,
    Cockpit {
        cockpit_id: Entity,
    },
}

#[derive(Debug, Default)]
pub struct WindowManage {
    window: Window,
    pub cockpit_window: CockpitWindowState,
    pub game_state: GameWindowState,
}

impl WindowManage {
    pub fn get_window(&self) -> Window {
        self.window
    }

    pub fn set_window(&mut self, window: Window) {
        self.window = window;
        self.cockpit_window = CockpitWindowState::default();
        self.game_state = Default::default();
    }
}
