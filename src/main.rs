extern crate core;

use rltk::Rltk;
use specs::prelude::*;
use state::State;

use crate::actions::actions_system::ActionsSystem;
use crate::actions::avatar_actions_system::FindAvatarActionsSystem;
use crate::area::Area;
use crate::commons::grid::NGrid;
use crate::commons::v2i;
use crate::commons::v2i::V2I;
use crate::models::*;
use crate::ship::Ship;
use crate::view::cockpit_window::CockpitWindowState;
use crate::view::window::Window;
use crate::visibility_system::VisibilitySystem;

pub mod actions;
pub mod area;
pub mod cfg;
pub mod commons;
pub mod events;
pub mod gridref;
pub mod loader;
pub mod locations;
pub mod models;
pub mod sectors;
pub mod ship;
pub mod state;
pub mod utils;
pub mod view;
pub mod visibility_system;

pub fn run_systems(st: &mut State, _ctx: &mut Rltk) {
    let mut s = VisibilitySystem {};
    s.run_now(&st.ecs);

    let mut s = FindAvatarActionsSystem {};
    s.run_now(&st.ecs);

    let mut s = ActionsSystem {};
    s.run_now(&st.ecs);

    let mut s = ship::systems::FlyToSystem {};
    s.run_now(&st.ecs);

    st.ecs.maintain();
}

fn main() -> rltk::BError {
    // setup
    use rltk::RltkBuilder;

    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let context = RltkBuilder::simple80x50().with_title("Alien").build()?;

    // initialize
    let cfg = cfg::Cfg::new();

    let ship_map_ast = loader::parse_map(&cfg.map_parser, cfg::SHIP_MAP).expect("fail to load map");
    let ship_grid = loader::new_grid_from_ast(&ship_map_ast);

    let house_ast =
        loader::parse_map(&cfg.map_parser, cfg::HOUSE_MAP).expect("fail to load house map");
    let house_grid = loader::new_grid_from_ast(&house_ast);

    let spawn_x = ship_grid.get_width() / 2 - 5;
    let spawn_y = ship_grid.get_height() / 2;

    let mut gs = State::new(cfg);
    gs.ecs.insert(Window::World);
    gs.ecs.insert(CockpitWindowState::default());

    // load scenery
    let sector_id = loader::create_sector(&mut gs.ecs);
    log::debug!("sector id {:?}", sector_id);

    let mut planets_zones: Vec<(Entity, SurfaceTileKind)> = (0..3)
        .map(|i| loader::create_planet_zone(&mut gs.ecs, i, 100, area::Tile::Ground))
        .map(|e| (e, SurfaceTileKind::Plain))
        .collect();

    let house_pos = V2I::new(15, 15);
    let house_grid_id = loader::create_planet_zone_from(
        &mut gs.ecs,
        3,
        100,
        area::Tile::Ground,
        vec![(house_pos, &house_grid)],
    );
    planets_zones.push((house_grid_id, SurfaceTileKind::Structure));

    log::debug!("planet zones id {:?}", planets_zones);

    let planet_id = loader::create_planet(
        &mut gs.ecs,
        "Planet X",
        Location::Sector {
            sector_id,
            pos: P2::new(5, 0),
        },
        planets_zones,
        2,
    );
    log::debug!("planet id {:?}", planet_id);

    let ship_location = Location::Orbit {
        target_id: planet_id,
    };
    // let ship_location = Location::Sector {
    //     sector_id: sector_id,
    //     pos: P2::new(0, 0),
    // }
    let ship_id = loader::create_ship(
        &mut gs.ecs,
        "ship",
        Ship {
            current_command: ship::Command::Idle,
            move_calm_down: 0,
        },
        ship_location,
        NGrid::from_grid(ship_grid),
    );
    log::debug!("ship id {:?}", ship_id);

    let avatar_entity_id = loader::create_avatar(
        &mut gs.ecs,
        Position {
            grid_id: ship_id,
            point: (spawn_x, spawn_y).into(),
        },
    );
    log::info!("avatar id: {:?}", avatar_entity_id);

    gs.ecs.insert(Player::new(avatar_entity_id));

    // load objects
    loader::parse_map_objects(&mut gs.ecs, v2i::ZERO, ship_id, ship_map_ast)
        .expect("fail to load map objects");
    loader::parse_map_objects(&mut gs.ecs, house_pos, house_grid_id, house_ast)
        .expect("fail to load map objects");

    sectors::update_bodies_list(&mut gs.ecs);

    rltk::main_loop(context, gs)
}

#[cfg(test)]
mod test {
    use crate::cfg::Cfg;
    use crate::state::State;

    #[test]
    fn test_acceptance() {
        let mut s = new_state_basic_scenery();
    }

    pub fn new_state_basic_scenery() {
        let mut state = new_state();
    }

    pub fn new_state() -> State {
        State::new(Cfg::new())
    }
}
