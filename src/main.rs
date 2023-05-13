extern crate core;

use std::collections::HashSet;

use rltk::{Rltk, RGB};
use specs::prelude::*;
use state::State;

use crate::actions::actions_system::ActionsSystem;
use crate::actions::avatar_actions_system::FindAvatarActionsSystem;
use crate::actions::EntityActions;
use crate::commons::grid::{Grid, NGrid};
use crate::gmap::GMap;
use crate::models::*;
use crate::ship::Ship;
use crate::view::cockpit_window::CockpitWindowState;
use crate::view::window::Window;
use crate::view::{Renderable, Viewshed};
use crate::visibility_system::VisibilitySystem;

pub mod actions;
pub mod cfg;
pub mod commons;
pub mod events;
pub mod gmap;
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
    use gridref::GridRef;
    use rltk::RltkBuilder;

    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    let context = RltkBuilder::simple80x50().with_title("Alien").build()?;

    // initialize
    let cfg = cfg::Cfg::new();

    let ship_map_ast = loader::parse_map(cfg::SHIP_MAP).expect("fail to load map");
    let mut ship_grid =
        loader::parse_map_tiles(&cfg.raw_map_tiles, &ship_map_ast).expect("fail to load map tiles");

    let spawn_x = ship_grid.width / 2 - 5;
    let spawn_y = ship_grid.height / 2;

    let mut gs = State::new(cfg);
    gs.ecs.insert(Window::World);
    gs.ecs.insert(CockpitWindowState::default());

    // load scenery
    let sector_id = loader::create_sector(&mut gs.ecs);
    log::debug!("sector id {:?}", sector_id);

    let planets_zones = (0..4)
        .map(|i| loader::create_planet_zone(&mut gs.ecs, i, 100, gmap::GMapTile::Ground))
        .map(|z| (z, SurfaceTileKind::Plain))
        .collect();
    log::debug!("planet zones id {:?}", planets_zones);

    let planet_id = loader::create_planet(&mut gs.ecs, sector_id, "Planet X", planets_zones, 2);
    log::debug!("planet id {:?}", planet_id);

    let builder = gs.ecs.create_entity();
    let ship_id = builder.entity;

    let ship_gmap = GMap::new(NGrid::from_grid(ship_grid), vec![ship_id]);

    let ship_id = builder
        .with(Label {
            name: "ship".to_string(),
        })
        .with(Ship {
            current_command: ship::Command::Idle,
            move_calm_down: 0,
        })
        .with(Location::Orbit {
            target_id: planet_id,
        })
        // .with(Location::Sector {
        //     sector_id: sector_id,
        //     pos: P2::new(0, 0),
        // })
        .with(GridRef::GMap(ship_gmap))
        .build();
    log::debug!("ship id {:?}", ship_id);

    let avatar_entity = gs
        .ecs
        .create_entity()
        .with(Avatar {})
        .with(Label {
            name: "player".to_string(),
        })
        .with(Position {
            grid_id: ship_id,
            point: (spawn_x, spawn_y).into(),
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            priority: 1,
        })
        .with(Viewshed {
            visible_tiles: vec![],
            know_tiles: HashSet::new(),
            range: 16,
        })
        .with(EntityActions {
            actions: vec![],
            current: None,
        })
        .build();

    log::info!("avatar id: {}", avatar_entity.id());

    gs.ecs.insert(Player::new(avatar_entity));

    loader::parse_map_objects(&mut gs.ecs, ship_id, ship_map_ast)
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
