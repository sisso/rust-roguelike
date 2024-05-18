use crate::models::{ObjectsType, Position};
use hecs::{Entity, World};

pub fn find_objects_at<'a>(world: &World, pos: &Position) -> Vec<(Entity, ObjectsType)> {
    let mut result = vec![];
    for (e, (o, p)) in world.query::<(&ObjectsType, &Position)>().iter() {
        if p == pos {
            result.push((e.clone(), o.clone()));
        }
    }
    result
}
