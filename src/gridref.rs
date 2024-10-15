use crate::commons::grid::Coord;
use crate::models::Position;
use crate::Area;
use hecs::{Entity, Ref, RefMut, World};

pub type AreaId = Entity;

/// Entity that hold the gmap of the following object. To find the real grid the references must be
/// followed until a GMap is found and them search on what layer index belong to this object
#[derive(Debug, Clone)]
pub enum AreaRef {
    Ref(AreaId),
    Struct(Area),
}

impl AreaRef {
    // for give grid_id, search recursive until find the id of current gmap it belongs
    pub fn resolve_gmap_id(world: &World, id: AreaId) -> Option<AreaId> {
        let value = world.get::<&AreaRef>(id).ok()?;
        match &*value {
            AreaRef::Struct(_) => Some(id),
            AreaRef::Ref(ref_id) => AreaRef::resolve_gmap_id(world, *ref_id),
        }
    }

    pub fn resolve_area(world: &World, grid_id: AreaId) -> Option<Ref<Area>> {
        let real_grid_id = Self::resolve_gmap_id(world, grid_id)?;
        let value = world.get::<&AreaRef>(real_grid_id).ok()?;
        match &*value {
            AreaRef::Struct(_) => Some(Ref::map(value, |i| i.get_gmap().unwrap())),
            _ => None,
        }
    }

    pub fn resolve_area_mut(world: &World, grid_id: Entity) -> Option<RefMut<Area>> {
        let real_grid_id = Self::resolve_gmap_id(world, grid_id)?;
        let value = world.get::<&mut AreaRef>(real_grid_id).ok()?;
        match &*value {
            AreaRef::Struct(_) => Some(RefMut::map(value, |i| i.get_gmap_mut().unwrap())),
            _ => None,
        }
    }

    pub fn remove_entity(world: &World, id: Entity, pos: Position) {
        Self::resolve_area_mut(world, pos.grid_id)
            .unwrap()
            .remove_object(id, pos.point);
    }

    pub fn replace(world: &mut World, id: Entity, new_ref: AreaRef) -> Option<AreaRef> {
        let previous = world.remove_one::<AreaRef>(id).ok();
        world.insert_one(id, new_ref).unwrap();
        previous
    }

    pub fn extract(world: &World, from_grid_id: Entity, layer_id: Entity) -> Option<(Area, Coord)> {
        Self::resolve_area_mut(world, from_grid_id).and_then(|mut gmap| gmap.remove_layer(layer_id))
    }

    pub fn get_gmap(&self) -> Option<&Area> {
        match self {
            AreaRef::Struct(g) => Some(g),
            _ => None,
        }
    }

    pub fn get_gmap_mut(&mut self) -> Option<&mut Area> {
        match self {
            AreaRef::Struct(g) => Some(g),
            _ => None,
        }
    }
}
