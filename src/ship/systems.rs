use crate::commons::grid::Coord;
use crate::commons::recti;
use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::ship::Command;
use crate::{Location, Position, Sector, SectorBody, Ship, Surface, SurfaceZone, P2};
use hecs::{Entity, Ref, World};
use log::{debug, info, warn};

fn clamp(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn run(world: &mut World) {
    let mut update_ship_fly_to = vec![];

    for (ship_id, ship) in &mut world.query::<&mut Ship>() {
        // update calm down
        if ship.move_calm_down > 0 {
            ship.move_calm_down -= 1;
            debug!("calm down {:?}", ship.move_calm_down);
            continue;
        }

        match ship.current_command {
            Command::FlyTo { target_id } => {
                // do_ship_fly(&mut locations, ship_id, ship, target_id)}
                update_ship_fly_to.push((ship_id, target_id));
            }

            Command::Land {
                target_id,
                place_coords,
            } => {
                do_ship_landing(world, ship_id, ship, target_id, place_coords);
            }

            Command::Launch => {
                // find ship grid
                let grid_id = GridRef::find_gmap_entity(world, ship_id).unwrap();

                // find what body we are landed
                let surface_body_id = Surface::find_surface_body(&world, grid_id).unwrap();

                // update ship command to idle
                ship.current_command = Command::Idle;

                // extract ship grid
                let (grid, previous_coords) =
                    GridRef::extract(&world, grid_id, ship_id).expect("fail to extract grid");
                world
                    .insert_one(ship_id, GridRef::GMap(grid))
                    .expect("fail to insert gmap");

                // move objects inside ship grid back to ship
                // TODO: move only objects in top of the removed grid
                move_all_objects(world, grid_id, ship_id, previous_coords.inverse());

                // change ship state
                world
                    .insert_one(
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

// impl<'a> System<'a> for FlyToSystem {
//     type SystemData = (
//         Entities<'a>,
//         WriteStorage<'a, Ship>,
//         WriteStorage<'a, Location>,
//         ReadStorage<'a, Sector>,
//         ReadStorage<'a, SectorBody>,
//         WriteStorage<'a, GridRef>,
//         WriteStorage<'a, Position>,
//         ReadStorage<'a, Surface>,
//     );
//
//     fn run(
//         &mut self,
//         (entities, mut ships, mut locations, sectors, bodies, mut grids, mut positions, surfaces): Self::SystemData,
//     ) {
//         for (ship_id, ship) in (&entities, &mut ships).join() {
//             // update calm down
//             if ship.move_calm_down > 0 {
//                 ship.move_calm_down -= 1;
//                 debug!("calm down {:?}", ship.move_calm_down);
//                 continue;
//             }
//
//             // execute command
//             match ship.current_command {
//                 Command::FlyTo { target_id } => {
//                     do_ship_fly(&mut locations, ship_id, ship, target_id)
//                 }
//
//                 Command::Land {
//                     target_id,
//                     place_coords,
//                 } => {
//                     do_ship_landing(
//                         &entities,
//                         &mut locations,
//                         &mut grids,
//                         &mut positions,
//                         ship_id,
//                         ship,
//                         target_id,
//                         place_coords,
//                     );
//                 }
//
//                 Command::Launch => {
//                     // find ship grid
//                     let grid_id = GridRef::find_gmap_entity(&mut grids, ship_id).unwrap();
//
//                     // find what body we are landed
//                     let surface_body_id =
//                         Surface::find_surface_body(&entities, &surfaces, grid_id).unwrap();
//
//                     // update ship command to idle
//                     ship.current_command = Command::Idle;
//
//                     // extract ship grid
//                     let (grid, previous_coords) =
//                         GridRef::extract(&mut grids, grid_id, ship_id).unwrap();
//                     (&mut grids).insert(ship_id, GridRef::GMap(grid)).unwrap();
//
//                     // move objects inside ship grid back to ship
//                     // TODO: move only objects in top of the removed grid
//                     move_all_objects(
//                         &entities,
//                         &mut positions,
//                         grid_id,
//                         ship_id,
//                         &previous_coords.inverse(),
//                     );
//
//                     // change ship state
//                     locations
//                         .insert(
//                             ship_id,
//                             Location::Orbit {
//                                 target_id: surface_body_id,
//                             },
//                         )
//                         .expect("fail to insert orbit");
//                 }
//                 _ => {}
//             }
//         }
//     }
// }
//
/// Landing:
/// 1. find landing gmap and landing position
/// 2. replace ship GridRef to a reference to the landing position entity
/// 3. Move all entities Location to the new gmap and update its position
/// 4. take ship grid and add as a new layer to the landing position
///
/// Why do we move grid into target grid? Is easy to manage a single data structure NGrid that for
/// each cell recursive find the parent so we can check what area belong that grid
fn do_ship_landing(
    world: &mut World,
    ship_id: Entity,
    ship: &mut Ship,
    target_id: Entity,
    place_coords: P2,
) {
    // update ship command to idle
    ship.current_command = Command::Idle;

    // replace ship reference to new target
    let ship_gmap = match GridRef::replace(world, ship_id, GridRef::Ref(target_id)) {
        Some(GridRef::GMap(gmap)) => gmap,
        _ => panic!("unexpected grid_ref for ship_id {:?}", ship_id),
    };

    // get landing zone
    let mut result = world
        .get::<&mut GridRef>(target_id)
        .expect("fail to get target grid");

    let target_gmap = match &mut *result {
        GridRef::GMap(gmap) => gmap,
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
    move_all_objects(world, ship_id, target_id, ship_pos);

    debug!(
        "moving ship map {:?} into surface {:?} on {:?}",
        ship_id.id(),
        target_id.id(),
        ship_pos
    );

    target_gmap.merge(ship_gmap, &ship_pos);

    // update ship location
    // convert line below into new world api
    world
        .insert_one(
            ship_id,
            Location::BodySurfacePlace {
                body_id: target_id,
                place_coords: place_coords,
                grid_pos: ship_pos,
            },
        )
        .expect("fail to update location");
}

fn move_all_objects(world: &World, from_grid_id: Entity, to_grid_id: Entity, to_pos: V2I) {
    for (e, p) in &mut world.query::<&mut Position>() {
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

fn do_ship_fly(world: &mut World, ship_id: Entity, target_id: Entity) {
    let mut ship = world.get::<&mut Ship>(ship_id).unwrap();
    ship.move_calm_down = super::FLY_SLEEP_TIME;

    // update position
    let maybe_target_pos = world
        .get::<&Location>(target_id)
        .ok()
        .and_then(|value| match &*value {
            Location::Sector { pos, .. } => Some(pos.clone()),
            _ => None,
        });

    let target_pos = match maybe_target_pos {
        Some(value) => value,
        _ => {
            warn!("invalid location for flyto target");
            return;
        }
    };

    let mut insert_at = None;

    let location = world.get::<&mut Location>(ship_id);
    match location {
        Ok(mut location) => match &mut *location {
            Location::Sector { pos, .. } if *pos == target_pos => {
                info!("ship arrival, entering in orbit");
                ship.current_command = Command::Idle;
                insert_at = Some(Location::Orbit { target_id });
            }
            Location::Sector {
                pos,
                sector_id: _sector,
            } => {
                let delta_x = clamp(target_pos.x - pos.x, -1, 1);
                let delta_y = clamp(target_pos.y - pos.y, -1, 1);
                info!("moving {:?} by {},{}", pos, delta_x, delta_y);
                pos.x += delta_x;
                pos.y += delta_y;
            }
            other => warn!(
                "invalid location for ship with flyto command, found {:?}",
                other
            ),
        },
        _ => warn!("invalid location for ship with flyto command"),
    }

    if let Some(new_location) = insert_at {
        world
            .insert_one(ship_id, new_location)
            .expect("fail to insert new location");
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_landing() {}
}
