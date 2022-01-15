use crate::{Player, Position, Ship, State};
use specs::prelude::*;

#[derive(Clone, Debug)]
pub enum Command {
    Status,
    Land,
    FlyTo,
    Launch,
}

pub fn list_commands(ecs: &World) -> Vec<Command> {
    let player = ecs.fetch::<Player>();
    let positions = ecs.read_storage::<Position>();
    let pos = positions
        .get(player.get_avatar())
        .expect("player has no position");
    let ships = ecs.read_storage::<Ship>();
    let ship = ships.get(pos.grid_id).expect("player is not in a ship");

    vec![Command::Status, Command::Land]
}
