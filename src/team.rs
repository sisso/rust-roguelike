#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Team {
    Player,
    Mob,
}

impl Team {
    pub fn is_enemy(&self, other: Team) -> bool {
        *self != other
    }
}
