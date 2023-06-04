use crate::{area, commons, Grid};
use std::collections::HashSet;

use crate::actions::EntityActions;
use crate::area::{Area, Cell, Tile};
use crate::commons::grid::{GridCell, NGrid, PGrid};
use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::models::{
    Avatar, Label, Location, ObjectsType, Position, Sector, SectorBody, Surface, SurfaceTileKind,
};
use crate::ship::Ship;
use crate::view::{Renderable, Viewshed};
use rltk::{Algorithm2D, RGB};
use specs::prelude::*;

pub fn create_sector(world: &mut World) -> Entity {
    world.create_entity().with(Sector::default()).build()
}

pub fn create_planet_zone(world: &mut World, index: usize, size: i32, tile: Tile) -> Entity {
    create_planet_zone_from(world, index, size, tile, vec![])
}

pub fn create_planet_zone_from(
    world: &mut World,
    index: usize,
    size: i32,
    tile: Tile,
    buildings: Vec<(V2I, &Grid<Cell>)>,
) -> Entity {
    let mut grid = Grid::new_square(size, || Cell { tile });

    for (pos, other) in buildings {
        grid.merge(pos, other);
    }

    let builder = world.create_entity();
    let gmap = Area::new(NGrid::from_grid(grid), vec![builder.entity]);
    let zone_id = builder
        .with(Label {
            name: format!("zone {}", index),
        })
        .with(GridRef::GMap(gmap))
        .build();

    zone_id
}

pub fn create_planet(
    world: &mut World,
    label: &str,
    location: Location,
    zones: Vec<(Entity, SurfaceTileKind)>,
    width_and_height: i32,
) -> Entity {
    assert_eq!(width_and_height * width_and_height, zones.len() as i32);

    world
        .create_entity()
        .with(SectorBody::Planet)
        .with(location)
        .with(Label {
            name: label.to_string(),
        })
        .with(Surface {
            width: width_and_height,
            height: width_and_height,
            tiles: zones.iter().map(|(_, t)| *t).collect(),
            zones: zones.iter().map(|(e, _)| *e).collect(),
        })
        .build()
}

pub fn create_ship(
    world: &mut World,
    label: &str,
    ship: Ship,
    location: Location,
    ship_grid: NGrid<Cell>,
) -> Entity {
    let builder = world.create_entity();

    let ship_id = builder.entity;
    let ship_gmap = Area::new(ship_grid, vec![ship_id]);

    builder
        .with(Label {
            name: label.to_string(),
        })
        .with(ship)
        .with(location)
        .with(GridRef::GMap(ship_gmap))
        .build();

    ship_id
}

pub fn create_avatar(world: &mut World, position: Position) -> Entity {
    world
        .create_entity()
        .with(Avatar {})
        .with(Label {
            name: "player".to_string(),
        })
        .with(position)
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            priority: 1,
        })
        .with(Viewshed {
            visible_tiles: vec![],
            know_tiles: HashSet::new(),
            range: 16,
        })
        .with(EntityActions {
            actions: vec![],
            current: None,
        })
        .build()
}

pub fn parse_map_tiles(
    legend: &Vec<(char, Tile)>,
    map: &ParseMapAst,
) -> Result<Grid<Cell>, ParseMapError> {
    let mut cells = vec![];

    for i in 0..(map.get_width() * map.get_height()) {
        let ch = *map.get(i);
        let tile = match legend.iter().find(|(c, _)| c == &ch).map(|(_, tile)| tile) {
            Some(t) => t,
            None => return Err(ParseMapError::UnknownChar(ch)),
        };

        cells.push(Cell { tile: tile.clone() })
    }

    let grid = Grid::new_from(map.get_width(), map.get_height(), cells);
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

pub type ParseMapAst = Grid<char>;

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

    let grid = ParseMapAst::new_from(width, height, cells);
    Ok(grid)
}

pub fn parse_map_objects(
    ecs: &mut World,
    pos: V2I,
    grid_id: Entity,
    ast: ParseMapAst,
) -> Result<(), ParseMapError> {
    let mut changes: Vec<(Position, ObjectsType)> = vec![];
    {
        let cfg = ecs.fetch::<super::cfg::Cfg>();
        for (index, cell) in ast.iter() {
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

            let mut local_pos = ast.index_to_coords(index);
            local_pos.x += pos.x;
            local_pos.y += pos.y;

            changes.push((
                Position {
                    grid_id: grid_id,
                    point: V2I::new(local_pos.x, local_pos.y),
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
        assert_eq!(map.get_width(), 6);
        assert_eq!(map.get_height(), 3);
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
