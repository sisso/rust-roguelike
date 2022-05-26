use crate::commons::grid::Coord;
use crate::gmap::GMap;
use crate::ship::Command;
use crate::{commons, Location, Position, Sector, SectorBody, Ship, Surface};
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
        WriteStorage<'a, GMap>,
        WriteStorage<'a, Position>,
    );

    fn run(
        &mut self,
        (entities, mut ships, mut locations, sectors, bodies, mut gmaps, mut positions): Self::SystemData,
    ) {
        for (ship_entity, ship) in (&entities, &mut ships).join() {
            // update calm down
            if ship.move_calm_down > 0 {
                ship.move_calm_down -= 1;
                debug!("calm down {:?}", ship.move_calm_down);
                continue;
            }

            // execute command
            match ship.current_command {
                Command::FlyTo { target_id } => {
                    do_ship_fly(&mut locations, ship_entity, ship, target_id)
                }

                Command::Land {
                    pos: surf_pos,
                    target_id,
                } => {
                    // update ship command to idle
                    ship.current_command = Command::Idle;

                    // get landing zone
                    let ship_gmap = (&gmaps)
                        .get(ship_entity)
                        .expect("ship map not found")
                        .clone();

                    let target_gmap = (&mut gmaps)
                        .get_mut(target_id)
                        .expect("gmap for landing target id not found");

                    todo!()

                    // bake ship grid into new zone
                    // let target_center_pos =
                    //     Coord::new(target_gmap.width / 2, target_gmap.height / 2);
                    // let ship_pos = Coord::new(
                    //     target_center_pos.x - ship_gmap.width / 2,
                    //     target_center_pos.y - ship_gmap.height / 2,
                    // );
                    //
                    // debug!(
                    //     "copying ship map {:?} into surface {:?} on {:?}",
                    //     ship_entity.id(),
                    //     target_id.id(),
                    //     ship_pos
                    // );
                    // for x in 0..ship_gmap.width {
                    //     for y in 0..ship_gmap.height {
                    //         let coords = Coord::new(x, y);
                    //         let global = commons::recti::to_global(&ship_pos, &coords);
                    //         let sindex =
                    //             commons::grid::coords_to_index(ship_gmap.width, &coords) as usize;
                    //         let tindex =
                    //             commons::grid::coords_to_index(target_gmap.width, &global) as usize;
                    //
                    //         if ship_gmap.cells[sindex].tile.is_nothing() {
                    //             continue;
                    //         }
                    //
                    //         target_gmap.cells[tindex] = ship_gmap.cells[sindex].clone();
                    //     }
                    // }
                    //
                    // // move objects into new zone
                    // for (e, p) in (&entities, &mut positions).join() {
                    //     if p.grid_id == ship_entity {
                    //         let global = commons::recti::to_global(&ship_pos, &p.point);
                    //         debug!("moving {} from {:?} to {:?}", e.id(), p.point, global);
                    //         p.grid_id = target_id;
                    //         p.point = global;
                    //     }
                    // }
                    //
                    // // update ship location
                    // (&mut locations).insert(
                    //     ship_entity,
                    //     Location::BodySurfacePlace {
                    //         body_id: target_id,
                    //         place_coords: ship_pos,
                    //         surface_pos: surf_pos,
                    //     },
                    // );
                }

                Command::Launch => {
                    // copy grid back to entity
                    // move objects to previous grid
                    warn!("landing is not implemented");
                }
                _ => {}
            }
        }
    }
}

fn do_ship_fly(
    locations: &mut WriteStorage<Location>,
    ship_entity: Entity,
    ship: &mut Ship,
    target_id: Entity,
) {
    ship.move_calm_down = super::FLY_SLEEP_TIME;

    // update position
    let target_pos = {
        match locations.get(target_id) {
            Some(Location::Sector { pos, .. }) => pos.clone(),
            other => {
                warn!("invalid location for flyto target: {:?}", other);
                return;
            }
        }
    };

    match locations.get_mut(ship_entity) {
        Some(Location::Sector { pos, .. }) if *pos == target_pos => {
            info!("ship arrival, entering in orbit");
            ship.current_command = Command::Idle;
            locations
                .insert(
                    ship_entity,
                    Location::Orbit {
                        target_id: target_id,
                    },
                )
                .unwrap();
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

#[cfg(test)]
mod test {
    #[test]
    fn test_landing() {}
}
