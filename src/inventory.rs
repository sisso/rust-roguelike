use hecs::Entity;

#[derive(Clone, Debug, Default)]
pub struct Inventory {
    pub items: Vec<Entity>,
}

impl Inventory {}
