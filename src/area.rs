use super::models::*;
use crate::combat::CombatStats;
use crate::commons;
use crate::commons::grid::{BaseGrid, Coord, Grid, NGrid};
use crate::commons::v2i::V2I;
use crate::gridref::AreaId;
use crate::team::Team;
use hecs::{Entity, World};
use rltk::SmallVec;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Tile {
    Ground,
    Floor,
    Wall,
    Space,
    // works like None
    OutOfMap,
}

impl Tile {
    fn is_opaque(&self) -> bool {
        match self {
            Tile::Wall => true,
            Tile::OutOfMap => true,
            _ => false,
        }
    }

    fn is_walkable(&self) -> bool {
        match self {
            Tile::Wall => false,
            Tile::OutOfMap => false,
            _ => true,
        }
    }

    fn is_nothing(&self) -> bool {
        match self {
            Tile::Space => true,
            Tile::OutOfMap => true,
            _ => false,
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Space
    }
}

/// It is a collection of many Grid layered on top of each other, this work as a "cached" version
/// of all grids living on same area one on top of other
///
#[derive(Debug, Clone)]
pub struct Area {
    /// grids on this map, the index must match with layers
    grid: NGrid<Cell>,
    /// entities that own on each grid in this map
    layers: Vec<AreaId>,
}

impl Area {}

impl Area {
    pub fn new(grid: NGrid<Cell>, layers: Vec<AreaId>) -> Self {
        Self { grid, layers }
    }

    pub fn from(id: Entity, grid: Grid<Cell>) -> Self {
        Self::new(NGrid::from_grid(grid), vec![id])
    }

    pub fn get_layer_entity_at(&self, coord: &Coord) -> Option<AreaId> {
        self.grid
            .get_layer(coord)
            .and_then(|index| self.layers.get(index).cloned())
    }
    pub fn get_grid(&self) -> &NGrid<Cell> {
        &self.grid
    }

    pub fn get_grid_mut(&mut self) -> &mut NGrid<Cell> {
        &mut self.grid
    }

    pub fn merge(&mut self, gmap: Area, pos: &P2) {
        self.grid.merge(gmap.grid, pos);
        self.layers.extend(gmap.layers.into_iter());
    }

    pub fn contains_layer(&self, grid_id: AreaId) -> bool {
        self.layers.contains(&grid_id)
    }

    pub fn get_layers(&self) -> &Vec<Entity> {
        &self.layers
    }

    pub fn remove_layer(&mut self, entity: AreaId) -> Option<(Area, Coord)> {
        let index = self.layers.iter().position(|i| *i == entity)?;
        self.layers.remove(index);

        let pgrid = self.grid.remove(index);
        let gmap = Area::new(NGrid::from_grid(pgrid.grid), vec![entity]);
        Some((gmap, pgrid.pos))
    }

    pub fn move_object(&mut self, id: Entity, from: Position, to: Position) {
        let be = self.remove_object(id, from.point);
        self.grid.get_mut_at(to.point).objects.push(be);
    }

    pub fn list_objects_at(&self, point: V2I) -> &Vec<BasicEntity> {
        let tile = self.get_grid().get_at(point);
        &tile.objects
    }

    pub fn clear_objects(&mut self) {
        for grid in self.grid.get_layers_mut() {
            for i in 0..grid.grid.len() {
                grid.grid.get_mut(i as i32).clear_objects();
            }
        }
    }

    pub fn add_object(&mut self, point: V2I, basic_entity: BasicEntity) {
        self.grid.get_mut_at(point).add_object(basic_entity);
    }

    pub fn remove_object(&mut self, id: Entity, point: Coord) -> BasicEntity {
        let cell = self.grid.get_mut_at(point);
        let objects = &mut cell.objects;
        let index = objects
            .iter()
            .position(|i| i.id == id)
            .unwrap_or_else(|| panic!("{:?} not found on previous position {:?}", id, point));
        objects.swap_remove(index)
    }
}

pub static EMPTY_CELL: Cell = Cell {
    tile: Tile::Space,
    objects: vec![],
    is_walkable: true,
    is_opaque: false,
};

pub static EMPTY_BASIC_ENTITY_VEC: Vec<BasicEntity> = vec![];

#[derive(Debug, Clone)]
pub struct Cell {
    tile: Tile,
    objects: Vec<BasicEntity>,
    is_walkable: bool,
    is_opaque: bool,
}

impl Cell {
    pub fn new(tile: Tile) -> Self {
        let mut cell = Cell {
            tile,
            ..Default::default()
        };
        cell.resolve();
        cell
    }

    fn resolve(&mut self) {
        self.is_walkable = self.tile.is_walkable();
        self.is_opaque = self.tile.is_opaque();

        for be in &self.objects {
            if !be.kind.is_walkable() {
                self.is_walkable = false;
            }
            if be.kind.is_opaque() {
                self.is_opaque = true;
            }
        }
    }

    fn add_object(&mut self, be: BasicEntity) {
        self.objects.push(be);
        self.resolve();
    }

    pub fn clear_objects(&mut self) {
        self.objects.clear();
        self.resolve();
    }

    pub fn find_enemies_of(&self, world: &World, attacker_team: Team) -> Vec<BasicEntity> {
        self.objects
            .iter()
            .filter(|be| world.get::<&CombatStats>(be.id).is_ok())
            .filter(|be| {
                world
                    .get::<&Team>(be.id)
                    .map(|team| team.is_enemy(attacker_team))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    pub fn is_opaque(&self) -> bool {
        self.is_opaque
    }

    pub fn is_walkable(&self) -> bool {
        self.is_walkable
    }

    pub fn tile(&self) -> Tile {
        self.tile
    }

    pub fn objects(&self) -> &Vec<BasicEntity> {
        &self.objects
    }
}

impl commons::grid::GridCell for Cell {
    fn is_empty(&self) -> bool {
        self.tile.is_nothing()
    }
}

impl Default for Cell {
    fn default() -> Self {
        EMPTY_CELL.clone()
    }
}

impl Default for &Cell {
    fn default() -> Self {
        &EMPTY_CELL
    }
}

impl rltk::Algorithm2D for NGrid<Cell> {
    fn dimensions(&self) -> rltk::Point {
        self.get_size().into_rlk_point()
    }

    fn in_bounds(&self, pos: rltk::Point) -> bool {
        self.is_valid_coords(V2I::from(pos))
    }
}

impl rltk::BaseMap for NGrid<Cell> {
    fn is_opaque(&self, idx: usize) -> bool {
        let coords = self.index_to_coords(idx as i32);
        self.get_at_opt(coords)
            .map(|i| i.is_opaque())
            .unwrap_or(true)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        self.get_8_neighbours(self.index_to_coords(idx as i32))
            .into_iter()
            .filter(|coord| self.get_at(*coord).is_walkable())
            .map(|coord| (self.coords_to_index(coord) as usize, 1.0))
            .collect()
    }

    fn get_pathing_distance(&self, id1: usize, id2: usize) -> f32 {
        let c1 = self.index_to_coords(id1 as i32);
        let c2 = self.index_to_coords(id2 as i32);
        c1.distance_sqr(c2)
    }
}
