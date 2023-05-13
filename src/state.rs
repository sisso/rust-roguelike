use crate::actions::EntityActions;
use crate::cfg::Cfg;
use crate::gridref::GridRef;
use crate::models::{
    Avatar, Label, Location, ObjectsType, Player, Position, Sector, SectorBody, Surface,
};
use crate::ship::Ship;
use crate::view;
use crate::view::cockpit_window::CockpitWindowState;
use crate::view::window::Window;
use crate::view::{Renderable, Viewshed};
use rltk::BTerm as Rltk;
use specs::prelude::*;
use specs::World;

pub struct State {
    pub ecs: World,
}

impl State {
    pub fn new(cfg: Cfg) -> Self {
        let mut gs = State { ecs: World::new() };
        gs.ecs.register::<Cfg>();
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
        gs.ecs.register::<Location>();
        gs.ecs.register::<Surface>();
        gs.ecs.register::<Sector>();
        gs.ecs.register::<Label>();
        gs.ecs.register::<SectorBody>();
        gs.ecs.register::<GridRef>();

        gs.ecs.insert(cfg);

        gs
    }
}

impl rltk::GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        let window = *self.ecs.fetch::<Window>();

        match window {
            Window::World => {
                view::player_input(self, ctx);
                crate::run_systems(self, ctx);
                view::draw_map_and_objects(self, ctx);
                view::draw_gui(self, ctx);
            }

            Window::Cockpit => {
                crate::run_systems(self, ctx);
                view::draw_map_and_objects(self, ctx);
                view::cockpit_window::draw(self, ctx);
            }
        }
    }
}
