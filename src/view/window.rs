use crate::view::cockpit_window::CockpitWindowState;
use crate::view::game_window::ShootWindowState;
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
    WorldShoot,
}

#[derive(Debug, Default)]
pub struct WindowManage {
    window: Window,
    pub cockpit_window: CockpitWindowState,
    pub shoot_state: ShootWindowState,
}

impl WindowManage {
    pub fn get_window(&self) -> Window {
        self.window
    }

    pub fn set_window(&mut self, window: Window) {
        self.window = window;
    }
}
