use rltk::{a_star_search, BaseMap, GameState, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::*;
use std::cmp::{max, min};

const MAP_W: i32 = 80;
const MAP_H: i32 = 50;

type Index = usize;

#[derive(Component, Clone, Debug, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component, Debug)]
struct Player {}

#[derive(Component, Debug, Clone)]
struct Mob {}

#[derive(PartialEq, Copy, Clone, Debug)]
enum TileType {
    Wall,
    Floor,
}

#[derive(Component, Debug, Clone)]
pub struct GMap {
    cells: Vec<Cell>,
}

impl BaseMap for GMap {}

#[derive(Component, Debug, Clone)]
pub struct Cell {
    tile: TileType,
    mark_to_dig: bool,
}

#[derive(Component, PartialEq, Clone, Debug)]
enum AiCommand {
    None,
    Dig,
}

// #[derive(Component, PartialEq, Clone, Debug)]
// enum AiState {
//     Idle,
//     Dig {
//         pos: Option<Position>,
//         path: Vec<Position>,
//     },
// }

#[derive(Component, Clone, Debug)]
struct AiDig {
    pos: Option<Position>,
    path: Vec<Position>,
}

struct State {
    ecs: World,
}

fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * MAP_W as usize) + x as usize
}

fn idx_xy(index: Index) -> Position {
    Position {
        x: index as i32 % MAP_W,
        y: index as i32 / MAP_W,
    }
}

fn new_map() -> GMap {
    let total_cells = (MAP_W * MAP_H) as usize;
    let mut cells = vec![
        Cell {
            tile: TileType::Wall,
            mark_to_dig: false
        };
        total_cells
    ];

    // Make the boundaries walls
    // for x in 0..80 {
    //     map[xy_idx(x, 0)] = TileType::Wall;
    //     map[xy_idx(x, 49)] = TileType::Wall;
    // }
    // for y in 0..50 {
    //     map[xy_idx(0, y)] = TileType::Wall;
    //     map[xy_idx(79, y)] = TileType::Wall;
    // }

    // Now we'll randomly splat a bunch of walls. It won't be pretty, but it's a decent illustration.
    // First, obtain the thread-local RNG:
    let mut rng = rltk::RandomNumberGenerator::new();

    // for _i in 0..400 {
    //     let x = rng.roll_dice(1, 79);
    //     let y = rng.roll_dice(1, 49);
    //     let idx = xy_idx(x, y);
    //     if idx != xy_idx(40, 25) {
    //         map[idx] = TileType::Wall;
    //     }
    // }

    for x in MAP_W - 4..MAP_W {
        for y in 0..MAP_H {
            cells[xy_idx(x, y)].tile = TileType::Floor;
        }
    }

    GMap { cells: cells }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<GMap>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.cells[destination_idx].tile != TileType::Wall {
            pos.x = min(MAP_W - 1, max(0, pos.x + delta_x));
            pos.y = min(MAP_H - 1, max(0, pos.y + delta_y));
        }
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        },
    }

    if ctx.left_click {
        let mut map = gs.ecs.fetch_mut::<GMap>();

        let (x, y) = ctx.mouse_pos();

        let index = xy_idx(x, y);

        if map.cells[index].tile == TileType::Wall {
            map.cells[index].mark_to_dig = true; // !map.cells[index].mark_to_dig;
        }
    }
}

fn draw_map(map: &GMap, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for cell in map.cells.iter() {
        match cell.tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall if cell.mark_to_dig => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.2, 0.7, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('#'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::from_f32(0., 0., 0.),
                    rltk::to_cp437('#'),
                );
            }
        }

        // Move the coordinates
        x += 1;
        if x >= MAP_W {
            x = 0;
            y += 1;
        }
    }
}

fn run_ai_dig(state: &mut State) {
    let entities = state.ecs.entities();
    let positions = state.ecs.read_storage::<Position>();
    let commands = state.ecs.read_storage::<AiCommand>();
    let mut dig_states = state.ecs.write_storage::<AiDig>();
    let gmap = &state.ecs.fetch::<GMap>();

    // let mut changes = vec![];

    for (e, pos, command, dig_state) in (&entities, &positions, &commands, &mut dig_states).join() {
        if dig_state.pos.is_none() {
            let target = gmap
                .cells
                .iter()
                .enumerate()
                .find(|(i, c)| c.mark_to_dig)
                .map(|(i, c)| i);

            if let Some(index) = target {
                // find path to the target
                if let Some(path) = search_path(&gmap, pos, &idx_xy(index)) {}
            } else {
                // no target found
            }
        }
    }

    // let commands = &mut state.ecs.write_storage::<AiCommand>();
    // let states = &mut state.ecs.write_storage::<AiState>();
    //
    // for (e, command, state) in changes {
    //     if let Some(command) = command {
    //         println!("{:?} to {:?}", e, command);
    //         commands.insert(e, command);
    //     }
    //
    //     if let Some(state) = state {
    //         println!("{:?} to {:?}", e, state);
    //         states.insert(e, state);
    //     }
    // }
}

fn search_path(gmap: &GMap, from: &Position, to: &Position) -> Option<Vec<Position>> {
    a_star_search(from, to, gmap);
    unimplemented!()
}

fn run_ai(state: &mut State) {
    let entities = state.ecs.entities();
    let commands = state.ecs.read_storage::<AiCommand>();
    let dig_state = &mut state.ecs.write_storage::<AiDig>();

    for (e, command) in (&entities, &commands).join() {
        match command {
            AiCommand::None => {
                if dig_state.contains(e) {
                    dig_state.remove(e);
                }
            }
            AiCommand::Dig => {
                if !dig_state.contains(e) {
                    dig_state.insert(
                        e,
                        AiDig {
                            pos: None,
                            path: vec![],
                        },
                    );
                }
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        {
            let map = self.ecs.fetch::<GMap>();
            draw_map(&map, ctx);
        }

        {
            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();

            for (pos, render) in (&positions, &renderables).join() {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }

            let mouse_pos = ctx.mouse_pos();
            ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
        }

        run_ai(self)
    }
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50().with_title("Dugeon").build()?;
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Mob>();
    // gs.ecs.register::<AiState>();
    gs.ecs.register::<AiCommand>();
    gs.ecs.register::<AiDig>();

    gs.ecs.insert(new_map());

    gs.ecs
        .create_entity()
        .with(Position {
            x: MAP_W - 2,
            y: MAP_H / 2,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    gs.ecs
        .create_entity()
        .with(Position {
            x: MAP_W - 2,
            y: MAP_H / 2 + 1,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('i'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Mob {})
        .with(AiCommand::Dig)
        .build();

    rltk::main_loop(context, gs)
}

#[test]
fn test_idx_xy_and_xy_idx() {
    let index = xy_idx(3, 5);
    let coords = idx_xy(index);
    assert_eq!(coords, Position { x: 3, y: 5 });
}
