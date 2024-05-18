use crate::{Location, Sector};
use hecs::World;
use log;

pub fn update_bodies_list(ecs: &World) {
    for (obj_id, location) in &mut ecs.query::<&Location>() {
        match location {
            Location::Sector { sector_id, .. } => {
                let mut sector = ecs.get::<&mut Sector>(*sector_id).unwrap();
                log::debug!(
                    "adding {:?} at {:?} to sector {:?}",
                    obj_id,
                    location,
                    sector_id
                );
                sector.bodies.push(obj_id)
            }
            _ => log::debug!("skipping {:?} at {:?}", obj_id, location),
        }
    }
}
