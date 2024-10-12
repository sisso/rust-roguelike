use super::models::*;
use crate::commons;
use crate::commons::grid::{BaseGrid, Coord, Grid, NGrid};
use crate::commons::v2i::V2I;
use hecs::Entity;
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
    pub fn is_opaque(&self) -> bool {
        match self {
            Tile::Wall => true,
            Tile::OutOfMap => true,
            _ => false,
        }
    }

    pub fn is_nothing(&self) -> bool {
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
    layers: Vec<Entity>,
}

impl Area {
    pub fn new(grid: NGrid<Cell>, layers: Vec<Entity>) -> Self {
        Self { grid, layers }
    }

    pub fn from(id: Entity, grid: Grid<Cell>) -> Self {
        Self::new(NGrid::from_grid(grid), vec![id])
    }

    pub fn get_layer_entity_at(&self, coord: &Coord) -> Option<Entity> {
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

    pub fn get_layers(&self) -> &Vec<Entity> {
        &self.layers
    }

    pub fn remove_layer(&mut self, entity: Entity) -> Option<(Area, Coord)> {
        let index = self.layers.iter().position(|i| *i == entity)?;
        self.layers.remove(index);

        let pgrid = self.grid.remove(index);
        let gmap = Area::new(NGrid::from_grid(pgrid.grid), vec![entity]);
        Some((gmap, pgrid.pos))
    }

    pub fn move_entity(&mut self, id: Entity, from: Position, to: Position) {
        self.grid
            .get_mut_at(from.point)
            .objects
            .retain(|i| *i != id);
        self.grid.get_mut_at(to.point).objects.push(id);
    }
}

pub const EMPTY_CELL: Cell = Cell {
    tile: Tile::Space,
    objects: vec![],
};

#[derive(Debug, Clone)]
pub struct Cell {
    pub tile: Tile,
    pub objects: Vec<Entity>,
}

impl Cell {
    pub fn new(tile: Tile) -> Self {
        Cell {
            tile,
            ..Default::default()
        }
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
            .map(|i| i.tile.is_opaque())
            .unwrap_or(true)
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        self.get_8_neighbours(self.index_to_coords(idx as i32))
            .into_iter()
            .filter(|coord| !self.get_at(*coord).tile.is_opaque())
            .map(|coord| (self.coords_to_index(coord) as usize, 1.0))
            .collect()
    }

    fn get_pathing_distance(&self, id1: usize, id2: usize) -> f32 {
        let c1 = self.index_to_coords(id1 as i32);
        let c2 = self.index_to_coords(id2 as i32);
        c1.distance_sqr(c2)
    }
}
