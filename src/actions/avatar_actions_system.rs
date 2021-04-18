use crate::actions::{get_available_actions, EntityActions};
use crate::gmap::GMap;
use crate::models::{Avatar, ObjectsType, Position};
use crate::utils::find_objects_at;
use crate::view::Viewshed;
use specs::prelude::*;

pub struct FindAvatarActionsSystem {}

impl<'a> System<'a> for FindAvatarActionsSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Avatar>,
        WriteStorage<'a, EntityActions>,
        ReadStorage<'a, ObjectsType>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, (entities, avatars, mut actions, objects, positions): Self::SystemData) {
        for (avatar, actions, pos) in (&avatars, &mut actions, &positions).join() {
            let objects_at =
                find_objects_at(&entities, &objects, &positions, pos.point.x, pos.point.y);

            actions.actions = get_available_actions(avatar, &objects_at);
        }
    }
}
