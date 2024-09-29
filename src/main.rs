extern crate core;

use state::State;

use crate::area::Area;
use crate::cfg::Cfg;
use crate::models::*;
use crate::ship::Ship;

pub mod actions;
mod ai;
pub mod area;
pub mod cfg;
mod combat;
pub mod commons;
pub mod events;
mod game_log;
pub mod gridref;
mod health;
pub mod loader;
pub mod locations;
mod mob;
pub mod models;
pub mod sectors;
pub mod ship;
pub mod state;
pub mod utils;
pub mod view;
mod visibility;
pub mod visibility_system;

fn main() -> rltk::BError {
    // setup
    use rltk::RltkBuilder;

    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    prepare_text_pallet();

    let context = RltkBuilder::simple(cfg::SCREEN_W, cfg::SCREEN_H)
        .unwrap()
        .with_title("Space RL")
        .build()?;

    // initialize
    let cfg = Cfg::new();
    let state = State::new(cfg);
    rltk::main_loop(context, state)
}

fn prepare_text_pallet() {
    rltk::register_palette_color("red", rltk::RED);
    rltk::register_palette_color("gray", rltk::GRAY);
}

#[cfg(test)]
mod test {
    use crate::cfg::Cfg;
    use crate::state::State;

    #[test]
    fn test_acceptance() {
        let mut s = new_state_basic_scenery();
    }

    pub fn new_state_basic_scenery() {
        let mut state = new_state();
    }

    pub fn new_state() -> State {
        State::new(Cfg::new())
    }
}
