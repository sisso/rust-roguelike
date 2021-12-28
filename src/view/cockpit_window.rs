use crate::cockpit::Command;
use crate::view::window::Window;
use crate::{cfg, State};
use rltk::{Rltk, VirtualKeyCode, RGB};

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

    let mut x = border + 2;
    let mut y = border + 2;

    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "The cockpit");
    y += 2;

    let commands = super::super::cockpit::list_commands(state);
    for (i, command) in commands.iter().enumerate() {
        let command_str = match command {
            Command::Status => "status",
            Command::Land => "land",
            Command::FlyTo => "fly to",
            Command::Launch => "launch",
        };
        ctx.print_color(
            x,
            y,
            rltk::GRAY,
            rltk::BLACK,
            format!("{}) {}", i, command_str),
        );
        y += 1;
    }
}
