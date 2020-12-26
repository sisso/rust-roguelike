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
    let total_cells = (MAP_W * MAP_H) as usize;
    let mut cells = vec![];

    let mut rng = rltk::RandomNumberGenerator::new();

    for index in 0..total_cells {
        let tile = match rng.range(0, 5) {
            0 => TileType::Plain,
            1 => TileType::Water,
            2 => TileType::Forest,
            3 => TileType::Mountain,
            4 => TileType::City,
            other => panic!(format!("invalid random range {}", other)),
        };

        cells.push(Cell { index, tile: tile });
    }

    GMap {
        width: MAP_W,
        height: MAP_H,
        cells: cells,
    }
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

        // match cell.tile {
        //     TileType::Plain => {
        //         ctx.set(x, y, rltk::GRAY, rltk::LIGHT_GREEN, rltk::to_cp437('.'));
        //     }
        //     TileType::Forest => {
        //         ctx.set(x, y, rltk::GRAY, rltk::GREEN, rltk::to_cp437('F'));
        //     }
        //     TileType::Water => {
        //         ctx.set(x, y, rltk::GRAY, rltk::BLUE, rltk::to_cp437('~'));
        //     }
        //     TileType::Mountain => {
        //         ctx.set(x, y, rltk::GRAY, rltk::WHITE, rltk::to_cp437('M'));
        //     }
        //     TileType::City => {
        //         ctx.set(x, y, rltk::GRAY, rltk::GRAY, rltk::to_cp437('#'));
        //     }
        // }

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
