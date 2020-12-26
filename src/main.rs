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

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum TileType {
    Plain,
    Forest,
    Mountain,
    Water,
    City,
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

    fn apply_random(cells: &mut Vec<Cell>, rng: &mut RandomNumberGenerator) {
        // total random
        for cell in cells.iter_mut() {
            let tile = match rng.range(0, 5) {
                0 => TileType::Plain,
                1 => TileType::Water,
                2 => TileType::Forest,
                3 => TileType::Mountain,
                4 => TileType::City,
                other => panic!(format!("invalid random range {}", other)),
            };

            cell.tile = tile;
        }
    }

    fn add_city(rng: &mut RandomNumberGenerator, map: &mut GMap, pos: Position, size: u16) {
        let mut added = 0;
        let mut cursor = pos;

        map.cells[xy_idx(cursor.x, cursor.y)].tile = TileType::City;

        for _ in 0..size {
            let new_cursor = match rng.range(0, 4) {
                0 => cursor.get_at(Dir::N),
                1 => cursor.get_at(Dir::S),
                2 => cursor.get_at(Dir::E),
                3 => cursor.get_at(Dir::W),
                _ => panic!("invalid random number"),
            };

            if map.is_valid_xy(new_cursor.x, new_cursor.y) {
                let index = xy_idx(new_cursor.x, new_cursor.y);

                cursor = new_cursor;
                map.cells[index].tile = TileType::City;
                added += 1;
            }
        }
    }

    fn add_random_cities(rng: &mut RandomNumberGenerator, map: &mut GMap, min: u16, max: u16) {
        let amount = rng.range(min, max + 1);
        for _ in 0..amount {
            let pos = Position {
                x: rng.range(0, map.width),
                y: rng.range(0, map.height),
            };

            let size = rng.range(1, 8);

            add_city(rng, map, pos, size);
        }
    }

    let total_cells = (MAP_W * MAP_H) as usize;
    let mut rng = rltk::RandomNumberGenerator::new();

    let mut gmap = GMap {
        width: MAP_W,
        height: MAP_H,
        cells: create(total_cells, TileType::Plain),
    };

    add_random_cities(&mut rng, &mut gmap, 2, 8);

    gmap
}

fn draw_map(map: &GMap, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for cell in map.cells.iter() {
        match cell.tile {
            TileType::Plain => {
                ctx.set(x, y, rltk::LIGHT_GREEN, rltk::BLACK, rltk::to_cp437('.'));
            }
            TileType::Forest => {
                ctx.set(x, y, rltk::GREEN, rltk::BLACK, rltk::to_cp437('F'));
            }
            TileType::Water => {
                ctx.set(x, y, rltk::BLUE, rltk::BLACK, rltk::to_cp437('~'));
            }
            TileType::Mountain => {
                ctx.set(x, y, rltk::WHITE, rltk::BLACK, rltk::to_cp437('M'));
            }
            TileType::City => {
                ctx.set(x, y, rltk::GRAY, rltk::BLACK, rltk::to_cp437('#'));
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

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

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

    gs.ecs.insert(new_map());

    rltk::main_loop(context, gs)
}

#[test]
fn test_idx_xy_and_xy_idx() {
    let index = xy_idx(3, 5);
    let coords = idx_xy(index);
    assert_eq!(coords, Position { x: 3, y: 5 });
}
