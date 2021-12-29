use std::collections::HashSet;

use log::*;
use rltk::{Rltk, RGB};
use specs::prelude::*;

use crate::actions::actions_system::ActionsSystem;
use crate::actions::avatar_actions_system::FindAvatarActionsSystem;
use crate::actions::EntityActions;
use crate::gmap::GMap;
use crate::models::*;
use crate::ship::{Ship, ShipState};
use crate::view::cockpit_window::CockpitWindowState;
use crate::view::window::Window;
use crate::view::{Renderable, Viewshed};
use crate::visibility_system::VisibilitySystem;

pub mod actions;
pub mod cfg;
pub mod cockpit;
pub mod events;
pub mod gmap;
pub mod loader;
pub mod models;
mod ngridmap;
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

    st.ecs.maintain();
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let cfg = cfg::Cfg::new();
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
    gs.ecs.register::<GMap>();

    let map_ast = loader::parse_map(cfg::SHIP_MAP).expect("fail to load map");
    let map =
        loader::parse_map_tiles(&cfg.raw_map_tiles, &&map_ast).expect("fail to load map tiles");

    let spawn_x = map.width / 2;
    let spawn_y = map.height / 2;

    gs.ecs.insert(Window::World);
    gs.ecs.insert(cfg);
    gs.ecs.insert(CockpitWindowState::default());
    let grid_id = gs
        .ecs
        .create_entity()
        .with(Ship {
            state: ShipState::Space,
        })
        .with(map)
        .build();
    let avatar_entity = gs
        .ecs
        .create_entity()
        .with(Avatar {})
        .with(Position {
            grid_id,
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

    loader::parse_map_objects(&mut gs.ecs, grid_id, map_ast).expect("fail to load map objects");

    rltk::main_loop(context, gs)
}
