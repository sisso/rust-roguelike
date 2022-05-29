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

    // pub fn find_gmap_mut<'a, 'b>(
    //     storage: &'a mut WriteStorage<'b, GridRef>,
    //     id: Entity,
    // ) -> Option<&'a mut GMap> {
    //     let mut current_id = id;
    //
    //     loop {
    //         let grid_ref = storage.get_mut(id);
    //         match grid_ref {
    //             Some(GridRef::GMap(gmap)) => return Some(gmap),
    //             Some(GridRef::Ref(ref_id)) => current_id = *ref_id,
    //             None => return None,
    //         }
    //     }
    // }

    pub fn replace<'a, 'b>(
        storage: &'a mut WriteStorage<'b, GridRef>,
        id: Entity,
        new_ref: GridRef,
    ) -> Option<GridRef> {
        let previous = storage.remove(id);
        storage.insert(id, new_ref).unwrap();
        previous
    }

    pub fn get_gmap(&self) -> Option<&GMap> {
        match self {
            GridRef::GMap(g) => Some(g),
            _ => None,
        }
    }
}
