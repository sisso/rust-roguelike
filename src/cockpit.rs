use crate::State;

pub enum Command {
    Status,
    Land,
    FlyTo,
    Launch,
}

pub fn list_commands(state: &State) -> Vec<Command> {
    todo!()
}
