use hecs::{Entity, World};

use crate::actions::EntityActions;
use crate::area::{Area, Cell, Tile};
use crate::commons::grid::{Grid, NGrid};
use crate::commons::grid_string::ParseMapError;
use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::health::Health;
use crate::mob::Mob;
use crate::models::{
    Avatar, Label, Location, ObjectsKind, Position, Sector, SectorBody, Surface, SurfaceTileKind,
};
use crate::ship::Ship;
use crate::state::State;
use crate::view::{Renderable, Viewshed};
use rltk::RGB;

pub fn create_sector(world: &mut World) -> Entity {
    world.spawn((Sector::default(),))
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

    let id = world.reserve_entity();

    let gmap = Area::new(NGrid::from_grid(grid), vec![id]);

    world.spawn_at(
        id,
        (
            Label {
                name: format!("zone {}", index),
            },
            GridRef::GMap(gmap),
        ),
    );

    id
}

pub fn create_planet(
    world: &mut World,
    label: &str,
    location: Location,
    zones: Vec<(Entity, SurfaceTileKind)>,
    width_and_height: i32,
) -> Entity {
    assert_eq!(width_and_height * width_and_height, zones.len() as i32);

    world.spawn((
        SectorBody::Planet,
        location,
        Label {
            name: label.to_string(),
        },
        Surface {
            width: width_and_height,
            height: width_and_height,
            tiles: zones.iter().map(|(_, t)| *t).collect(),
            zones: zones.iter().map(|(e, _)| *e).collect(),
        },
    ))
}

pub fn create_ship(
    world: &mut World,
    label: &str,
    ship: Ship,
    location: Location,
    ship_grid: NGrid<Cell>,
) -> Entity {
    let ship_id = world.reserve_entity();
    let ship_gmap = Area::new(ship_grid, vec![ship_id]);

    world.spawn_at(
        ship_id,
        (
            Label {
                name: label.to_string(),
            },
            ship,
            location,
            GridRef::GMap(ship_gmap),
        ),
    );

    ship_id
}

pub fn create_avatar(world: &mut World, avatar_id: Entity, position: Position) {
    world
        .insert(
            avatar_id,
            (
                Avatar {},
                Label {
                    name: "player".to_string(),
                },
                position,
                Renderable {
                    glyph: rltk::to_cp437('@'),
                    fg: RGB::named(rltk::YELLOW),
                    bg: RGB::named(rltk::BLACK),
                    priority: 1,
                },
                Viewshed {
                    visible_tiles: vec![],
                    know_tiles: Default::default(),
                    range: 16,
                },
                EntityActions {
                    available: vec![],
                    requested: None,
                },
                ObjectsKind::Player,
            ),
        )
        .unwrap();
}

pub fn create_mob(state: &mut State, position: Position) -> Entity {
    state.ecs.spawn((
        Avatar {},
        Label {
            name: "mob".to_string(),
        },
        position,
        Renderable {
            glyph: rltk::to_cp437('m'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            priority: 1,
        },
        Viewshed {
            visible_tiles: vec![],
            know_tiles: Default::default(),
            range: 16,
        },
        Mob {},
        Health {
            hp: 1,
            ..Default::default()
        },
        ObjectsKind::Mob,
    ))
}

pub fn new_grid_from_ast(map_ast: &MapAst) -> Grid<Cell> {
    let cells = map_ast.iter().map(|e| Cell { tile: e.tile }).collect();
    Grid::new_from(map_ast.get_width(), map_ast.get_height(), cells)
}

pub struct MapAstCell {
    pub tile: Tile,
    pub obj: Option<ObjectsKind>,
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
            Some(ObjectsKind::Door { vertical }) => {
                let icon = if vertical { '|' } else { '-' };
                ecs.spawn((
                    pos,
                    Renderable {
                        glyph: rltk::to_cp437(icon),
                        fg: RGB::named(rltk::CYAN),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    },
                    ObjectsKind::Door { vertical },
                ));
            }
            Some(ObjectsKind::Cockpit) => {
                ecs.spawn((
                    pos,
                    Renderable {
                        glyph: rltk::to_cp437('C'),
                        fg: RGB::named(rltk::BLUE),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    },
                    ObjectsKind::Cockpit,
                ));
            }
            Some(ObjectsKind::Engine) => {
                ecs.spawn((
                    pos,
                    Renderable {
                        glyph: rltk::to_cp437('E'),
                        fg: RGB::named(rltk::RED),
                        bg: RGB::named(rltk::BLACK),
                        priority: 0,
                    },
                    ObjectsKind::Engine,
                ));
            }
            other => {
                log::warn!("unknown cell kind {:?}", other);
            }
        }
    });

    Ok(())
}
