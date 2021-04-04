mod actions;
pub mod cfg;
pub mod gmap;
pub mod loader;
pub mod models;
pub mod systems;
pub mod view;

use crate::gmap::GMap;
use crate::models::*;
use crate::systems::visibility_system::VisibilitySystem;
use crate::view::{camera::Camera, Renderable, Viewshed};
use log::*;
use rltk::{Point, Rect, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

use std::collections::HashSet;

pub struct State {
    pub ecs: World,
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => actions::try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => actions::try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => actions::try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => actions::try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad7 => actions::try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad8 => actions::try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad9 => actions::try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad4 => actions::try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad5 => actions::try_move_player(0, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad6 => actions::try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad1 => actions::try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad2 => actions::try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad3 => actions::try_move_player(1, 1, &mut gs.ecs),
            // VirtualKeyCode::W => gs.camera.y -= 1,
            // VirtualKeyCode::A => gs.camera.x -= 1,
            // VirtualKeyCode::D => gs.camera.x += 1,
            // VirtualKeyCode::S => gs.camera.y += 1,
            _ => {}
        },
    }
}

impl rltk::GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        match &ctx.key {
            Some(VirtualKeyCode::Return) => {
                debug!("generate a new map");
                self.ecs
                    .insert(loader::map_empty(cfg::SCREEN_W, cfg::SCREEN_H));
            }
            _ => {}
        }

        {
            // merge all visible and know tiles from player
            let viewshed = self.ecs.read_storage::<Viewshed>();
            let avatars = self.ecs.read_storage::<Avatar>();
            let positions = self.ecs.read_storage::<Position>();
            let views = (&viewshed, &avatars, &positions).join().collect::<Vec<_>>();
            let (v, _, pos) = views.iter().next().unwrap();

            let camera = Camera::fromCenter(pos.point);

            // draw
            let map = self.ecs.fetch::<GMap>();
            view::draw_map(&camera, &v.visible_tiles, &v.know_tiles, &map, ctx);
            view::draw_objects(&camera, &v.visible_tiles, &self.ecs, ctx);
        }

        view::draw_gui(self, ctx);

        {
            let mouse_pos = ctx.mouse_pos();
            ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
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
    gs.ecs.register::<Avatar>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<ObjectsType>();

    let map_ast = loader::parse_map(cfg::SHIP_MAP).expect("fail to load map");
    let map =
        loader::parse_map_tiles(&cfg.raw_map_tiles, &&map_ast).expect("fail to load map tiles");

    let spawn_x = map.width / 2;
    let spawn_y = map.height / 2;

    gs.ecs.insert(map);
    gs.ecs.insert(cfg);
    gs.ecs
        .create_entity()
        .with(Position {
            point: (spawn_x, spawn_y).into(),
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            priority: 1,
        })
        .with(Avatar {})
        .with(Viewshed {
            visible_tiles: vec![],
            know_tiles: HashSet::new(),
            range: 16,
        })
        .build();

    loader::parse_map_objects(&mut gs.ecs, map_ast).expect("fail to load map objects");

    rltk::main_loop(context, gs)
}
