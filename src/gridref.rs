use crate::commons::grid::Coord;
use crate::Area;
use hecs::{Entity, Ref, RefMut, World};

pub type GridId = Entity;

/// Entity that hold the gmap of the following object. To find the real grid the references must be
/// followed until a GMap is found and them search on what layer index belong to this object
#[derive(Debug, Clone)]
pub enum GridRef {
    Ref(GridId),
    GMap(Area),
}

impl GridRef {
    pub fn find_area(world: &World, id: GridId) -> Option<Ref<Area>> {
        let value = world.get::<&GridRef>(id).ok()?;
        match &*value {
            GridRef::GMap(_) => Some(Ref::map(value, |v| v.get_gmap().unwrap())),
            GridRef::Ref(ref_id) => GridRef::find_area(world, *ref_id),
        }
    }

    pub fn find_gmap_entity(world: &World, id: GridId) -> Option<GridId> {
        let mut current_id = id;
        loop {
            let current_grid = world.get::<&GridRef>(current_id).ok()?;
            match &*current_grid {
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

    pub fn find_gmap_mut(world: &World, id: Entity) -> Option<RefMut<Area>> {
        let current_id = Self::find_gmap_entity(world, id)?;
        let value = world.get::<&mut GridRef>(current_id).ok()?;
        match &*value {
            GridRef::GMap(_) => Some(RefMut::map(value, |i| i.get_gmap_mut().unwrap())),
            _ => None,
        }
    }

    pub fn replace(world: &mut World, id: Entity, new_ref: GridRef) -> Option<GridRef> {
        let previous = world.remove_one::<GridRef>(id).ok();
        world.insert_one(id, new_ref).unwrap();
        previous
    }

    pub fn extract(world: &World, from_grid_id: Entity, layer_id: Entity) -> Option<(Area, Coord)> {
        Self::find_gmap_mut(world, from_grid_id).and_then(|mut gmap| gmap.remove_layer(layer_id))
    }

    pub fn get_gmap(&self) -> Option<&Area> {
        match self {
            GridRef::GMap(g) => Some(g),
            _ => None,
        }
    }

    pub fn get_gmap_mut(&mut self) -> Option<&mut Area> {
        match self {
            GridRef::GMap(g) => Some(g),
            _ => None,
        }
    }
}
