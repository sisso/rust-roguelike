use std::collections::HashSet;

use log::*;
use rltk::{Rltk, RGB};
use specs::prelude::*;

use crate::actions::actions_system::ActionsSystem;
use crate::actions::avatar_actions_system::FindAvatarActionsSystem;
use crate::actions::EntityActions;
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
pub mod loader;
pub mod locations;
pub mod models;
pub mod ngridmap;
pub mod sectors;
pub mod ship;
pub mod utils;
pub mod view;
pub mod visibility_system;

pub struct State {
    pub ecs: World,
}

impl rltk::GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let window = *self.ecs.fetch::<Window>();

        match window {
            Window::World => {
                view::player_input(self, ctx);
                run_systems(self, ctx);
                view::draw_map_and_objects(self, ctx);
                view::draw_gui(self, ctx);
            }

            Window::Cockpit => {
                run_systems(self, ctx);
                view::draw_map_and_objects(self, ctx);
                view::cockpit_window::draw(self, ctx);
            }
        }
    }
}

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

    env_logger::builder().filter(None, LevelFilter::Info).init();

    let context = RltkBuilder::simple80x50().with_title("Alien").build()?;
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<cfg::Cfg>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<ObjectsType>();
    gs.ecs.register::<EntityActions>();
    gs.ecs.register::<Window>();
    gs.ecs.register::<Ship>();
    gs.ecs.register::<Avatar>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<CockpitWindowState>();
    gs.ecs.register::<gmap::GMap>();
    gs.ecs.register::<Location>();
    gs.ecs.register::<Surface>();
    gs.ecs.register::<Sector>();
    gs.ecs.register::<Label>();
    gs.ecs.register::<SectorBody>();

    // initialize
    let cfg = cfg::Cfg::new();
    let ship_map_ast = loader::parse_map(cfg::SHIP_MAP).expect("fail to load map");
    let ship_map = loader::parse_map_tiles(&cfg.raw_map_tiles, &&ship_map_ast)
        .expect("fail to load map tiles");

    let spawn_x = ship_map.width / 2;
    let spawn_y = ship_map.height / 2;

    gs.ecs.insert(Window::World);
    gs.ecs.insert(cfg);
    gs.ecs.insert(CockpitWindowState::default());

    // load scenery
    let sector_id = gs.ecs.create_entity().with(Sector::default()).build();
    let planets_zones_id = (0..4)
        .map(|i| {
            let size = 10;
            let total_cells = size * size;
            let mut cells = Vec::with_capacity(total_cells);
            for j in 0..(total_cells) {
                cells.push(gmap::Cell {
                    tile: gmap::GMapTile::Ground,
                })
            }

            let gmap = gmap::GMap {
                width: size as i32,
                height: size as i32,
                cells: cells,
            };

            let zone_id = gs
                .ecs
                .create_entity()
                .with(gmap)
                .with(Label {
                    name: format!("zone {}", i),
                })
                .build();

            zone_id
        })
        .collect::<Vec<_>>();

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

    let ship_id = gs
        .ecs
        .create_entity()
        .with(Label {
            name: "ship".to_string(),
        })
        .with(Ship {
            current_command: ship::Command::Idle,
            move_calm_down: 0,
        })
        .with(Location::Sector {
            sector_id: sector_id,
            pos: P2::new(0, 0),
        })
        .with(ship_map)
        .build();
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

    gs.ecs.insert(Player::new(avatar_entity));

    loader::parse_map_objects(&mut gs.ecs, ship_id, ship_map_ast)
        .expect("fail to load map objects");

    sectors::update_bodies_list(&mut gs.ecs);

    rltk::main_loop(context, gs)
}
