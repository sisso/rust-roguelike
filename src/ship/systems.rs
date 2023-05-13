use crate::commons::grid::Coord;
use crate::commons::recti;
use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::ship::Command;
use crate::{Location, Position, Sector, SectorBody, Ship, Surface, SurfaceZone, P2};
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
        ReadStorage<'a, Surface>,
    );

    fn run(
        &mut self,
        (entities, mut ships, mut locations, sectors, bodies, mut grids, mut positions, surfaces): Self::SystemData,
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
                    do_ship_landing(
                        &entities,
                        &mut locations,
                        &mut grids,
                        &mut positions,
                        ship_id,
                        ship,
                        target_id,
                        place_coords,
                    );
                }

                Command::Launch => {
                    // find ship grid
                    let grid_id = GridRef::find_gmap_entity_mut(&mut grids, ship_id).unwrap();

                    // find what body we are landed
                    let surface_body_id =
                        Surface::find_surface_body(&entities, &surfaces, grid_id).unwrap();

                    // update ship command to idle
                    ship.current_command = Command::Idle;

                    // extract ship grid
                    let (grid, previous_coords) =
                        GridRef::extract(&mut grids, grid_id, ship_id).unwrap();
                    (&mut grids).insert(ship_id, GridRef::GMap(grid)).unwrap();

                    // move objects inside ship grid back to ship
                    // TODO: move only objects in top of the removed grid
                    move_all_objects(
                        &entities,
                        &mut positions,
                        grid_id,
                        ship_id,
                        &previous_coords.inverse(),
                    );

                    // change ship state
                    locations
                        .insert(
                            ship_id,
                            Location::Orbit {
                                target_id: surface_body_id,
                            },
                        )
                        .expect("fail to insert orbit");
                }
                _ => {}
            }
        }
    }
}

fn do_ship_landing(
    entities: &Entities,
    locations: &mut WriteStorage<Location>,
    mut grids: &mut WriteStorage<GridRef>,
    positions: &mut WriteStorage<Position>,
    ship_id: Entity,
    ship: &mut Ship,
    target_id: Entity,
    place_coords: P2,
) {
    // update ship command to idle
    ship.current_command = Command::Idle;

    // replace ship reference to new target
    let ship_gmap = match GridRef::replace(&mut grids, ship_id, GridRef::Ref(target_id)) {
        Some(GridRef::GMap(gmap)) => gmap,
        _ => panic!("unexpected grid_ref for ship_id {:?}", ship_id),
    };

    // get landing zone
    let target_gmap = match (&mut grids).get_mut(target_id) {
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
    move_all_objects(entities, positions, ship_id, target_id, &ship_pos);

    debug!(
        "moving ship map {:?} into surface {:?} on {:?}",
        ship_id.id(),
        target_id.id(),
        ship_pos
    );

    target_gmap.merge(ship_gmap, &ship_pos);

    // update ship location
    locations
        .insert(
            ship_id,
            Location::BodySurfacePlace {
                body_id: target_id,
                place_coords: place_coords,
                grid_pos: ship_pos,
            },
        )
        .expect("fail to update location");
}

fn move_all_objects(
    entities: &Entities,
    positions: &mut WriteStorage<Position>,
    from_grid_id: Entity,
    to_grid_id: Entity,
    to_pos: &V2I,
) {
    for (e, p) in (entities, positions).join() {
        if p.grid_id == from_grid_id {
            let global = recti::to_global(&to_pos, &p.point);
            debug!(
                "update object {} from {:?} grid {:?} to {:?} at grid {:?}",
                e.id(),
                p.point,
                from_grid_id,
                global,
                to_grid_id
            );
            p.grid_id = to_grid_id;
            p.point = global;
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
