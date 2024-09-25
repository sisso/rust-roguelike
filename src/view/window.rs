use hecs::Entity;

#[derive(Copy, Clone)]
pub enum Window {
    MainMenu,
    Lose,
    World,
    Cockpit { cockpit_id: Entity },
}
