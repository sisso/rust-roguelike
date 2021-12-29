use crate::State;

#[derive(Clone, Debug)]
pub enum Command {
    Status,
    Land,
    FlyTo,
    Launch,
}

pub fn list_commands(state: &State) -> Vec<Command> {
    vec![Command::Status, Command::Land]
}
