use crate::commons::grid::Coord;
use crate::commons::recti;
use crate::gridref::GridRef;
use crate::ship::Command;
use crate::{Location, Position, Sector, SectorBody, Ship};
use log::{debug, info, warn};
use specs::prelude::*;

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
        WriteStorage<'a, GridRef>,
        WriteStorage<'a, Position>,
    );

    fn run(
        &mut self,
        (entities, mut ships, mut locations, sectors, bodies, mut grids, mut positions): Self::SystemData,
    ) {
        for (ship_id, ship) in (&entities, &mut ships).join() {
            // update calm down
            if ship.move_calm_down > 0 {
                ship.move_calm_down -= 1;
                debug!("calm down {:?}", ship.move_calm_down);
                continue;
            }

            // execute command
            match ship.current_command {
                Command::FlyTo { target_id } => {
                    do_ship_fly(&mut locations, ship_id, ship, target_id)
                }

                Command::Land {
                    target_id,
                    place_coords,
                } => {
                    // update ship command to idle
                    ship.current_command = Command::Idle;

                    // replace ship reference to new target
                    let ship_gmap =
                        match GridRef::replace(&mut grids, ship_id, GridRef::Ref(target_id)) {
                            Some(GridRef::GMap(gmap)) => gmap,
                            _ => panic!("unexpected grid_ref for ship_id {:?}", ship_id),
                        };

                    // get landing zone
                    let gridsm = &mut grids;
                    let target_gmap = match (gridsm).get_mut(target_id) {
                        Some(GridRef::GMap(gmap)) => gmap,
                        _ => panic!("unexpected grid_ref for ship_id {:?}", ship_id),
                    };

                    // move grid layers into new map
                    let target_center_pos = Coord::new(
                        target_gmap.get_grid().get_width() / 2,
                        target_gmap.get_grid().get_height() / 2,
                    );
                    let ship_pos = Coord::new(
                        target_center_pos.x - ship_gmap.get_grid().get_width() / 2,
                        target_center_pos.y - ship_gmap.get_grid().get_height() / 2,
                    );

                    // move objects into new zone
                    for (e, p) in (&entities, &mut positions).join() {
                        if p.grid_id == ship_id {
                            let global = recti::to_global(&ship_pos, &p.point);
                            debug!(
                                "on land, update object {} from {:?} to {:?}",
                                e.id(),
                                p.point,
                                global
                            );
                            p.grid_id = target_id;
                            p.point = global;
                        }
                    }

                    debug!(
                        "moving ship map {:?} into surface {:?} on {:?}",
                        ship_id.id(),
                        target_id.id(),
                        ship_pos
                    );

                    target_gmap.merge(ship_gmap, &ship_pos);

                    // update ship location
                    (&mut locations).insert(
                        ship_id,
                        Location::BodySurfacePlace {
                            body_id: target_id,
                            place_coords: place_coords,
                            grid_pos: ship_pos,
                        },
                    );
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
            sector_id: _sector,
        }) => {
            let delta_x = clamp(target_pos.x - pos.x, -1, 1);
            let delta_y = clamp(target_pos.y - pos.y, -1, 1);
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
