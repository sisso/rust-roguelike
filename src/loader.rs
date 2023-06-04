use std::collections::HashSet;

use crate::actions::EntityActions;
use crate::area::{Area, Cell, Tile};
use crate::cfg::MapParserCfg;
use crate::commons::grid::{Grid, NGrid};
use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::models::{
    Avatar, Label, Location, ObjectsType, Position, Sector, SectorBody, Surface, SurfaceTileKind,
};
use crate::ship::Ship;
use crate::view::{Renderable, Viewshed};
use rltk::RGB;
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

pub fn new_grid_from_ast(map_ast: &MapAst) -> Grid<Cell> {
    let cells = map_ast.iter().map(|e| Cell { tile: e.tile }).collect();
    Grid::new_from(map_ast.get_width(), map_ast.get_height(), cells)
}

pub struct MapAstCell {
    pub tile: Tile,
    pub obj: Option<ObjectsType>,
}

pub type MapAst = Grid<MapAstCell>;
pub type RawMapAst = Grid<char>;

#[derive(Debug)]
pub enum ParseMapError {
    UnknownChar(char),
    FewLines,
    InvalidLineWidth(String),
}

pub fn parse_map(cfg: &MapParserCfg, map: &str) -> Result<MapAst, ParseMapError> {
    let raw = parse_map_str(map)?;
    parse_rawmap(cfg, &raw)
}

fn parse_rawmap(cfg: &MapParserCfg, map: &RawMapAst) -> Result<MapAst, ParseMapError> {
    let mut cells = Vec::with_capacity(map.len());

    for ch in map.iter() {
        let tile = match cfg.raw_map_tiles.iter().find(|(c, _)| c == ch) {
            Some((_, tile)) => *tile,
            None => return Err(ParseMapError::UnknownChar(*ch)),
        };

        let obj = match cfg.raw_map_objects.iter().find(|(c, _)| c == ch) {
            Some((_, obj)) => Some(*obj),
            None => None,
        };

        let cell = MapAstCell { tile, obj };

        cells.push(cell);
    }

    let grid = Grid::new_from(map.get_width(), map.get_height(), cells);
    Ok(grid)
}

/// All empty spaces are removed an can not be used
/// If first line is empty, is removed,
/// if last line is empty, is removed
fn parse_map_str(map: &str) -> Result<RawMapAst, ParseMapError> {
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
    let mut cells = Vec::with_capacity((width * height) as usize);

    for (_y, line) in lines.iter().enumerate() {
        if line.len() != width as usize {
            return Err(ParseMapError::InvalidLineWidth(line.clone()));
        }

        for ch in line.chars() {
            cells.push(ch)
        }
    }

    let grid = RawMapAst::new_from(width, height, cells);
    Ok(grid)
}

pub fn parse_map_objects(
    ecs: &mut World,
    source_pos: V2I,
    grid_id: Entity,
    map_ast: MapAst,
) -> Result<(), ParseMapError> {
    map_ast.iter().enumerate().for_each(|(index, c)| {
        let mut local_pos = map_ast.index_to_coords(index as i32);
        local_pos.x += source_pos.x;
        local_pos.y += source_pos.y;

        let pos = Position {
            grid_id,
            point: local_pos,
        };

        match c.obj {
            Some(ObjectsType::Door { vertical }) => {
                let icon = if vertical { '|' } else { '-' };
                ecs.create_entity()
                    .with(pos)
                    .with(Renderable {
                        glyph: rltk::to_cp437(icon),
                        fg: RGB::named(rltk::CYAN),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    })
                    .with(ObjectsType::Door { vertical })
                    .build();
            }
            Some(ObjectsType::Cockpit) => {
                ecs.create_entity()
                    .with(pos)
                    .with(Renderable {
                        glyph: rltk::to_cp437('C'),
                        fg: RGB::named(rltk::BLUE),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    })
                    .with(ObjectsType::Cockpit)
                    .build();
            }
            Some(ObjectsType::Engine) => {
                ecs.create_entity()
                    .with(pos)
                    .with(Renderable {
                        glyph: rltk::to_cp437('E'),
                        fg: RGB::named(rltk::RED),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    })
                    .with(ObjectsType::Engine)
                    .build();
            }
            None => {}
        }
    });

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_map_should_find_the_map_dimension() {
        let map = parse_map_str(
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
        parse_map_str(
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
