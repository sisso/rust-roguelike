use crate::cockpit::Command;
use crate::view::window::Window;
use crate::{
    cfg, cockpit, ship, GMap, Label, Location, Player, Position, Sector, SectorBody, Ship, State,
    Surface,
};
use log::{info, warn};
use rltk::{BTerm, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::*;
use std::borrow::Borrow;

struct LocalInfo {
    pub avatar_id: Entity,
    pub ship_id: Entity,
    pub orbiting_id: Option<Entity>,
}

impl LocalInfo {
    fn new(ecs: &World) -> LocalInfo {
        let player = ecs.fetch::<Player>();
        let avatar_id = player.get_avatar_id();
        let pos_storage = ecs.read_storage::<Position>();

        let pos = pos_storage.get(avatar_id).unwrap();
        let ship_id = pos.grid_id;

        let locations_storage = ecs.read_storage::<Location>();
        let orbiting_id = match locations_storage.get(ship_id) {
            Some(Location::Orbit { target_id }) => Some(*target_id),
            _ => None,
        };

        LocalInfo {
            avatar_id,
            ship_id,
            orbiting_id,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SubWindow {
    Main,
    Land,
}

#[derive(Component, Debug)]
pub struct CockpitWindowState {
    pub sub_window: SubWindow,
    pub last_msg: Option<String>,
}

impl CockpitWindowState {
    pub fn new(sub_window: SubWindow) -> Self {
        CockpitWindowState {
            sub_window,
            last_msg: None,
        }
    }
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
        SubWindow::Land => draw_land_menu(state, ctx, info),
    }
}

fn draw_main(state: &mut State, ctx: &mut Rltk, info: LocalInfo) {
    // frame
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

    // status
    y = draw_status(state, ctx, &info, x, y);
    // sector
    y = draw_sector_map(state, ctx, x, y, info.ship_id);
    // orbiting map
    y = draw_orbiting_map(state, ctx, &info, x, y);
    // actions
    let commands = super::super::cockpit::list_commands(&state.ecs, info.ship_id);
    y = draw_actions(state, ctx, x, y, &commands);
    // draw messages
    y = draw_msg(state, ctx, border, x, y);

    // process inputs
    let executed = match (ctx.key, get_key_index(ctx.key)) {
        (_, Some(index)) => try_do_command(state, ctx, info.ship_id, commands.get(index)),
        (Some(VirtualKeyCode::Escape), _) => {
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

fn draw_msg(state: &State, ctx: &mut BTerm, border: i32, x: i32, y: i32) -> i32 {
    let window_state = state.ecs.fetch::<CockpitWindowState>();
    if let Some(msg) = &window_state.last_msg {
        ctx.print_color(x, cfg::SCREEN_H - border - 1, rltk::GRAY, rltk::RED, msg);
    }
    y
}

fn draw_actions(
    state: &mut State,
    ctx: &mut BTerm,
    x: i32,
    mut y: i32,
    commands: &Vec<Command>,
) -> i32 {
    let labels = state.ecs.read_storage::<Label>();
    for (i, command) in commands.iter().enumerate() {
        let command_str = match command {
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

    y
}

fn draw_status(state: &mut State, ctx: &mut Rltk, info: &LocalInfo, x: i32, mut y: i32) -> i32 {
    let ship_storage = state.ecs.read_storage::<Ship>();
    let location_storage = state.ecs.read_storage::<Location>();

    let ship = ship_storage.get(info.ship_id);
    let location = location_storage.get(info.ship_id);

    y += 1;

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

    y
}

fn try_do_command(
    state: &mut State,
    ctx: &mut Rltk,
    ship_id: Entity,
    command: Option<&cockpit::Command>,
) -> Result<(), String> {
    match command {
        Some(cockpit::Command::Land) => state.ecs.insert(CockpitWindowState::new(SubWindow::Land)),

        Some(command) => cockpit::do_command(&mut state.ecs, ship_id, command),
        _ => {}
    }
    Ok(())
}

/// return ne y value
fn draw_sector_map(state: &mut State, ctx: &mut Rltk, x: i32, y: i32, ship_id: Entity) -> i32 {
    let entities = state.ecs.entities();
    let sectors = state.ecs.read_storage::<Sector>();
    let locations = state.ecs.read_storage::<Location>();
    let labels = state.ecs.read_storage::<Label>();

    // get ship location
    let (ship_pos, ship_sector_id) = match locations.get(ship_id) {
        Some(Location::Sector { pos, sector_id }) => (pos.clone(), *sector_id),
        Some(Location::Orbit { target_id }) => {
            match crate::locations::resolve_sector_pos(&locations, *target_id) {
                Some(value) => value,
                None => return y,
            }
        }
        _ => {
            return y;
        }
    };

    // draw frame
    let mut fg = rltk::GRAY;
    let bg = rltk::GRAY;
    for ix in 0..cfg::SECTOR_SIZE {
        for iy in 0..cfg::SECTOR_SIZE {
            ctx.set(x + ix, y + iy, fg, bg, ' ' as rltk::FontCharType);
        }
    }

    // draw objects
    let sector = sectors.get(ship_sector_id).unwrap();

    let mut bodies_bitset = BitSet::default();
    sector.bodies.iter().for_each(|i| {
        let _ = bodies_bitset.add(i.id());
    });

    for (e, loc, lab) in (&entities, &locations, &labels).join() {
        let (pos, _) = match crate::locations::resolve_sector_pos(&locations, e) {
            Some(value) => value,
            _ => continue,
        };

        let index_x = pos.x + cfg::SECTOR_SIZE / 2;
        let index_y = pos.y + cfg::SECTOR_SIZE / 2;

        if index_x < 0 || index_y < 0 || index_x >= cfg::SECTOR_SIZE || index_y >= cfg::SECTOR_SIZE
        {
            warn!(
                "entity {:?} position {:?} with index {:?} is outside of sector map",
                e,
                pos,
                (index_x, index_y)
            );
            continue;
        }

        if e == ship_id {
            fg = rltk::BLUE;
        } else {
            fg = rltk::GREEN;
        }

        ctx.set(x + index_x, y + index_y, fg, bg, '*' as rltk::FontCharType);
    }

    y + 1 + cfg::SECTOR_SIZE
}

fn draw_orbiting_map(
    state: &mut State,
    ctx: &mut Rltk,
    info: &LocalInfo,
    x: i32,
    mut y: i32,
) -> i32 {
    let locations_storage = state.ecs.read_storage::<Location>();
    let orbiting_id = match locations_storage.get(info.ship_id) {
        Some(Location::Orbit { target_id }) => target_id,
        _ => return y,
    };

    let surfaces_storage = state.ecs.read_storage::<Surface>();
    let surface = match surfaces_storage.get(*orbiting_id) {
        Some(surface) => surface,
        _ => return y,
    };

    y += 1;
    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Orbiting surface");
    y += 1;

    for _ in 0..surface.height {
        for sx in 0..surface.width {
            let fg = rltk::GREEN;
            let bg = rltk::GRAY;

            ctx.set(x + sx as i32, y, fg, bg, '#' as rltk::FontCharType);
        }

        y += 1;
    }

    y
}

enum LandMenuOption {
    Back,
    LandAt(i32, i32),
}

fn draw_land_menu(state: &mut State, ctx: &mut Rltk, info: LocalInfo) {
    // frame
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

    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Choose landing location");
    y += 2;

    // status
    y = draw_status(state, ctx, &info, x, y);
    // orbiting map
    y = draw_orbiting_map(state, ctx, &info, x, y);

    // draw options to land
    let mut options = vec![LandMenuOption::Back];
    let surfaces_storage = state.ecs.read_storage::<Surface>();
    let surface = match info
        .orbiting_id
        .as_ref()
        .and_then(|id| surfaces_storage.get(*id))
    {
        Some(surface) => surface,
        _ => {
            std::mem::drop(surfaces_storage);
            state.ecs.insert(CockpitWindowState::new(SubWindow::Main));
            return;
        }
    };

    y += 1;
    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Orbiting surface");
    y += 1;

    for sy in 0..surface.height {
        for sx in 0..surface.width {
            options.push(LandMenuOption::LandAt(sx as i32, sy as i32));
        }
    }

    for (i, option) in options.iter().enumerate() {
        let option_str = match option {
            LandMenuOption::Back => format!("{}) back", i),
            LandMenuOption::LandAt(x, y) => format!("{}) land at ({},{})", i, x, y),
        };

        ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, option_str);
        y += 1;
    }

    // process inputs
    match get_key_index(ctx.key).and_then(|index| options.get(index)) {
        Some(LandMenuOption::Back) => {
            std::mem::drop(surfaces_storage);
            state.ecs.insert(CockpitWindowState::new(SubWindow::Main))
        }
        Some(LandMenuOption::LandAt(x, y)) => {
            todo!()
        }
        _ => {}
    }
}

pub fn get_key_index(key: Option<VirtualKeyCode>) -> Option<usize> {
    match key {
        Some(VirtualKeyCode::Key0) => Some(0),
        Some(VirtualKeyCode::Key1) => Some(1),
        Some(VirtualKeyCode::Key2) => Some(2),
        Some(VirtualKeyCode::Key3) => Some(3),
        Some(VirtualKeyCode::Key4) => Some(4),
        Some(VirtualKeyCode::Key5) => Some(5),
        Some(VirtualKeyCode::Key6) => Some(6),
        Some(VirtualKeyCode::Key7) => Some(7),
        Some(VirtualKeyCode::Key8) => Some(8),
        Some(VirtualKeyCode::Key9) => Some(9),
        _ => None,
    }
}
