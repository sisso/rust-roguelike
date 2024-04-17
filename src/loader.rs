use std::collections::HashSet;

use crate::actions::EntityActions;
use crate::area::{Area, Cell, Tile};
use crate::cfg::MapParserCfg;
use crate::commons::grid::{Grid, NGrid};
use crate::commons::grid_string::ParseMapError;
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
