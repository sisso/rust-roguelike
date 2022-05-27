use crate::{commons, Grid};

use crate::commons::v2i::V2I;
use crate::gmap::{Cell, GMap, GMapTile};
use crate::models::{ObjectsType, Position};
use crate::view::Renderable;
use rltk::{Algorithm2D, RGB};
use specs::prelude::*;

pub fn parse_map_tiles(
    legend: &Vec<(char, GMapTile)>,
    map: &ParseMapAst,
) -> Result<Grid<Cell>, ParseMapError> {
    let mut cells = vec![];

    for i in 0..(map.width as usize * map.height as usize) {
        let ch = map.cells[i];
        let tile = match legend.iter().find(|(c, _)| c == &ch).map(|(_, tile)| tile) {
            Some(t) => t,
            None => return Err(ParseMapError::UnknownChar(ch)),
        };

        cells.push(Cell { tile: tile.clone() })
    }

    let grid = commons::grid::Grid {
        width: map.width,
        height: map.height,
        list: cells,
    };

    Ok(grid)
}

// pub fn map_empty(width: i32, height: i32) -> GMap {
//     fn create(total_cells: usize, default_tile: GMapTile) -> Vec<Cell> {
//         let mut cells = vec![];
//         // total random
//         for _ in 0..total_cells {
//             cells.push(Cell { tile: default_tile });
//         }
//
//         cells
//     }
//
//     fn apply_walls(map: &mut GMap) {
//         for x in 0..(map.width as i32) {
//             let i = map.point2d_to_index((x, 0).into());
//             map.cells[i].tile = GMapTile::Wall;
//             let i = map.point2d_to_index((x, map.height - 1).into());
//             map.cells[i].tile = GMapTile::Wall;
//         }
//
//         for y in 0..(map.height as i32) {
//             let i = map.point2d_to_index((0, y).into());
//             map.cells[i].tile = GMapTile::Wall;
//             let i = map.point2d_to_index((map.width - 1, y).into());
//             map.cells[i].tile = GMapTile::Wall;
//         }
//     }
//
//     let total_cells = (width * height) as usize;
//     // let mut rng = rltk::RandomNumberGenerator::new();
//
//     let mut gmap: GMap = Grid {
//         width: width,
//         height: height,
//         list: create(total_cells, GMapTile::Floor),
//     }
//     .into();
//
//     apply_walls(&mut gmap);
//
//     gmap
// }

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

pub fn parse_map_objects(
    ecs: &mut World,
    grid_id: Entity,
    ast: ParseMapAst,
) -> Result<(), ParseMapError> {
    let mut changes: Vec<(Position, ObjectsType)> = vec![];
    {
        let cfg = ecs.fetch::<super::cfg::Cfg>();
        let grids = &ecs.read_storage::<GMap>();
        let map = grids.get(grid_id).unwrap();
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

            let pos = map.index_to_point2d(index);

            changes.push((
                Position {
                    grid_id: grid_id,
                    point: V2I::new(pos.x, pos.y),
                },
                kind,
            ));
        }
    }

    for (pos, kind) in changes {
        match kind {
            ObjectsType::Door { vertical } => {
                let icon = if vertical { '|' } else { '-' };
                ecs.create_entity()
                    .with(pos)
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
                    .with(pos)
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
                    .with(pos)
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
    use super::*;

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
}
