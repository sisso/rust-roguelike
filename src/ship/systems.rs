use crate::ship::Command;
use crate::{Location, Sector, SectorBody, Ship};
use log::{debug, info, warn};
use specs::prelude::*;
use std::borrow::BorrowMut;

pub struct FlyToSystem {}

fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

impl<'a> System<'a> for FlyToSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Ship>,
        WriteStorage<'a, Location>,
        ReadStorage<'a, Sector>,
        ReadStorage<'a, SectorBody>,
    );

    fn run(&mut self, (entities, mut ships, mut locations, sectors, bodies): Self::SystemData) {
        for (e, ship) in (&entities, &mut ships).join() {
            // update calm down
            if ship.move_calm_down > 0 {
                ship.move_calm_down -= 1;
                debug!("calm down {:?}", ship.move_calm_down);
                continue;
            }

            // execute command
            match ship.current_command {
                Command::FlyTo { target_id } => {
                    ship.move_calm_down = super::FLY_SLEEP_TIME;

                    // update position
                    let target_pos = {
                        match locations.get(target_id) {
                            Some(Location::Sector { pos, .. }) => pos.clone(),
                            other => {
                                warn!("invalid location for flyto target: {:?}", other);
                                continue;
                            }
                        }
                    };

                    match locations.get_mut(e) {
                        Some(Location::Sector { pos, .. }) if *pos == target_pos => {
                            info!("ship arrival, entering in orbit");
                            ship.current_command = Command::Idle;
                            locations.insert(
                                e,
                                Location::Orbit {
                                    target_id: target_id,
                                },
                            );
                        }
                        Some(Location::Sector {
                            pos,
                            sector_id: sector,
                        }) => {
                            let mut delta_x = clamp(target_pos.x - pos.x, -1, 1);
                            let mut delta_y = clamp(target_pos.y - pos.y, -1, 1);
                            info!("moving {:?} by {},{}", pos, delta_x, delta_y);
                            pos.x += delta_x;
                            pos.y += delta_y;
                        }
                        other => {
                            warn!("invalid location for ship with flyto command: {:?}", other);
                        }
                    }
                }

                Command::Land { .. } => {}
                _ => {}
            }
        }
    }
}
