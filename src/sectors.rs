use crate::{Location, Sector};
use specs::prelude::*;

pub fn update_objects_list(ecs: &mut World) {
    let objects = ecs.entities();
    let locations = ecs.read_storage::<Location>();
    let mut sectors = ecs.write_storage::<Sector>();

    for (obj_id, location) in (&objects, &locations).join() {
        match location {
            Location::Sector {
                sector: sector_id, ..
            } => {
                let sector = sectors.get_mut(*sector_id).unwrap();
                sector.bodies.push(obj_id)
            }
            _ => {}
        }
    }
}
