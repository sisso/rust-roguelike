use crate::models::{ObjectsType, Position};
use specs::prelude::*;

pub fn find_objects_at<'a>(
    entities: &Entities<'a>,
    objects: &ReadStorage<'a, ObjectsType>,
    positions: &ReadStorage<'a, Position>,
    x: i32,
    y: i32,
) -> Vec<(Entity, ObjectsType)> {
    let mut result = vec![];
    for (e, o, p) in (entities, objects, positions).join() {
        let p = p.point;
        if p.x == x && p.y == y {
            result.push((e.clone(), o.clone()));
        }
    }
    result
}
