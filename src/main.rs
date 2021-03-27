use log::*;
use rltk::{BaseMap, GameState, RandomNumberGenerator, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::*;
use std::cmp::{max, min};

const SCREEN_W: i32 = 80;
const SCREEN_H: i32 = 50;

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
    Space,
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
        self.width as i32 > x && x >= 0 && self.height as i32 > y && y >= 0
    }

    pub fn is_valid(&self, index: Index) -> bool {
        index < self.cells.len()
    }

    fn xy_idx(&self, x: i32, y: i32) -> usize {
        xy_idx((self.width) as i32, x, y)
    }

    fn idx_xy(&self, index: Index) -> Position {
        idx_xy((self.width) as i32, index)
    }
}

fn xy_idx(width: i32, x: i32, y: i32) -> usize {
    ((y * width as i32) + x) as usize
}

fn idx_xy(width: i32, index: Index) -> Position {
    Position {
        x: index as i32 % width as i32,
        y: index as i32 / width as i32,
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

#[derive(Debug)]
struct ParseMapAst {
    width: i32,
    height: i32,
    cells: Vec<char>,
}

#[derive(Debug)]
enum ParseMapError {
    UnknownChar(char),
    FewLines,
    InvalidLineWidth(String),
}

/// All empty spaces are removed an can not be used
/// If first line is empty, is removed,
/// if last line is empty, is removed
fn parse_map(map: &str) -> Result<ParseMapAst, ParseMapError> {
    let mut lines: Vec<String> = map.split("\n").map(|line| line.replace(" ", "")).collect();

    if lines.is_empty() {
        return Err(ParseMapError::FewLines);
    }

    if lines[0].is_empty() {
        lines.remove(0);
    }

    if lines.is_empty() {
        return Err(ParseMapError::FewLines);
    }

    if lines[lines.len() - 1].is_empty() {
        lines.remove(lines.len() - 1);
    }

    let width = lines[0].len() as i32;
    let height = lines.len() as i32;
    let mut cells = vec![];

    for (y, line) in lines.iter().enumerate() {
        if line.len() != width as usize {
            return Err(ParseMapError::InvalidLineWidth(line.clone()));
        }

        for ch in line.chars() {
            cells.push(ch)
        }
    }

    Ok(ParseMapAst {
        width,
        height,
        cells: cells,
    })
}

fn parse_map_tiles(
    map: &ParseMapAst,
    legend: &Vec<(char, TileType)>,
) -> Result<GMap, ParseMapError> {
    let mut gmap = GMap {
        width: map.width,
        height: map.height,
        cells: vec![],
    };

    for i in 0..(map.width as usize * map.height as usize) {
        let ch = map.cells[i];
        let tile = match legend.iter().find(|(c, _)| c == &ch).map(|(_, tile)| tile) {
            Some(t) => t,
            None => return Err(ParseMapError::UnknownChar(ch)),
        };

        gmap.cells.push(Cell {
            index: i,
            tile: tile.clone(),
        })
    }

    Ok(gmap)
}

fn map_ship() -> GMap {
    let raw = r"
_______####________________
_______EEE#________________
_______##.#________________
________#.#________________
______###-####-#######_____
______#.....#...#....!_____
______#.@...|...#....!_____
______#.....#...|....!_____
______###-############_____
________#.#________________
_______##.#________________
_______EEE#________________
_______####________________"
        .trim();

    let legend = vec![
        ('_', TileType::Space),
        ('.', TileType::Floor),
        ('#', TileType::Wall),
        ('E', TileType::Wall),
        ('-', TileType::Floor),
        ('|', TileType::Floor),
        ('@', TileType::Floor),
        ('!', TileType::Floor),
    ];

    let ast = parse_map(raw).expect("fail to parse map");
    parse_map_tiles(&ast, &legend).expect("fail to parse map")
}

fn map_empty() -> GMap {
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
        for x in 0..(map.width as i32) {
            let i = map.xy_idx(x, 0);
            map.cells[i].tile = TileType::Wall;
            let i = map.xy_idx(x, map.height - 1);
            map.cells[i].tile = TileType::Wall;
        }

        for y in 0..(map.height as i32) {
            let i = map.xy_idx(0, y);
            map.cells[i].tile = TileType::Wall;
            let i = map.xy_idx(map.width - 1, y);
            map.cells[i].tile = TileType::Wall;
        }
    }

    let total_cells = (SCREEN_W * SCREEN_H) as usize;
    // let mut rng = rltk::RandomNumberGenerator::new();

    let mut gmap = GMap {
        width: SCREEN_W,
        height: SCREEN_H,
        cells: create(total_cells, TileType::Floor),
    };

    apply_walls(&mut gmap);

    gmap
}

fn draw_map(map: &GMap, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for cell in &map.cells {
        match cell.tile {
            TileType::Floor => {
                ctx.set(x, y, rltk::LIGHT_GREEN, rltk::BLACK, rltk::to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, rltk::GREEN, rltk::BLACK, rltk::to_cp437('#'));
            }
            TileType::Space => {
                ctx.set(x, y, rltk::BLACK, rltk::BLACK, rltk::to_cp437(' '));
            }
        }

        // Move the coordinates
        x += 1;
        if x >= map.width {
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
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.cells[destination_idx].tile != TileType::Wall {
            pos.x = min(map.width - 1, max(0, pos.x + delta_x));
            pos.y = min(map.height - 1, max(0, pos.y + delta_y));
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
            VirtualKeyCode::Numpad7 => try_move_player(-1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad8 => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad9 => try_move_player(1, -1, &mut gs.ecs),
            VirtualKeyCode::Numpad4 => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad5 => try_move_player(0, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad6 => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Numpad1 => try_move_player(-1, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad2 => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::Numpad3 => try_move_player(1, 1, &mut gs.ecs),
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
                self.ecs.insert(map_empty());
            }
            Some(key) => {
                println!("{:?}", key);
            }
            None => {}
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

    let map = map_ship();
    let spawn_x = map.width / 2;
    let spawn_y = map.height / 2;

    gs.ecs.insert(map);
    gs.ecs
        .create_entity()
        .with(Position {
            x: spawn_x,
            y: spawn_y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Avatar {})
        .build();

    rltk::main_loop(context, gs)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_idx_xy_and_xy_idx() {
        let index = xy_idx(SCREEN_W, 3, 5);
        let coords = idx_xy(SCREEN_W, index);
        assert_eq!(coords, Position { x: 3, y: 5 });
    }

    #[test]
    fn test_parse_map_should_find_the_map_dimension() {
        let map = parse_map(
            r"
            #....#
            ______ 
            #....#
            ",
        )
        .expect("fail to parse map");
        assert_eq!(map.width, 6);
        assert_eq!(map.height, 3);
    }

    #[test]
    fn test_parse_map_should_fail_for_invalid_maps() {
        let legend = get_parse_map_default_legend();
        parse_map(
            r"
            ###
            # #
            #
            
        ",
        )
        .err()
        .expect("map didnt fail");
    }

    fn get_parse_map_default_legend() -> Vec<(char, TileType)> {
        let legend = vec![
            ('_', TileType::Space),
            ('.', TileType::Floor),
            ('#', TileType::Wall),
            ('E', TileType::Wall),
        ];

        legend
    }
}
