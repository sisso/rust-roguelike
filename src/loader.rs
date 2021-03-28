use crate::gmap::{Cell, GMap, TileType};
use crate::models::{ObjectsType, Position};
use crate::view::Renderable;
use rltk::RGB;
use specs::prelude::*;

pub fn parse_map_tiles(
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

pub fn map_empty(width: i32, height: i32) -> GMap {
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

    let total_cells = (width * height) as usize;
    // let mut rng = rltk::RandomNumberGenerator::new();

    let mut gmap = GMap {
        width: width,
        height: height,
        cells: create(total_cells, TileType::Floor),
    };

    apply_walls(&mut gmap);

    gmap
}

#[derive(Debug)]
pub struct ParseMapAst {
    pub width: i32,
    pub height: i32,
    pub cells: Vec<char>,
}

#[derive(Debug)]
pub enum ParseMapError {
    UnknownChar(char),
    FewLines,
    InvalidLineWidth(String),
}

/// All empty spaces are removed an can not be used
/// If first line is empty, is removed,
/// if last line is empty, is removed
pub fn parse_map(map: &str) -> Result<ParseMapAst, ParseMapError> {
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

pub fn parse_map_objects(ecs: &mut World, ast: ParseMapAst) -> Result<(), ParseMapError> {
    let mut changes: Vec<(Position, ObjectsType)> = vec![];
    {
        let cfg = ecs.fetch::<super::cfg::Cfg>();
        for (index, cell) in ast.cells.iter().enumerate() {
            let kind = cfg
                .raw_map_objects
                .iter()
                .find(|(ch, _tp)| ch == cell)
                .map(|(_, kind)| kind)
                .cloned();

            let kind = match kind {
                Some(k) => k,
                None => continue,
            };

            let pos = {
                let map = ecs.fetch::<GMap>();
                map.idx_xy(index)
            };

            changes.push((pos, kind));
        }
    }
    for (pos, kind) in changes {
        match kind {
            ObjectsType::Door { vertical } => {
                let icon = if vertical { '|' } else { '-' };
                ecs.create_entity()
                    .with(Position { x: pos.x, y: pos.y })
                    .with(Renderable {
                        glyph: rltk::to_cp437(icon),
                        fg: RGB::named(rltk::CYAN),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    })
                    .with(kind)
                    .build();
            }
            ObjectsType::Cockpit => {
                ecs.create_entity()
                    .with(Position { x: pos.x, y: pos.y })
                    .with(Renderable {
                        glyph: rltk::to_cp437('C'),
                        fg: RGB::named(rltk::BLUE),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    })
                    .with(kind)
                    .build();
            }
            ObjectsType::Engine => {
                ecs.create_entity()
                    .with(Position { x: pos.x, y: pos.y })
                    .with(Renderable {
                        glyph: rltk::to_cp437('E'),
                        fg: RGB::named(rltk::RED),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    })
                    .with(kind)
                    .build();
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::super::cfg;
    use super::*;

    #[test]
    fn test_idx_xy_and_xy_idx() {
        let index = xy_idx(cfg::SCREEN_W, 3, 5);
        let coords = idx_xy(cfg::SCREEN_W, index);
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
