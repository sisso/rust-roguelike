use crate::actions::{get_available_actions, Action, EntityActions};
use hecs::World;

use crate::models::Position;
use crate::utils::find_objects_at;

use crate::view::window::Window;

pub fn run(world: &mut World, window: &mut Window) {
    for (_e, (actions, pos)) in &mut world.query::<(&mut EntityActions, &Position)>() {
        // take current action and check if can be executed
        match actions.current.take() {
            Some(action) => {
                let objects_at = find_objects_at(&world, pos);
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
