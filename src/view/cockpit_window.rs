use crate::view::window::Window;
use crate::{cfg, cockpit, ship, GMap, Label, Location, Player, Position, Ship, State};
use log::info;
use rltk::{Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::*;
use std::borrow::Borrow;

struct LocalInfo {
    pub avatar_id: Entity,
    pub ship_id: Entity,
}

impl LocalInfo {
    fn new(ecs: &World) -> LocalInfo {
        let pos_storage = ecs.read_storage::<Position>();
        let ship_storage = ecs.read_storage::<Ship>();

        let player = ecs.fetch::<Player>();
        let avatar_id = player.get_avatar_id();
        let pos = pos_storage.get(avatar_id).unwrap();
        let ship_id = pos.grid_id;

        LocalInfo { avatar_id, ship_id }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SubWindow {
    Main,
    Status,
}

#[derive(Component, Debug)]
pub struct CockpitWindowState {
    pub sub_window: SubWindow,
    pub last_msg: Option<String>,
}

impl Default for CockpitWindowState {
    fn default() -> Self {
        CockpitWindowState {
            sub_window: SubWindow::Main,
            last_msg: None,
        }
    }
}

pub fn draw(state: &mut State, ctx: &mut Rltk) {
    let info = LocalInfo::new(&state.ecs);

    let sub_window = {
        let wks = state.ecs.fetch::<CockpitWindowState>();
        wks.sub_window.clone()
    };

    match sub_window {
        SubWindow::Main => draw_main(state, ctx, info),
        SubWindow::Status => draw_status(state, ctx, info),
    }
}

fn draw_main(state: &mut State, ctx: &mut Rltk, info: LocalInfo) {
    let labels = state.ecs.read_storage::<Label>();

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

    let commands = super::super::cockpit::list_commands(&state.ecs, info.ship_id);
    for (i, command) in commands.iter().enumerate() {
        let command_str = match command {
            cockpit::Command::Status => "status".to_string(),
            cockpit::Command::Land => "land".to_string(),
            cockpit::Command::FlyTo { target_id } => {
                let label = labels.get(*target_id);
                let name = label.map(|i| i.name.as_str()).unwrap_or("unknown");
                format!("fly to {}", name)
            }
            cockpit::Command::Launch => "launch".to_string(),
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
    std::mem::drop(labels);
    {
        let window_state = state.ecs.fetch::<CockpitWindowState>();
        if let Some(msg) = &window_state.last_msg {
            ctx.print_color(x, cfg::SCREEN_H - border - 1, rltk::GRAY, rltk::RED, msg);
        }
    }

    let executed = match ctx.key {
        Some(VirtualKeyCode::Key0) => try_do_command(state, ctx, info.ship_id, commands.get(0)),
        Some(VirtualKeyCode::Key1) => try_do_command(state, ctx, info.ship_id, commands.get(1)),
        Some(VirtualKeyCode::Key2) => try_do_command(state, ctx, info.ship_id, commands.get(2)),
        Some(VirtualKeyCode::Key3) => try_do_command(state, ctx, info.ship_id, commands.get(3)),
        Some(VirtualKeyCode::Key4) => try_do_command(state, ctx, info.ship_id, commands.get(4)),
        Some(VirtualKeyCode::Key5) => try_do_command(state, ctx, info.ship_id, commands.get(5)),
        Some(VirtualKeyCode::Key6) => try_do_command(state, ctx, info.ship_id, commands.get(6)),
        Some(VirtualKeyCode::Key7) => try_do_command(state, ctx, info.ship_id, commands.get(7)),
        Some(VirtualKeyCode::Escape) => {
            state.ecs.insert(Window::World);
            Ok(())
        }
        _ => Ok(()),
    };

    match executed {
        Err(msg) => {
            let mut window_state = state.ecs.fetch_mut::<CockpitWindowState>();
            window_state.last_msg = Some(msg);
        }
        _ => {}
    }
}

fn draw_status(state: &mut State, ctx: &mut Rltk, info: LocalInfo) {
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

    let ship_storage = state.ecs.read_storage::<Ship>();
    let location_storage = state.ecs.read_storage::<Location>();

    let ship = ship_storage.get(info.ship_id);
    let location = location_storage.get(info.ship_id);

    match (location, &ship.map(|i| i.current_command)) {
        (Some(Location::Sector { pos, .. }), Some(ship::Command::FlyTo { .. })) => {
            ctx.print_color(
                x,
                y,
                rltk::GRAY,
                rltk::BLACK,
                format!("Ship at {:?} flying in space", pos),
            );
            y += 1;
        }
        (Some(Location::Sector { pos, .. }), Some(ship::Command::Idle)) => {
            ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Ship is drifting in space");
            y += 1;
        }
        (Some(Location::Orbit { target_id, .. }), _) => {
            ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Ship is orbiting a object");
            y += 1;
        }
        (Some(Location::BodySurfacePlace { .. }), _) => {
            ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Ship landed");
            y += 1;
        }
        _ => {
            ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Ship is the unknown");
            y += 1;
        }
    }

    match ctx.key {
        Some(VirtualKeyCode::Escape) => {
            state.ecs.fetch_mut::<CockpitWindowState>().sub_window = SubWindow::Main;
        }
        _ => {}
    }
}

fn try_do_command(
    state: &mut State,
    ctx: &mut Rltk,
    ship_id: Entity,
    command: Option<&cockpit::Command>,
) -> Result<(), String> {
    match command {
        Some(cockpit::Command::Status) => {
            state.ecs.fetch_mut::<CockpitWindowState>().sub_window = SubWindow::Status;
            Ok(())
        }
        Some(cockpit::Command::FlyTo { target_id }) => {
            let ship_command = ship::Command::FlyTo {
                target_id: *target_id,
            };
            info!("update ship {:?} command to {:?}", ship_id, ship_command);
            state
                .ecs
                .write_storage::<Ship>()
                .get_mut(ship_id)
                .unwrap()
                .current_command = ship_command;
            state.ecs.fetch_mut::<CockpitWindowState>().sub_window = SubWindow::Status;
            Ok(())
        }
        _ => Err("unknown command".to_string()),
    }
}
