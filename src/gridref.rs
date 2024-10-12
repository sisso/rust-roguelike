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
    // for give grid_id, search recursive until find the id of current gmap it belongs
    pub fn resolve_references(world: &World, id: GridId) -> Option<GridId> {
        let value = world.get::<&GridRef>(id).ok()?;
        match &*value {
            GridRef::GMap(_) => Some(id),
            GridRef::Ref(ref_id) => GridRef::resolve_references(world, *ref_id),
        }
    }

    pub fn find_area(world: &World, grid_id: GridId) -> Option<Ref<Area>> {
        let real_grid_id = Self::resolve_references(world, grid_id)?;
        let value = world.get::<&GridRef>(real_grid_id).ok()?;
        match &*value {
            GridRef::GMap(_) => Some(Ref::map(value, |i| i.get_gmap().unwrap())),
            _ => None,
        }
    }

    pub fn find_gmap_mut(world: &World, grid_id: Entity) -> Option<RefMut<Area>> {
        let real_grid_id = Self::resolve_references(world, grid_id)?;
        let value = world.get::<&mut GridRef>(real_grid_id).ok()?;
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
