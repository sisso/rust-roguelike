extern crate core;

use crate::actions::{actions_system, avatar_actions_system};
use hecs::Entity;
use rltk::Rltk;
use state::State;

use crate::area::Area;
use crate::commons::grid::NGrid;
use crate::commons::v2i::V2I;
use crate::commons::{grid_string, v2i};
use crate::loader::MapAstCell;
use crate::models::*;
use crate::ship::Ship;
use crate::view::cockpit_window::CockpitWindowState;
use crate::view::window::Window;

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

pub fn run_systems(st: &mut State, ctx: &mut Rltk) {
    visibility_system::run(&st.ecs);
    avatar_actions_system::run(&mut st.ecs);
    actions_system::run(&mut st.ecs, &mut st.window);
    ship::systems::run(&mut st.ecs);
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

    let parser = |ch| {
        let tile = cfg
            .map_parser
            .raw_map_tiles
            .iter()
            .find(|(c, tile)| *c == ch)
            .map(|(_, tile)| *tile)?;

        let obj = cfg
            .map_parser
            .raw_map_objects
            .iter()
            .find(|(c, tile)| *c == ch)
            .map(|(_, obj)| *obj);

        Some(MapAstCell {
            tile: tile,
            obj: obj,
        })
    };

    let ship_map_ast = grid_string::parse_map(parser, cfg::SHIP_MAP).expect("fail to load map");
    let ship_grid = loader::new_grid_from_ast(&ship_map_ast);

    let house_ast = grid_string::parse_map(parser, cfg::HOUSE_MAP).expect("fail to load house map");
    let house_grid = loader::new_grid_from_ast(&house_ast);

    let spawn_x = ship_grid.get_width() / 2 - 5;
    let spawn_y = ship_grid.get_height() / 2;

    let mut gs = State::new(cfg);
    gs.ecs.spawn((Window::World,));
    gs.ecs.spawn((CockpitWindowState::default(),));

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
        gs.player.get_avatar_id(),
        Position {
            grid_id: ship_id,
            point: (spawn_x, spawn_y).into(),
        },
    );
    log::info!("avatar id: {:?}", avatar_entity_id);

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
