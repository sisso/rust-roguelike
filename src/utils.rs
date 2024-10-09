use crate::health::Health;
use crate::models::{ObjectsKind, Position};
use crate::team::Team;
use hecs::{Entity, World};

pub fn find_objects_at<'a>(world: &World, pos: Position) -> Vec<(Entity, ObjectsKind)> {
    let mut result = vec![];
    for (e, (o, p)) in world.query::<(&ObjectsKind, &Position)>().iter() {
        if *p == pos {
            result.push((e.clone(), o.clone()));
        }
    }
    result
}

pub fn find_damageable_at<'a>(world: &World, pos: Position, enemies_of: Team) -> Vec<Entity> {
    let mut result = vec![];
    for (e, (p, t, _)) in world.query::<(&Position, &Team, &Health)>().iter() {
        if *p == pos && t.is_enemy(enemies_of) {
            result.push(e);
        }
    }
    result
}

pub fn get_position(world: &World, id: Entity) -> Option<Position> {
    world.query_one::<&Position>(id).unwrap().get().cloned()
}
