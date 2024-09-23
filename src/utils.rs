use crate::models::{ObjectsKind, Position};
use hecs::{Entity, World};

pub fn find_objects_at<'a>(world: &World, pos: &Position) -> Vec<(Entity, ObjectsKind)> {
    let mut result = vec![];
    for (e, (o, p)) in world.query::<(&ObjectsKind, &Position)>().iter() {
        if p == pos {
            result.push((e.clone(), o.clone()));
        }
    }
    result
}

pub fn find_mobs_at<'a>(world: &World, pos: &Position) -> Vec<Entity> {
    find_objects_at(world, pos)
        .into_iter()
        .filter(|i| i.1 == ObjectsKind::Mob)
        .map(|i| i.0)
        .collect()
}
