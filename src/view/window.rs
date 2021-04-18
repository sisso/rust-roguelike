use specs::prelude::*;
use specs_derive::*;

#[derive(Component, Copy, Clone)]
pub enum Window {
    World,
    Cockpit,
}
