use log::*;
use rltk::{Algorithm2D, BaseMap, GameState, RandomNumberGenerator, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::*;
use std::cmp::{max, min};
use std::collections::HashSet;

const SHIP_MAP: &str = r"
___________________________
___________________________
___________________________
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
_______####________________
___________________________
___________________________
___________________________
";

const SCREEN_W: i32 = 80;
const SCREEN_H: i32 = 50;

type Index = usize;

#[derive(Component, Debug)]
struct Cfg {
    raw_map_tiles: Vec<(char, TileType)>,
    raw_map_objects: Vec<(char, ObjectsType)>,
}

impl Cfg {
    pub fn new() -> Self {
        let raw_map_tiles: Vec<(char, TileType)> = vec![
            ('_', TileType::Space),
            ('.', TileType::Floor),
            ('#', TileType::Wall),
            ('E', TileType::Wall),
            ('-', TileType::Floor),
            ('|', TileType::Floor),
            ('@', TileType::Floor),
            ('!', TileType::Floor),
        ];

        let raw_map_objects: Vec<(char, ObjectsType)> = vec![
            ('E', ObjectsType::Engine),
            ('-', ObjectsType::Door { vertical: false }),
            ('|', ObjectsType::Door { vertical: true }),
            ('@', ObjectsType::Cockpit),
            ('!', ObjectsType::Door { vertical: true }),
        ];
        Cfg {
            raw_map_tiles,
            raw_map_objects,
        }
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub know_tiles: HashSet<rltk::Point>,
    pub range: i32,
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
    priority: i32,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum TileType {
    Floor,
    Wall,
    Space,
}

#[derive(PartialEq, Copy, Clone, Debug)]
enum ObjectsType {
    Door { vertical: bool },
    Engine,
    Cockpit,
}

#[derive(Component, Debug, Clone)]
pub struct GMap {
    width: i32,
    height: i32,
    cells: Vec<Cell>,
}

impl Algorithm2D for GMap {
    fn dimensions(&self) -> rltk::Point {
        rltk::Point::new(self.width, self.height)
    }
}

impl BaseMap for GMap {
    fn is_opaque(&self, idx: usize) -> bool {
        self.cells[idx].tile == TileType::Wall
    }
}
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

    for (_y, line) in lines.iter().enumerate() {
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
    legend: &Vec<(char, TileType)>,
    map: &ParseMapAst,
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

fn draw_map(
    visible_cells: &Vec<rltk::Point>,
    know_cells: &HashSet<rltk::Point>,
    map: &GMap,
    ctx: &mut Rltk,
) {
    let mut y = 0;
    let mut x = 0;

    for cell in &map.cells {
        // calculate real tile
        let (mut fg, mut bg, mut ch) = match cell.tile {
            TileType::Floor => (rltk::LIGHT_GREEN, rltk::BLACK, '.'),
            TileType::Wall => (rltk::GREEN, rltk::BLACK, '#'),
            TileType::Space => (rltk::BLACK, rltk::BLACK, ' '),
        };

        // replace non visible tiles
        if visible_cells
            .iter()
            .find(|p| p.x == x && p.y == y)
            .is_none()
        {
            if know_cells.contains(&rltk::Point { x, y }) {
                // if is know
                fg = rltk::GRAY;
            } else {
                // unknown
                fg = rltk::BLACK;
                bg = rltk::BLACK;
                ch = ' ';
            }
        }

        ctx.set(x, y, fg, bg, ch as rltk::FontCharType);

        // Move the coordinates
        x += 1;
        if x >= map.width {
            x = 0;
            y += 1;
        }
    }
}

fn draw_objects(visible_cells: &Vec<rltk::Point>, ecs: &World, ctx: &mut Rltk) {
    let positions = ecs.read_storage::<Position>();
    let renderables = ecs.read_storage::<Renderable>();
    let mut objects = (&positions, &renderables).join().collect::<Vec<_>>();
    objects.sort_by(|&a, &b| a.1.priority.cmp(&b.1.priority));
    for (pos, render) in objects {
        if visible_cells
            .iter()
            .find(|p| p.x == pos.x && p.y == pos.y)
            .is_some()
        {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
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

fn parse_map_objects(gs: &mut State, ast: ParseMapAst) -> Result<(), ParseMapError> {
    let mut changes: Vec<(Position, ObjectsType)> = vec![];
    {
        let cfg = gs.ecs.fetch::<Cfg>();
        for (index, cell) in ast.cells.iter().enumerate() {
            let kind = cfg
                .raw_map_objects
                .iter()
                .find(|(ch, tp)| ch == cell)
                .map(|(_, kind)| kind)
                .cloned();

            let kind = match kind {
                Some(k) => k,
                None => continue,
            };

            let pos = {
                let map = gs.ecs.fetch::<GMap>();
                map.idx_xy(index)
            };

            changes.push((pos, kind));
        }
    }
    for (pos, kind) in changes {
        match kind {
            ObjectsType::Door { vertical } => {
                let icon = if vertical { '|' } else { '-' };
                gs.ecs
                    .create_entity()
                    .with(Position { x: pos.x, y: pos.y })
                    .with(Renderable {
                        glyph: rltk::to_cp437(icon),
                        fg: RGB::named(rltk::CYAN),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    })
                    .build();
            }
            ObjectsType::Cockpit => {
                gs.ecs
                    .create_entity()
                    .with(Position { x: pos.x, y: pos.y })
                    .with(Renderable {
                        glyph: rltk::to_cp437('C'),
                        fg: RGB::named(rltk::BLUE),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    })
                    .build();
            }
            ObjectsType::Engine => {
                gs.ecs
                    .create_entity()
                    .with(Position { x: pos.x, y: pos.y })
                    .with(Renderable {
                        glyph: rltk::to_cp437('E'),
                        fg: RGB::named(rltk::RED),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    })
                    .build();
            }
        }
    }

    Ok(())
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
            _ => {}
        }

        {
            // merge all visible and know tiles from player
            let viewshed = self.ecs.read_storage::<Viewshed>();
            let avatars = self.ecs.read_storage::<Avatar>();
            let views = (&viewshed, &avatars).join().collect::<Vec<_>>();
            let (v, _) = views.iter().next().unwrap();

            // draw
            let map = self.ecs.fetch::<GMap>();
            draw_map(&v.visible_tiles, &v.know_tiles, &map, ctx);
            draw_objects(&v.visible_tiles, &self.ecs, ctx);
        }

        {
            let mouse_pos = ctx.mouse_pos();
            ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::MAGENTA));
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        ReadExpect<'a, GMap>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (map, mut viewshed, pos): Self::SystemData) {
        for (viewshed, pos) in (&mut viewshed, &pos).join() {
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles =
                rltk::field_of_view(rltk::Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed
                .visible_tiles
                .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

            for pos in &viewshed.visible_tiles {
                viewshed.know_tiles.insert(*pos);
            }
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let cfg = Cfg::new();
    env_logger::builder().filter(None, LevelFilter::Info).init();

    let context = RltkBuilder::simple80x50().with_title("Alien").build()?;
    let mut gs = State { ecs: World::new() };
    gs.ecs.register::<Cfg>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Avatar>();
    gs.ecs.register::<Viewshed>();

    let map_ast = parse_map(SHIP_MAP).expect("fail to load map");
    let map = parse_map_tiles(&cfg.raw_map_tiles, &&map_ast).expect("fail to load map tiles");

    let spawn_x = map.width / 2;
    let spawn_y = map.height / 2;

    gs.ecs.insert(map);
    gs.ecs.insert(cfg);
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
            priority: 1,
        })
        .with(Avatar {})
        .with(Viewshed {
            visible_tiles: vec![],
            know_tiles: HashSet::new(),
            range: 16,
        })
        .build();

    parse_map_objects(&mut gs, map_ast).expect("fail to load map objects");

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
        let RAW_MAP_TILES = get_parse_map_default_legend();
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
        let RAW_MAP_TILES = vec![
            ('_', TileType::Space),
            ('.', TileType::Floor),
            ('#', TileType::Wall),
            ('E', TileType::Wall),
        ];

        RAW_MAP_TILES
    }
}
