use crate::actions::{get_available_actions, EntityActions};
use hecs::World;

use crate::models::Position;
use crate::utils::find_objects_at;

pub fn run(world: &mut World) {
    for (_, (actions, pos)) in &mut world.query::<(&mut EntityActions, &Position)>() {
        let objects_at = find_objects_at(&world, pos);
        actions.actions = get_available_actions(&objects_at);
    }
}
