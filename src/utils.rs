use crate::gridref::AreaRef;
use crate::models::{BasicEntity, Label, ObjKind, Position};
use hecs::{Entity, Ref, World};

pub fn get_kind(world: &World, id: Entity) -> ObjKind {
    let mut q = world.query_one::<&ObjKind>(id).unwrap();
    q.get().unwrap().clone()
}

pub fn find_objects_at(world: &World, pos: Position) -> Vec<BasicEntity> {
    let Some(area) = AreaRef::resolve_area(world, pos.grid_id) else {
        return vec![];
    };
    area.list_objects_at(pos.point).clone()
}

pub fn find_objects_at_with_label(
    ecs: &World,
    pos: Position,
) -> Vec<(Entity, ObjKind, Ref<Label>)> {
    let objects_at = find_objects_at(ecs, pos);
    let labels = find_labels(ecs, objects_at.iter().map(|i| &i.id));
    objects_at
        .into_iter()
        .zip(labels)
        .map(|i| (i.0.id, i.0.kind, i.1))
        .collect::<Vec<_>>()
}

pub fn get_position(world: &World, id: Entity) -> Option<Position> {
    world.query_one::<&Position>(id).unwrap().get().cloned()
}

pub fn find_labels<'a, 'b, T>(world: &'b World, ids: T) -> Vec<Ref<'b, Label>>
where
    T: Iterator<Item = &'a Entity>,
{
    ids.cloned()
        .map(|id| {
            world
                .get::<&Label>(id)
                .map_err(|_| format!("label not found for {:?}", id))
                .unwrap()
        })
        .collect()
}

pub fn reindex_grids_objects(world: &mut World) {
    let entities: Vec<(Entity, Position, ObjKind)> = world
        .query::<(&Position, &ObjKind)>()
        .iter()
        .map(|i| (i.0, *i.1 .0, *i.1 .1))
        .collect();

    for grid in &mut world.query::<&mut AreaRef>() {
        match grid {
            (_, AreaRef::Struct(area)) => {
                area.clear_objects();

                for (e, p, k) in &entities {
                    if area.contains_layer(p.grid_id) {
                        area.add_object(p.point, BasicEntity::new(*e, *k));
                    }
                }
            }
            _ => {}
        }
    }
}
