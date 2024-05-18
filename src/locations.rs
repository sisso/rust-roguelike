use crate::{Location, P2};
use hecs::{Entity, World};

pub fn resolve_sector_pos(world: &World, entity: Entity) -> Option<(P2, Entity)> {
    let value = world.get::<&Location>(entity).ok()?;
    match &*value {
        Location::Sector { pos, sector_id } => Some((pos.clone(), *sector_id)),
        Location::Orbit { target_id } => resolve_sector_pos(world, *target_id),
        _ => None,
    }
}
