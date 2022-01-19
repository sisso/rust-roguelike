pub mod systems;

use crate::models::Player;
use crate::P2;
use specs::prelude::*;
use specs_derive::*;

pub const FLY_SLEEP_TIME: u32 = 60;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Idle,
    FlyTo { target_id: Entity },
    Land { target_id: Entity, pos: P2 },
    Launch,
}

#[derive(Component, Debug, Clone)]
pub struct Ship {
    pub current_command: Command,
    pub move_calm_down: u32,
}

pub fn enter_cockpit(_avatar: &mut Player) {
    // change avatar state to be on control o ship from the
    // cockpit
}
