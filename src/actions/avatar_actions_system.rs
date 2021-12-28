use crate::actions::{get_available_actions, EntityActions};

use crate::models::{ObjectsType, Player, Position};
use crate::utils::find_objects_at;

use specs::prelude::*;
use specs::shred::Fetch;

pub struct FindAvatarActionsSystem {}

impl<'a> System<'a> for FindAvatarActionsSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Player>,
        WriteStorage<'a, EntityActions>,
        ReadStorage<'a, ObjectsType>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, (entities, avatar, mut actions, objects, positions): Self::SystemData) {
        for (_, actions, pos) in (avatar.get_avatarset(), &mut actions, &positions).join() {
            let objects_at =
                find_objects_at(&entities, &objects, &positions, pos.point.x, pos.point.y);

            actions.actions = get_available_actions(&objects_at);
        }
    }
}
