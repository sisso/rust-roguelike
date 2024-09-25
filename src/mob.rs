use crate::models::Position;
use hecs::{Entity, World};

#[derive(Debug, Clone, Default)]
pub struct Mob {}

pub fn find_mobs_at(world: &World, pos: &Position) -> Vec<Entity> {
    let mut result = vec![];
    for (e, (_, p)) in world.query::<(&Mob, &Position)>().iter() {
        if p == pos {
            result.push(e.clone());
        }
    }
    result
}
