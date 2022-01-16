use crate::{Location, P2};
use specs::{Entity, ReadStorage};

pub fn resolve_sector_pos(
    locations: &ReadStorage<Location>,
    entity: Entity,
) -> Option<(P2, Entity)> {
    match locations.get(entity) {
        Some(Location::Sector { pos, sector_id }) => Some((pos.clone(), *sector_id)),
        Some(Location::Orbit { target_id }) => resolve_sector_pos(locations, *target_id),
        _ => None,
    }
}
