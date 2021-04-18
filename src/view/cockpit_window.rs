use crate::view::window::Window;
use crate::{cfg, State};
use rltk::{Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::*;

pub fn input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        Some(VirtualKeyCode::Escape) => {
            gs.ecs.insert(Window::World);
        }
        _ => {}
    }
}

pub fn draw(state: &State, ctx: &mut Rltk) {
    let border = 4;

    ctx.draw_box(
        border,
        border,
        cfg::SCREEN_W - border * 2,
        cfg::SCREEN_H - border * 2,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    ctx.print_color(
        border + 2,
        border + 2,
        rltk::GRAY,
        rltk::BLACK,
        "The cockpit",
    );
}
