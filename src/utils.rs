use crate::health::Health;
use crate::models::{Label, ObjectsKind, Position};
use crate::team::Team;
use hecs::{Entity, World};

pub fn find_objects_at(world: &World, pos: Position) -> Vec<(Entity, ObjectsKind, Label)> {
    let mut result = vec![];
    for (e, (o, p, l)) in world.query::<(&ObjectsKind, &Position, &Label)>().iter() {
        if *p == pos {
            result.push((e.clone(), o.clone(), l.clone()));
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
