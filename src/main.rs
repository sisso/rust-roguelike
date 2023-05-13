extern crate core;

use std::collections::HashSet;

use log::*;
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
        .filter(None, LevelFilter::Debug)
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
    let sector_id = gs.ecs.create_entity().with(Sector::default()).build();
    let planets_zones_id = (0..4)
        .map(|i| {
            let size = 100;
            let total_cells = size * size;
            let mut cells = Vec::with_capacity(total_cells);
            for _j in 0..(total_cells) {
                cells.push(gmap::Cell {
                    tile: gmap::GMapTile::Ground,
                })
            }

            let builder = gs.ecs.create_entity();

            let gmap = GMap::new(
                NGrid::from_grid(Grid {
                    width: size as i32,
                    height: size as i32,
                    list: cells,
                })
                .into(),
                vec![builder.entity],
            );

            let zone_id = builder
                .with(Label {
                    name: format!("zone {}", i),
                })
                .with(GridRef::GMap(gmap))
                .build();

            zone_id
        })
        .collect::<Vec<_>>();

    log::debug!("planet zones id {:?}", planets_zones_id);

    let planet_id = gs
        .ecs
        .create_entity()
        .with(SectorBody::Planet)
        .with(Location::Sector {
            sector_id: sector_id,
            pos: P2::new(5, 0),
        })
        .with(Label {
            name: "Planet X".to_string(),
        })
        .with(Surface {
            width: 2,
            height: 2,
            tiles: vec![
                SurfaceTileKind::Plain,
                SurfaceTileKind::Plain,
                SurfaceTileKind::Plain,
                SurfaceTileKind::Plain,
            ],
            zones: planets_zones_id,
        })
        .build();

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

    info!("avatar id: {}", avatar_entity.id());

    gs.ecs.insert(Player::new(avatar_entity));

    loader::parse_map_objects(&mut gs.ecs, ship_id, ship_map_ast)
        .expect("fail to load map objects");

    sectors::update_bodies_list(&mut gs.ecs);

    rltk::main_loop(context, gs)
}
