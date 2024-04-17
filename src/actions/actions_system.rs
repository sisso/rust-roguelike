use crate::actions::{get_available_actions, Action, EntityActions};

use crate::models::{ObjectsType, Position};
use crate::utils::find_objects_at;

use crate::view::window::Window;
use specs::prelude::*;

pub struct ActionsSystem {}

impl<'a> System<'a> for ActionsSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, EntityActions>,
        ReadStorage<'a, ObjectsType>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Window>,
    );

    fn run(&mut self, (entities, mut actions, objects, positions, mut window): Self::SystemData) {
        for (_e, actions, pos) in (&entities, &mut actions, &positions).join() {
            // take current action and check if can be executed
            match actions.current.take() {
                Some(action) => {
                    let objects_at = find_objects_at(&entities, &objects, &positions, pos);
                    let available_actions = get_available_actions(&objects_at);
                    match available_actions.into_iter().find(|i| i == &action) {
                        Some(Action::Interact) => {
                            *window = Window::Cockpit;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
