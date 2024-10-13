use hecs::{Entity, World};

use crate::actions::EntityActions;
use crate::ai::Ai;
use crate::area::{Area, Cell, Tile};
use crate::cfg::Cfg;
use crate::combat::{CombatStats, Dice};
use crate::commons::grid::{Grid, NGrid};
use crate::commons::grid_string::ParseMapError;
use crate::commons::v2i::V2I;
use crate::commons::{grid_string, v2i};
use crate::gridref::GridRef;
use crate::health::Health;
use crate::inventory::Inventory;
use crate::item::Item;
use crate::mob::Mob;
use crate::models::{
    Avatar, Label, Location, ObjectsKind, Position, Sector, SectorBody, Surface, SurfaceTileKind,
    P2,
};
use crate::ship::Ship;
use crate::state::State;
use crate::team::Team;
use crate::view::Renderable;
use crate::visibility::{Visibility, VisibilityMemory};
use crate::{cfg, loader, sectors, ship, utils};
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
    let mut grid = Grid::new_square(size, || Cell::new(tile));

    for (pos, other) in buildings {
        grid.merge(pos, other);
    }

    let id = world.reserve_entity();

    let gmap = Area::from(id, grid);

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
                Team::Player,
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
                Visibility {
                    range: 16,
                    ..Default::default()
                },
                VisibilityMemory::default(),
                EntityActions {
                    available: vec![],
                    requested: None,
                },
                ObjectsKind::Player,
                Health {
                    hp: 10,
                    max_hp: 10,
                    ..Default::default()
                },
                CombatStats {
                    attack: Dice::new(2, 0),
                    defense: Dice::new(2, 0),
                    damage: Dice::new(1, 0),
                },
                Inventory::default(),
            ),
        )
        .unwrap();
}

pub fn create_mob(state: &mut State, position: Position) -> Entity {
    let id = state.ecs.spawn((
        Team::Mob,
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
        Visibility {
            visible_tiles: vec![],
            range: 14,
        },
        Mob::default(),
        Health {
            hp: 1,
            max_hp: 1,
            ..Default::default()
        },
        ObjectsKind::Mob,
        Ai::default(),
        CombatStats {
            attack: Dice::new(1, 0),
            defense: Dice::new(1, 0),
            damage: Dice::new(1, 0),
        },
    ));
    log::debug!("create mob {:?} at {:?}", id, position);
    id
}

pub fn new_grid_from_ast(map_ast: &MapAst) -> Grid<Cell> {
    let cells = map_ast.iter().map(|e| Cell::new(e.tile)).collect();
    Grid::new_from(map_ast.get_width(), map_ast.get_height(), cells)
}

#[derive(Debug, Clone)]
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
                    Label {
                        name: "door".to_string(),
                    },
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
                    Label {
                        name: "cockpit".to_string(),
                    },
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
                    Label {
                        name: "engine".to_string(),
                    },
                ));
            }
            Some(other) => {
                log::warn!("unknown cell kind {:?}", other);
            }
            None => {}
        }
    });

    Ok(())
}

// TODO: remove this hack that require cfg to clone
pub fn new_parser(cfg: Cfg) -> Box<dyn Fn(char) -> Option<MapAstCell>> {
    let f = move |ch| {
        let tile = cfg
            .map_parser
            .raw_map_tiles
            .iter()
            .find(|(c, tile)| *c == ch)
            .map(|(_, tile)| *tile)?;

        let obj = cfg
            .map_parser
            .raw_map_objects
            .iter()
            .find(|(c, tile)| *c == ch)
            .map(|(_, obj)| *obj);

        Some(MapAstCell {
            tile: tile,
            obj: obj,
        })
    };

    Box::new(f)
}

pub enum NewGameParams {
    Normal,
    Orbiting,
    Landed,
}

pub fn start_game(state: &mut State, params: &NewGameParams) {
    let parser = new_parser(state.cfg.clone());
    let ship_map_ast = grid_string::parse_map(parser, cfg::SHIP_MAP).expect("fail to load map");
    let ship_grid = new_grid_from_ast(&ship_map_ast);

    let parser = new_parser(state.cfg.clone());
    let house_ast = grid_string::parse_map(parser, cfg::HOUSE_MAP).expect("fail to load house map");
    let house_grid = new_grid_from_ast(&house_ast);

    let spawn_x = ship_grid.get_width() / 2 - 5;
    let spawn_y = ship_grid.get_height() / 2;

    // load scenery
    let sector_id = create_sector(&mut state.ecs);
    log::debug!("sector id {:?}", sector_id);

    let zone_size = 100;

    let mut planets_zones: Vec<(Entity, SurfaceTileKind)> = (0..3)
        .map(|i| create_planet_zone(&mut state.ecs, i, zone_size, Tile::Ground))
        .map(|e| (e, SurfaceTileKind::Plain))
        .collect();

    let house_pos = V2I::new(zone_size / 2 + 30, zone_size / 2);
    let planet_zone_house_grid_id = create_planet_zone_from(
        &mut state.ecs,
        0,
        100,
        Tile::Ground,
        vec![(house_pos, &house_grid)],
    );
    planets_zones.insert(0, (planet_zone_house_grid_id, SurfaceTileKind::Structure));

    log::debug!("planet zones id {:?}", planets_zones);

    let planet_id = create_planet(
        &mut state.ecs,
        "Planet X",
        Location::Sector {
            sector_id,
            pos: P2::new(5, 0),
        },
        planets_zones.clone(),
        2,
    );
    log::debug!("planet id {:?}", planet_id);

    let ship_location = match params {
        NewGameParams::Normal => Location::Sector {
            sector_id: sector_id,
            pos: P2::new(0, 0),
        },
        NewGameParams::Orbiting | NewGameParams::Landed => Location::Orbit {
            target_id: planet_id,
        },
    };

    let ship_id = create_ship(
        &mut state.ecs,
        "ship",
        Ship {
            current_command: ship::Command::Idle,
            move_calm_down: 0,
        },
        ship_location,
        NGrid::from_grid(ship_grid),
    );
    log::debug!("ship id {:?}", ship_id);

    let avatar_entity_id = create_avatar(
        &mut state.ecs,
        state.player.get_avatar_id(),
        Position {
            grid_id: ship_id,
            point: (spawn_x, spawn_y).into(),
        },
    );
    log::info!("avatar id: {:?}", avatar_entity_id);

    // load objects
    parse_map_objects(&mut state.ecs, v2i::ZERO, ship_id, ship_map_ast)
        .expect("fail to load map objects");
    parse_map_objects(
        &mut state.ecs,
        house_pos,
        planet_zone_house_grid_id,
        house_ast,
    )
    .expect("fail to load map objects");

    // spawn custom objects
    create_mob(
        state,
        Position {
            grid_id: planet_zone_house_grid_id,
            point: house_pos + V2I::new(-10, 0),
        },
    );

    create_item(
        state,
        "gold",
        Position::new(planet_zone_house_grid_id, house_pos + V2I::new(-15, 0)),
    );

    // extra changes
    match params {
        NewGameParams::Landed => {
            let surface_id = planets_zones[0].0;
            let landing_coords = V2I::new(zone_size / 2, zone_size / 2);

            ship::systems::do_ship_landing(
                &mut state.ecs,
                ship_id,
                surface_id,
                landing_coords,
                None,
            );
        }
        _ => {}
    }

    sectors::update_bodies_list(&mut state.ecs);
    utils::reindex_grids_objects(&mut state.ecs);
}

pub fn create_item(state: &mut State, label: &str, pos: Position) {
    let id = state.ecs.spawn((
        Item::default(),
        pos,
        Label {
            name: label.to_string(),
        },
        ObjectsKind::Item,
        Renderable {
            glyph: rltk::to_cp437('i'),
            fg: RGB::named(rltk::BLUE),
            bg: RGB::named(rltk::BLACK),
            priority: 0,
        },
    ));
    log::debug!("create item {:?} {:?}", id, label)
}
