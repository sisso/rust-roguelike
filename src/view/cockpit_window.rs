use crate::commons::grid::Dir;
use crate::commons::recti::RectI;
use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::models::SurfaceTileKind;
use crate::state::State;
use crate::view::window::Window;
use crate::{cfg, ship, view, Label, Location, Position, Sector, Ship, Surface, P2};
use hecs::{Entity, World};
use log::{info, warn};
use rltk::{BTerm, Rltk, VirtualKeyCode, RGB};

struct LocalInfo {
    pub avatar_id: Entity,
    pub grid_id: Entity,
    pub ship_id: Option<Entity>,
    pub orbiting_id: Option<Entity>,
}

impl LocalInfo {
    pub fn from(ecs: &World, avatar_id: Entity) -> LocalInfo {
        let pos = ecs.get::<&Position>(avatar_id).expect("position not found");
        let area = GridRef::find_area(ecs, pos.grid_id).expect("area not found");

        let layer_id = area
            .get_layer_entity_at(&pos.point)
            .expect("invalid entity at position");

        // check if layer id is a ship, and if it is orbiting a object
        let layer_is_ship = ecs.get::<&Ship>(layer_id).is_ok();
        let (ship_id, orbiting_id) = if layer_is_ship {
            let maybe_orbit_id = ecs.get::<&Location>(layer_id).ok().and_then(|i| match &*i {
                Location::Orbit { target_id } => Some(*target_id),
                _ => None,
            });
            (Some(layer_id), maybe_orbit_id)
        } else {
            (None, None)
        };

        LocalInfo {
            avatar_id,
            grid_id: pos.grid_id,
            ship_id,
            orbiting_id,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SubWindow {
    #[default]
    Main,
    Land {
        selected: P2,
    },
}

/// list of commands that a cockpit can show
#[derive(Clone, Debug)]
enum MenuOption {
    Land,
    FlyTo { target_id: Entity },
    Launch,
}

#[derive(Debug, Clone, Default)]
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

pub fn draw(state: &mut State, ctx: &mut Rltk, _cockpit_id: Entity, rect: RectI) {
    let info = LocalInfo::from(&state.ecs, state.player.get_avatar_id());

    match state.window_manage.cockpit_window.sub_window {
        SubWindow::Main => draw_main(state, ctx, info, rect),
        SubWindow::Land { .. } => draw_land_menu(
            state,
            ctx,
            rect,
            info.ship_id.expect("no ship id to show landing screen"),
            info.orbiting_id,
        ),
    }
}

fn draw_main(state: &mut State, ctx: &mut Rltk, info: LocalInfo, rect: RectI) {
    // frame
    ctx.draw_box(
        rect.get_x(),
        rect.get_y(),
        rect.get_width(),
        rect.get_height(),
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let x = rect.get_x() + 2;
    let mut y = rect.get_y() + 2;

    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "The cockpit");
    y += 2;

    // status
    let mut commands: Vec<MenuOption> = vec![];
    match &info.ship_id {
        Some(ship_id) => {
            // draw ship status
            y = draw_status(state, ctx, *ship_id, x, y);
            // sector
            y = draw_sector_map(state, ctx, x, y, *ship_id);
            // orbiting map
            y = draw_orbiting_map(state, ctx, *ship_id, x, y, None);
            // actions
            commands = list_commands(&state, *ship_id);
            y = draw_actions(state, ctx, x, y, &commands);
        }
        _ => {}
    }
    // draw messages
    y = draw_msg(state, ctx, 0, x, y);

    // process inputs
    let executed = match (ctx.key, get_key_index(ctx.key)) {
        (_, Some(index)) if info.ship_id.is_some() => {
            try_do_command(state, ctx, info.ship_id.unwrap(), commands.get(index))
        }
        (Some(VirtualKeyCode::Escape), _) => {
            state.window_manage.set_window(Window::World);
            Ok(())
        }
        _ => Ok(()),
    };

    match executed {
        Err(msg) => {
            state.window_manage.cockpit_window.last_msg = Some(msg);
        }
        _ => {}
    }
}

fn draw_msg(state: &State, ctx: &mut BTerm, border: i32, x: i32, y: i32) -> i32 {
    if let Some(msg) = &state.window_manage.cockpit_window.last_msg {
        ctx.print_color(x, cfg::SCREEN_H - border - 1, rltk::GRAY, rltk::RED, msg);
    }
    y
}

fn draw_actions(
    state: &State,
    ctx: &mut BTerm,
    x: i32,
    mut y: i32,
    commands: &Vec<MenuOption>,
) -> i32 {
    for (i, command) in commands.iter().enumerate() {
        let command_str = match command {
            MenuOption::Land => "land".to_string(),
            MenuOption::FlyTo { target_id } => {
                let label = state.ecs.get::<&Label>(*target_id);
                if let Ok(label) = label {
                    format!("fly to {}", label.name)
                } else {
                    "fly to unknown".to_string()
                }
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

fn draw_status(state: &State, ctx: &mut Rltk, ship_id: Entity, x: i32, mut y: i32) -> i32 {
    let mut unknown = || {
        ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Ship is the unknown");
        y + 1
    };

    let ship_command = match state.ecs.get::<&Ship>(ship_id) {
        Ok(ship) => ship.current_command.clone(),
        Err(_) => return unknown(),
    };

    let location = match state.ecs.get::<&Location>(ship_id) {
        Ok(l) => l,
        Err(_) => return unknown(),
    };

    y += 1;

    match (&*location, ship_command) {
        (Location::Sector { pos, .. }, ship::Command::FlyTo { .. }) => {
            ctx.print_color(
                x,
                y,
                rltk::GRAY,
                rltk::BLACK,
                format!("Ship at {:?} flying in space", pos),
            );
            y += 1;
        }
        (Location::Sector { pos: _, .. }, (ship::Command::Idle)) => {
            ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Ship is drifting in space");
            y += 1;
        }
        (Location::Orbit { target_id: _, .. }, _) => {
            ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Ship is orbiting a object");
            y += 1;
        }
        (Location::BodySurfacePlace { .. }, _) => {
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
        Some(MenuOption::Land) => {
            state.window_manage.cockpit_window = CockpitWindowState::new(SubWindow::Land {
                selected: P2::new(0, 0),
            });
        }

        Some(MenuOption::FlyTo { target_id }) => set_ship_command(
            &mut state.ecs,
            ship_id,
            ship::Command::FlyTo {
                target_id: *target_id,
            },
        ),

        Some(MenuOption::Launch) => {
            set_ship_command(&mut state.ecs, ship_id, ship::Command::Launch)
        }

        _ => {
            log::warn!("unknown command {:?}", command);
        }
    }
    Ok(())
}

/// return ne y value
fn draw_sector_map(state: &State, ctx: &mut Rltk, x: i32, y: i32, ship_id: Entity) -> i32 {
    let location = match state.ecs.get::<&Location>(ship_id) {
        Ok(l) => l,
        Err(_) => return y,
    };

    let (_ship_pos, ship_sector_id) = match &*location {
        Location::Sector { pos, sector_id } => (pos.clone(), *sector_id),
        Location::Orbit { target_id } => {
            match crate::locations::resolve_sector_pos(&state.ecs, *target_id) {
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
    let sector = state
        .ecs
        .get::<&Sector>(ship_sector_id)
        .expect("sector not found");

    for e in sector.bodies.iter().cloned() {
        let (pos, _) = match crate::locations::resolve_sector_pos(&state.ecs, e) {
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
    state: &State,
    ctx: &mut Rltk,
    ship_id: Entity,
    x: i32,
    mut y: i32,
    selected: Option<P2>,
) -> i32 {
    let orbiting_id = match state
        .ecs
        .get::<&Location>(ship_id)
        .ok()
        .and_then(|i| i.get_orbiting_body())
    {
        Some(id) => id,
        _ => return y,
    };

    let surface = state
        .ecs
        .get::<&Surface>(orbiting_id)
        .expect("surface not found");

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

            let ch = match surface.get_tile(sx, sy) {
                Some(SurfaceTileKind::Structure) => '$',
                _ => '#',
            };

            ctx.set(x + sx as i32, y, fg, bg, ch as rltk::FontCharType);
        }

        y += 1;
    }

    y
}

fn draw_land_menu(
    state: &mut State,
    ctx: &mut Rltk,
    rect: RectI,
    ship_id: Entity,
    orbiting_id: Option<Entity>,
) {
    let orbiting_id = match orbiting_id {
        Some(id) => id,
        None => return,
    };

    // frame
    ctx.draw_box(
        rect.get_x(),
        rect.get_y(),
        rect.get_width(),
        rect.get_height(),
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    let x = rect.get_x() + 2;
    let mut y = rect.get_y() + 2;

    ctx.print_color(x, y, rltk::GRAY, rltk::BLACK, "Choose landing location");
    y += 2;

    // status
    y = draw_status(state, ctx, ship_id, x, y);
    // orbiting map
    let place_coords = match &state.window_manage.cockpit_window.sub_window {
        SubWindow::Land { selected } => *selected,
        _ => panic!("unexpected subwindow"),
    };
    y = draw_orbiting_map(state, ctx, ship_id, x, y, Some(place_coords));

    // draw options to land
    let surface = match state.ecs.get::<&Surface>(orbiting_id).ok() {
        Some(surface) => surface,
        _ => {
            state.window_manage.cockpit_window = CockpitWindowState::new(SubWindow::Main);
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
            state.window_manage.cockpit_window = CockpitWindowState::new(SubWindow::Main);
        }
        (_, Some(index)) if index == 1 => {
            let selected_index =
                crate::commons::grid::coords_to_index(surface.width as i32, place_coords);
            let target_id = surface.zones[selected_index as usize];

            drop(surface);
            set_ship_command(
                &mut state.ecs,
                ship_id,
                ship::Command::Land {
                    target_id: target_id,
                    place_coords: place_coords,
                },
            );
            // reset cockipt window and close
            state.window_manage.cockpit_window = CockpitWindowState::new(SubWindow::Main);
            state.window_manage.set_window(Window::World);
        }
        (Some(VirtualKeyCode::Up), _) | (Some(VirtualKeyCode::Numpad8), _) => {
            drop(surface);
            set_selected_land_position(state, surface_size, place_coords, Dir::N)
        }
        (Some(VirtualKeyCode::Right), _) | (Some(VirtualKeyCode::Numpad6), _) => {
            drop(surface);
            set_selected_land_position(state, surface_size, place_coords, Dir::E)
        }
        (Some(VirtualKeyCode::Down), _) | (Some(VirtualKeyCode::Numpad2), _) => {
            drop(surface);
            set_selected_land_position(state, surface_size, place_coords, Dir::S)
        }
        (Some(VirtualKeyCode::Left), _) | (Some(VirtualKeyCode::Numpad4), _) => {
            drop(surface);
            set_selected_land_position(state, surface_size, place_coords, Dir::W)
        }
        _ => {}
    }
}

fn set_selected_land_position(state: &mut State, surface_size: P2, mut current: P2, dir: Dir) {
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

    state.window_manage.cockpit_window.sub_window = SubWindow::Land { selected: current };
}

fn set_ship_command(ecs: &mut World, ship_id: Entity, ship_command: ship::Command) {
    info!("update ship {:?} command to {:?}", ship_id, ship_command);
    let mut ship = ecs.get::<&mut Ship>(ship_id).unwrap();
    ship.current_command = ship_command;
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

fn list_commands(state: &State, ship_id: Entity) -> Vec<MenuOption> {
    let mut commands = vec![];

    // TODO: currently can be none when ship is landed
    let location = match state.ecs.get::<&Location>(ship_id).ok() {
        Some(loc) => loc,
        None => return commands,
    };

    match &*location {
        Location::Sector { sector_id, .. } => {
            let sector = state
                .ecs
                .get::<&Sector>(*sector_id)
                .expect("sector not found");
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
        Location::BodySurfacePlace { .. } => {
            commands.push(MenuOption::Launch);
        }
    }

    commands
}

pub fn draw_cockpit(state: &mut State, ctx: &mut Rltk, cockpit_id: Entity) {
    view::draw_default(state, ctx);
    draw(
        state,
        ctx,
        cockpit_id,
        RectI::new(2, 2, cfg::SCREEN_W - 5, cfg::SCREEN_H - 14),
    );
}
