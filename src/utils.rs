use crate::health::Health;
use crate::models::{Label, ObjectsKind, Position};
use crate::team::Team;
use crate::utils;
use hecs::{Entity, World};
use std::iter::Map;
use std::slice::Iter;

pub fn get_kind(world: &World, id: Entity) -> ObjectsKind {
    let mut q = world.query_one::<&ObjectsKind>(id).unwrap();
    q.get().unwrap().clone()
}

pub fn find_objects_at(world: &World, pos: Position) -> Vec<(Entity, ObjectsKind)> {
    let mut result = vec![];
    for (e, (o, p)) in world.query::<(&ObjectsKind, &Position)>().iter() {
        if *p == pos {
            result.push((e.clone(), o.clone()));
        }
    }
    result
}

pub fn find_objects_at_with_label(ecs: &World, pos: Position) -> Vec<(Entity, ObjectsKind, Label)> {
    let objects_at = find_objects_at(ecs, pos);
    let labels = find_labels(ecs, &objects_at.iter().map(|i| i.0).collect());
    objects_at
        .into_iter()
        .zip(labels.into_iter())
        .map(|i| (i.0 .0, i.0 .1, i.1))
        .collect::<Vec<_>>()
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

pub fn find_labels(world: &World, ids: &Vec<Entity>) -> Vec<Label> {
    let mut result: Vec<Label> = vec![];
    for id in ids {
        let label = &*world.get::<&Label>(*id).unwrap();
        result.push(label.clone());
    }
    result
}
