use crate::commons::grid::Coord;
use crate::Area;
use specs::prelude::*;
use specs_derive::*;

/// Entity that hold the gmap of the following object. To find the real grid the references must be
/// followed until a GMap is found and them search on what layer index belong to this object
#[derive(Component, Debug, Clone)]
pub enum GridRef {
    Ref(Entity),
    GMap(Area),
}

impl GridRef {
    pub fn find_area<'a>(storage: &'a ReadStorage<'a, GridRef>, id: Entity) -> Option<&'a Area> {
        storage.get(id).and_then(|i| match i {
            GridRef::GMap(gmap) => Some(gmap),
            GridRef::Ref(ref_id) => GridRef::find_area(storage, *ref_id),
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
    ) -> Option<&'a mut Area> {
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
    ) -> Option<(Area, Coord)> {
        match storage.get_mut(from_grid_id)? {
            GridRef::GMap(gmap) => gmap.remove_layer(layer_id),
            _ => None,
        }
    }

    pub fn get_gmap(&self) -> Option<&Area> {
        match self {
            GridRef::GMap(g) => Some(g),
            _ => None,
        }
    }
}
