use crate::{ship, Location, Player, Position, Sector, Ship, State, P2};
use log::{info, warn};
use specs::prelude::*;

#[derive(Clone, Debug)]
pub enum Command {
    Land,
    FlyTo { target_id: Entity },
    Launch,
}

pub fn do_command(ecs: &mut World, ship_id: Entity, command: &Command) {
    match command {
        Command::FlyTo { target_id } => {
            let ship_command = ship::Command::FlyTo {
                target_id: *target_id,
            };
            info!("update ship {:?} command to {:?}", ship_id, ship_command);
            ecs.write_storage::<Ship>()
                .get_mut(ship_id)
                .unwrap()
                .current_command = ship_command;
        }
        other => warn!("unexpected command {:?}", other),
    }
}

pub fn list_commands(ecs: &World, ship_id: Entity) -> Vec<Command> {
    let locations = ecs.read_storage::<Location>();
    let sectors = ecs.read_storage::<Sector>();

    let location = locations.get(ship_id).expect("ship has no location");

    let mut commands = vec![];

    match location {
        Location::Sector {
            sector_id: sector_id,
            ..
        } => {
            let sector = sectors.get(*sector_id).unwrap();
            for body_id in &sector.bodies {
                if *body_id == ship_id {
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
