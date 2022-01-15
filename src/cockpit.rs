use crate::{Location, Player, Position, Sector, Ship, State};
use specs::prelude::*;

#[derive(Clone, Debug)]
pub enum Command {
    Status,
    Land,
    FlyTo { target_id: Entity },
    Launch,
}

pub fn list_commands(ecs: &World) -> Vec<Command> {
    let player = ecs.fetch::<Player>();
    let positions = ecs.read_storage::<Position>();
    let ships = ecs.read_storage::<Ship>();
    let locations = ecs.read_storage::<Location>();
    let sectors = ecs.read_storage::<Sector>();

    let pos = positions
        .get(player.get_avatar())
        .expect("player has no position");
    let ship_id = pos.grid_id;
    let ship = ships.get(ship_id).expect("player is not in a ship");
    let location = locations.get(ship_id).expect("ship has no location");

    let mut commands = vec![Command::Status];

    match location {
        Location::Sector {
            sector: sector_id, ..
        } => {
            let sector = sectors.get(*sector_id).unwrap();
            for body_id in &sector.bodies {
                if body_id == ship_id {
                    continue;
                }

                commands.push(Command::FlyTo {
                    target_id: *body_id,
                });
            }
        }
        Location::Orbit { .. } => {
            commands.push(Command::Land);
        }
        Location::BodySurface { .. } => {
            commands.push(Command::Launch);
        }
        Location::BodySurfacePlace { .. } => {}
    }

    commands
}
