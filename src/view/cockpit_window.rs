use crate::cockpit::Command;
use crate::view::window::Window;
use crate::{cfg, State};
use rltk::{Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::*;
use std::ops::Sub;

// pub fn input(gs: &mut State, ctx: &mut Rltk) {
//     match ctx.key {
//         Some(VirtualKeyCode::Escape) => {
//             gs.ecs.insert(Window::World);
//         }
//         _ => {}
//     }
// }

#[derive(Clone, Copy, Debug)]
pub enum SubWindow {
    Main,
    Status,
}

#[derive(Component, Debug)]
pub struct CockpitWindowState {
    pub sub_window: SubWindow,
}

impl Default for CockpitWindowState {
    fn default() -> Self {
        CockpitWindowState {
            sub_window: SubWindow::Main,
        }
    }
}

pub fn draw(state: &mut State, ctx: &mut Rltk) {
    let sub_window = {
        let wks = state.ecs.fetch::<CockpitWindowState>();
        wks.sub_window.clone()
    };

    match sub_window {
        SubWindow::Main => draw_main(state, ctx),
        SubWindow::Status => draw_status(state, ctx),
    }
}

pub fn draw_main(state: &mut State, ctx: &mut Rltk) {
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

    match ctx.key {
        Some(VirtualKeyCode::Key0) => try_do_command(state, ctx, commands.get(0).cloned()),
        Some(VirtualKeyCode::Key1) => try_do_command(state, ctx, commands.get(1).cloned()),
        Some(VirtualKeyCode::Key2) => try_do_command(state, ctx, commands.get(2).cloned()),
        Some(VirtualKeyCode::Key3) => try_do_command(state, ctx, commands.get(3).cloned()),
        Some(VirtualKeyCode::Key4) => try_do_command(state, ctx, commands.get(4).cloned()),
        Some(VirtualKeyCode::Key5) => try_do_command(state, ctx, commands.get(5).cloned()),
        Some(VirtualKeyCode::Key6) => try_do_command(state, ctx, commands.get(6).cloned()),
        Some(VirtualKeyCode::Key7) => try_do_command(state, ctx, commands.get(7).cloned()),
        Some(VirtualKeyCode::Escape) => state.ecs.insert(Window::World),
        _ => {}
    }
}

pub fn draw_status(state: &mut State, ctx: &mut Rltk) {
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

    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "The cockpit Status page");
    y += 2;

    match ctx.key {
        Some(VirtualKeyCode::Escape) => {
            state.ecs.fetch_mut::<CockpitWindowState>().sub_window = SubWindow::Main;
        }
        _ => {}
    }
}

fn try_do_command(state: &mut State, ctx: &mut Rltk, command: Option<Command>) {
    if let Some(c) = command {
        do_command(state, ctx, c);
    }
}

fn do_command(state: &mut State, ctx: &mut Rltk, command: Command) {
    match command {
        Command::Status => {
            state.ecs.fetch_mut::<CockpitWindowState>().sub_window = SubWindow::Status;
        }
        _ => {}
    }
}
