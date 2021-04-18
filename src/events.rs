use specs::prelude::*;
use specs_derive::*;

#[derive(Component)]
pub struct EntitiesEvents {
    pub events: Vec<Event>,
}

pub enum Event {}
