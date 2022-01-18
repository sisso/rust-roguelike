use crate::{Location, Sector};
use log::debug;
use specs::prelude::*;

pub fn update_bodies_list(ecs: &mut World) {
    let objects = ecs.entities();
    let locations = ecs.read_storage::<Location>();
    let mut sectors = ecs.write_storage::<Sector>();

    for (obj_id, location) in (&objects, &locations).join() {
        match location {
            Location::Sector {
                sector_id: sector_id,
                ..
            } => {
                let sector = sectors.get_mut(*sector_id).unwrap();
                debug!(
                    "adding {:?} at {:?} to sector {:?}",
                    obj_id, location, sector_id
                );
                sector.bodies.push(obj_id)
            }
            _ => {
                debug!("skipping {:?} at {:?}", obj_id, location);
            }
        }
    }
}
