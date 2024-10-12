use crate::commons::grid::{BaseGrid, Coord};
use crate::commons::recti;
use crate::commons::v2i::V2I;
use crate::game_log::{GameLog, Msg};
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

pub fn run(world: &mut World, game_log: &mut GameLog) {
    let mut changes: Vec<Box<dyn FnOnce(&mut World, &mut GameLog)>> = vec![];

    for (ship_id, ship) in world.query_mut::<&mut Ship>() {
        // update calm down
        if ship.move_calm_down > 0 {
            ship.move_calm_down -= 1;
            debug!("calm down {:?}", ship.move_calm_down);
            continue;
        }

        match ship.current_command {
            Command::FlyTo { target_id } => changes.push(Box::new(
                move |world: &mut World, game_log: &mut GameLog| {
                    do_ship_fly(world, ship_id, target_id);
                },
            )),

            Command::Land {
                target_id,
                place_coords,
            } => {
                changes.push(Box::new(
                    move |world: &mut World, game_log: &mut GameLog| {
                        do_ship_landing(world, ship_id, target_id, place_coords, Some(game_log));
                    },
                ));
            }

            Command::Launch => {
                changes.push(Box::new(
                    move |world: &mut World, game_log: &mut GameLog| {
                        do_ship_launching(world, ship_id, game_log);
                    },
                ));
            }
            _ => {}
        }
    }

    for change in changes {
        change(world, game_log);
    }
}

fn do_ship_launching(world: &mut World, ship_id: Entity, game_log: &mut GameLog) {
    // find ship grid
    let grid_id = GridRef::resolve_references(world, ship_id).unwrap();

    // find what body we are landed
    let surface_body_id = Surface::find_surface_body(&world, grid_id).unwrap();

    // update ship command to idle
    set_ship_command(world, ship_id, Command::Idle);

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

    game_log.push(Msg::ShipLaunch);
}

fn set_ship_command(world: &mut World, ship_id: Entity, command: Command) {
    world
        .query_one_mut::<&mut Ship>(ship_id)
        .expect("ship not found")
        .current_command = command;
}

/// Landing:
/// 1. find landing gmap and landing position
/// 2. replace ship GridRef to a reference to the landing position entity
/// 3. Move all entities Location to the new gmap and update its position
/// 4. take ship grid and add as a new layer to the landing position
///
/// Why do we move grid into target grid? Is easy to manage a single data structure NGrid that for
/// each cell recursive find the parent so we can check what area belong that grid
pub fn do_ship_landing(
    world: &mut World,
    ship_id: Entity,
    target_id: Entity,
    place_coords: P2,
    game_log: Option<&mut GameLog>,
) {
    let ship_pos = {
        // update ship command to idle
        set_ship_command(world, ship_id, Command::Idle);

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
        ship_pos
    };

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

    if let Some(game_log) = game_log {
        game_log.push(Msg::ShipLand);
    }
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
    let insert_at = {
        let mut ship = world.get::<&mut Ship>(ship_id).unwrap();
        ship.move_calm_down = super::FLY_SLEEP_TIME;

        // update position
        let maybe_target_pos =
            world
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

        insert_at
    };

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
