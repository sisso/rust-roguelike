use crate::view::window::Window;
use crate::{
    cfg, ship, Dir, Label, Location, Player, Position, Sector, Ship, State,
    Surface, P2,
};
use log::{info, warn};
use rltk::{BTerm, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::*;


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
    Land { selected: P2 },
}

/// list of commands that a cockpit can show
#[derive(Clone, Debug)]
enum MenuOption {
    Land,
    FlyTo { target_id: Entity },
    Launch,
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
        SubWindow::Land { .. } => draw_land_menu(state, ctx, info),
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

    let x = border + 2;
    let mut y = border + 2;

    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "The cockpit");
    y += 2;

    // status
    y = draw_status(state, ctx, &info, x, y);
    // sector
    y = draw_sector_map(state, ctx, x, y, info.ship_id);
    // orbiting map
    y = draw_orbiting_map(state, ctx, &info, x, y, None);
    // actions
    let commands = list_commands(&state.ecs, info.ship_id);
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
    commands: &Vec<MenuOption>,
) -> i32 {
    let labels = state.ecs.read_storage::<Label>();
    for (i, command) in commands.iter().enumerate() {
        let command_str = match command {
            MenuOption::Land => "land".to_string(),
            MenuOption::FlyTo { target_id } => {
                let label = labels.get(*target_id);
                let name = label.map(|i| i.name.as_str()).unwrap_or("unknown");
                format!("fly to {}", name)
            }
            MenuOption::Launch => "launch".to_string(),
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
        (Some(Location::Sector { pos: _, .. }), Some(ship::Command::Idle)) => {
            ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Ship is drifting in space");
            y += 1;
        }
        (Some(Location::Orbit { target_id: _, .. }), _) => {
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
    _ctx: &mut Rltk,
    ship_id: Entity,
    command: Option<&MenuOption>,
) -> Result<(), String> {
    match command {
        Some(MenuOption::Land) => state.ecs.insert(CockpitWindowState::new(SubWindow::Land {
            selected: P2::new(0, 0),
        })),

        Some(MenuOption::FlyTo { target_id }) => set_ship_command(
            &mut state.ecs,
            ship_id,
            ship::Command::FlyTo {
                target_id: *target_id,
            },
        ),
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
    let (_ship_pos, ship_sector_id) = match locations.get(ship_id) {
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

    for (e, _loc, _lab) in (&entities, &locations, &labels).join() {
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
    selected: Option<P2>,
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

    for sy in 0..surface.height {
        for sx in 0..surface.width {
            let mut fg = rltk::GREEN;
            let bg = rltk::GRAY;

            if Some(P2::new(sx as i32, sy as i32)) == selected {
                fg = rltk::BLUE;
            }

            ctx.set(x + sx as i32, y, fg, bg, '#' as rltk::FontCharType);
        }

        y += 1;
    }

    y
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

    let x = border + 2;
    let mut y = border + 2;

    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Choose landing location");
    y += 2;

    // status
    y = draw_status(state, ctx, &info, x, y);
    // orbiting map
    let selected = match state.ecs.fetch::<CockpitWindowState>().sub_window {
        SubWindow::Land { selected } => selected,
        _ => panic!("unexpected subwindow"),
    };
    y = draw_orbiting_map(state, ctx, &info, x, y, Some(selected));

    // draw options to land
    let surfaces_storage = state.ecs.read_storage::<Surface>();
    let orbiting_id = match info.orbiting_id {
        Some(id) => id,
        None => return,
    };
    let surface = match surfaces_storage.get(orbiting_id) {
        Some(surface) => surface,
        _ => {
            std::mem::drop(surfaces_storage);
            state.ecs.insert(CockpitWindowState::new(SubWindow::Main));
            return;
        }
    };

    let surface_size = P2::new(surface.width as i32, surface.height as i32);

    y += 1;
    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Orbiting surface");
    y += 2;
    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "0) back");
    y += 1;
    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "1) land");
    y += 1;

    // process inputs
    match (ctx.key, get_key_index(ctx.key)) {
        (_, Some(index)) if index == 0 => {
            std::mem::drop(surfaces_storage);
            state.ecs.insert(CockpitWindowState::new(SubWindow::Main))
        }
        (_, Some(index)) if index == 1 => {
            let selected_index =
                crate::commons::grid::coords_to_index(surface.width as i32, &selected);
            let target_id = surface.zones[selected_index as usize];

            std::mem::drop(surfaces_storage);
            set_ship_command(
                &mut state.ecs,
                info.ship_id,
                ship::Command::Land {
                    target_id: target_id,
                    pos: P2::new(0, 0),
                },
            );
            state.ecs.insert(CockpitWindowState::new(SubWindow::Main))
        }
        (Some(VirtualKeyCode::Up), _) => {
            std::mem::drop(surfaces_storage);
            set_selected_land_position(&mut state.ecs, surface_size, selected, Dir::N)
        }
        (Some(VirtualKeyCode::Right), _) => {
            std::mem::drop(surfaces_storage);
            set_selected_land_position(&mut state.ecs, surface_size, selected, Dir::E)
        }
        (Some(VirtualKeyCode::Down), _) => {
            std::mem::drop(surfaces_storage);
            set_selected_land_position(&mut state.ecs, surface_size, selected, Dir::S)
        }
        (Some(VirtualKeyCode::Left), _) => {
            std::mem::drop(surfaces_storage);
            set_selected_land_position(&mut state.ecs, surface_size, selected, Dir::W)
        }

        _ => {}
    }
}

fn set_selected_land_position(ecs: &mut World, surface_size: P2, mut current: P2, dir: Dir) {
    let v = dir.as_vec();
    current.x += v.0;
    current.y += v.1;

    if current.x < 0 {
        current.x = surface_size.x;
    }
    if current.y < 0 {
        current.y = surface_size.y;
    }
    if current.x >= surface_size.x {
        current.x = 0;
    }
    if current.y >= surface_size.y {
        current.y = 0;
    }

    ecs.fetch_mut::<CockpitWindowState>().sub_window = SubWindow::Land { selected: current };
}

fn set_ship_command(ecs: &mut World, ship_id: Entity, ship_command: ship::Command) {
    info!("update ship {:?} command to {:?}", ship_id, ship_command);
    ecs.write_storage::<Ship>()
        .get_mut(ship_id)
        .unwrap()
        .current_command = ship_command;
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

fn list_commands(ecs: &World, ship_id: Entity) -> Vec<MenuOption> {
    let locations = ecs.read_storage::<Location>();
    let sectors = ecs.read_storage::<Sector>();

    // currently can be none when ship is landed TODO: ok ugly
    let location = match locations.get(ship_id) {
        Some(loc) => loc,
        None => {
            return vec![];
        }
    };

    let mut commands = vec![];

    match location {
        Location::Sector { sector_id, .. } => {
            let sector = sectors.get(*sector_id).unwrap();
            for body_id in &sector.bodies {
                if *body_id == ship_id {
                    continue;
                }

                commands.push(MenuOption::FlyTo {
                    target_id: *body_id,
                });
            }
        }
        Location::Orbit { .. } => {
            commands.push(MenuOption::Land);
        }
        Location::BodySurface { .. } => {
            commands.push(MenuOption::Launch);
        }
        Location::BodySurfacePlace { .. } => {}
    }

    commands
}
