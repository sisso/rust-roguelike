use log::*;
use rltk::{a_star_search, BaseMap, GameState, RandomNumberGenerator, Rltk, VirtualKeyCode, RGB};
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

impl Position {
    fn get_at(&self, dir: Dir) -> Position {
        let mut p = self.clone();

        match dir {
            Dir::N => p.y -= 1,
            Dir::S => p.y += 1,
            Dir::W => p.x -= 1,
            Dir::E => p.x += 1,
        }

        p
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum Dir {
    N,
    S,
    W,
    E,
}

impl Dir {
    pub fn inv(&self) -> Self {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Dir::N => "n",
            Dir::S => "s",
            Dir::E => "e",
            Dir::W => "w",
        }
    }
}

#[derive(Component, Debug)]
struct Avatar {}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum TileType {
    Floor,
    Wall,
}

#[derive(Component, Debug, Clone)]
pub struct GMap {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
}

impl BaseMap for GMap {}

impl GMap {
    pub fn is_valid_xy(&self, x: i32, y: i32) -> bool {
        self.width > x && x >= 0 && self.height > y && y >= 0
    }

    pub fn is_valid(&self, index: Index) -> bool {
        index >= 0 && index < self.cells.len()
    }
}

#[derive(Component, Debug, Clone)]
pub struct Cell {
    index: Index,
    tile: TileType,
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
    fn create(total_cells: usize, default_tile: TileType) -> Vec<Cell> {
        let mut cells = vec![];
        // total random
        for index in 0..total_cells {
            cells.push(Cell {
                index,
                tile: default_tile,
            });
        }

        cells
    }

    fn apply_walls(map: &mut GMap) {
        for x in 0..map.width {
            map.cells[xy_idx(x, 0)].tile = TileType::Wall;
            map.cells[xy_idx(x, map.height - 1)].tile = TileType::Wall;
        }

        for y in 0..map.height {
            map.cells[xy_idx(0, y)].tile = TileType::Wall;
            map.cells[xy_idx(map.width - 1, y)].tile = TileType::Wall;
        }
    }

    let total_cells = (MAP_W * MAP_H) as usize;
    // let mut rng = rltk::RandomNumberGenerator::new();

    let mut gmap = GMap {
        width: MAP_W,
        height: MAP_H,
        cells: create(total_cells, TileType::Floor),
    };

    apply_walls(&mut gmap);

    gmap
}

fn draw_map(map: &GMap, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for cell in map.cells.iter() {
        match cell.tile {
            TileType::Floor => {
                ctx.set(x, y, rltk::LIGHT_GREEN, rltk::BLACK, rltk::to_cp437(' '));
            }
            TileType::Wall => {
                ctx.set(x, y, rltk::GREEN, rltk::BLACK, rltk::to_cp437('#'));
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

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Avatar>();
    let map = ecs.fetch::<GMap>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.cells[destination_idx].tile != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
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
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        match &ctx.key {
            Some(VirtualKeyCode::Return) => {
                debug!("generate a new map");
                self.ecs.insert(new_map());
            }
            _ => {}
        }

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
    }
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    env_logger::builder().filter(None, LevelFilter::Info).init();

    let context = RltkBuilder::simple80x50().with_title("Alien").build()?;
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Avatar>();

    gs.ecs.insert(new_map());
    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Avatar {})
        .build();

    rltk::main_loop(context, gs)
}

#[test]
fn test_idx_xy_and_xy_idx() {
    let index = xy_idx(3, 5);
    let coords = idx_xy(index);
    assert_eq!(coords, Position { x: 3, y: 5 });
}
