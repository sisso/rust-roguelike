use crate::models::Avatar;
use specs::prelude::*;
use specs_derive::*;

#[derive(Debug, Clone, Copy)]
pub enum ShipState {
    Space,
    Landed,
}

#[derive(Component, Debug)]
pub struct Ship {
    // state: ShipState,
}

pub fn enter_cockpit(_avatar: &mut Avatar) {
    // change avatar state to be on control o ship from the
    // cockpit
}
