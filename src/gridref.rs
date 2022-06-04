use crate::commons::grid::Coord;
use crate::GMap;
use specs::prelude::*;
use specs_derive::*;

/// Entity that hold the gmap of the following object. You need to check on each layer in GMap
/// to find the correct grid
#[derive(Component, Debug, Clone)]
pub enum GridRef {
    Ref(Entity),
    GMap(GMap),
}

impl GridRef {
    pub fn find_gmap<'a>(storage: &'a ReadStorage<'a, GridRef>, id: Entity) -> Option<&'a GMap> {
        storage.get(id).and_then(|i| match i {
            GridRef::GMap(gmap) => Some(gmap),
            GridRef::Ref(ref_id) => GridRef::find_gmap(storage, *ref_id),
        })
    }

    pub fn find_gmap_entity_mut<'a, 'b>(
        storage: &'a mut WriteStorage<'b, GridRef>,
        id: Entity,
    ) -> Option<Entity> {
        let mut current_id = id;
        loop {
            let current_grid = storage.get(current_id)?;
            match current_grid {
                GridRef::Ref(id) => {
                    current_id = *id;
                }
                GridRef::GMap(_) => {
                    break;
                }
            }
        }

        Some(current_id)
    }

    pub fn find_gmap_mut<'a, 'b>(
        storage: &'a mut WriteStorage<'b, GridRef>,
        id: Entity,
    ) -> Option<&'a mut GMap> {
        let current_id = Self::find_gmap_entity_mut(storage, id)?;
        match storage.get_mut(current_id) {
            Some(GridRef::GMap(gmap)) => Some(gmap),
            _ => None,
        }
    }

    pub fn replace<'a, 'b>(
        storage: &'a mut WriteStorage<'b, GridRef>,
        id: Entity,
        new_ref: GridRef,
    ) -> Option<GridRef> {
        let previous = storage.remove(id);
        storage.insert(id, new_ref).unwrap();
        previous
    }

    pub fn extract<'a, 'b>(
        storage: &'a mut WriteStorage<'b, GridRef>,
        from_grid_id: Entity,
        layer_id: Entity,
    ) -> Option<(GMap, Coord)> {
        match storage.get_mut(from_grid_id)? {
            GridRef::GMap(gmap) => {
                gmap.remove_layer(layer_id)
            }
            _ => None,
        }
    }

    pub fn get_gmap(&self) -> Option<&GMap> {
        match self {
            GridRef::GMap(g) => Some(g),
            _ => None,
        }
    }
}
