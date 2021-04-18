use crate::actions::{get_available_actions, Action, EntityActions};
use crate::gmap::GMap;
use crate::models::{Avatar, ObjectsType, Position};
use crate::ship;
use crate::utils::find_objects_at;
use crate::view::Viewshed;
use specs::prelude::*;

pub struct ActionsSystem {}

impl<'a> System<'a> for ActionsSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Avatar>,
        WriteStorage<'a, EntityActions>,
        ReadStorage<'a, ObjectsType>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, (entities, mut avatars, mut actions, objects, positions): Self::SystemData) {
        for (e, avatar, actions, pos) in (&entities, &mut avatars, &mut actions, &positions).join()
        {
            let objects_at =
                find_objects_at(&entities, &objects, &positions, pos.point.x, pos.point.y);

            match actions.current {
                None => continue,
                Some(Action::CheckCockpit) => {
                    match objects_at
                        .iter()
                        .find(|(_, ot)| *ot == ObjectsType::Cockpit)
                    {
                        Some((cockpit_entity, _)) => {
                            ship::enter_cockpit(avatar);
                        }
                        None => {}
                    }
                }
            }
        }
    }
}
