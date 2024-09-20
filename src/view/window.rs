use hecs::Entity;

#[derive(Copy, Clone)]
pub enum Window {
    World,
    Cockpit { cockpit_id: Entity },
}
