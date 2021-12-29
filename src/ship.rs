use crate::models::Player;
use specs::prelude::*;
use specs_derive::*;

#[derive(Debug, Clone, Copy)]
pub enum ShipState {
    Space,
    Landed,
}

#[derive(Component, Debug)]
pub struct Ship {
    pub state: ShipState,
}

pub fn enter_cockpit(_avatar: &mut Player) {
    // change avatar state to be on control o ship from the
    // cockpit
}
