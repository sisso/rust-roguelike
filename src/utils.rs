use crate::models::{ObjectsType, Position};
use specs::prelude::*;

pub fn find_objects_at<'a>(
    entities: &Entities<'a>,
    objects: &ReadStorage<'a, ObjectsType>,
    positions: &ReadStorage<'a, Position>,
    pos: &Position,
) -> Vec<(Entity, ObjectsType)> {
    let mut result = vec![];
    for (e, o, p) in (entities, objects, positions).join() {
        if p == pos {
            result.push((e.clone(), o.clone()));
        }
    }
    result
}
