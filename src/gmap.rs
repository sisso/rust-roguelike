use super::models::*;
use crate::commons;
use crate::commons::grid::{Coord, Grid, NGrid};
use crate::commons::v2i::V2I;
use specs::prelude::*;
use specs_derive::*;
use std::ops::Index;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GMapTile {
    Ground,
    Floor,
    Wall,
    Space,
    // normally used by returns to avoid option
    OutOfMap,
}

impl GMapTile {
    pub fn is_opaque(&self) -> bool {
        match self {
            GMapTile::Wall => true,
            GMapTile::OutOfMap => true,
            _ => false,
        }
    }

    pub fn is_nothing(&self) -> bool {
        match self {
            GMapTile::Space => true,
            GMapTile::OutOfMap => true,
            _ => false,
        }
    }
}

impl Default for GMapTile {
    fn default() -> Self {
        GMapTile::Space
    }
}

type LayerId = usize;

#[derive(Debug, Clone)]
pub struct GMap {
    grid: NGrid<Cell>,
    layers: Vec<Entity>,
}

impl GMap {
    pub fn new(grid: NGrid<Cell>, layers: Vec<Entity>) -> Self {
        Self { grid, layers }
    }
    pub fn get_layer_entity_at(&self, coord: &Coord) -> Option<Entity> {
        self.grid
            .get_layer(coord)
            .and_then(|index| self.layers.get(index).cloned())
    }
    pub fn get_grid(&self) -> &NGrid<Cell> {
        &self.grid
    }
    pub fn merge(&mut self, gmap: GMap, pos: &P2) {
        self.grid.merge(gmap.grid, pos);
        self.layers.extend(gmap.layers.into_iter());
    }

    pub fn get_layers(&self) -> &Vec<Entity> {
        &self.layers
    }

    pub fn remove_layer(&mut self, entity: Entity) -> Option<(GMap, Coord)> {
        let index = self.layers.iter().position(|i| *i == entity)?;
        self.layers.remove(index);

        let pgrid = self.grid.remove(index);
        let gmap = GMap::new(NGrid::from_grid(pgrid.grid), vec![entity]);
        Some((gmap, pgrid.pos))
    }
}

impl rltk::Algorithm2D for GMap {
    fn dimensions(&self) -> rltk::Point {
        let size = self.grid.get_size();
        rltk::Point::new(size.x, size.y)
    }

    fn in_bounds(&self, pos: rltk::Point) -> bool {
        self.grid.is_valid(&V2I::new(pos.x, pos.y))
    }
}

impl rltk::BaseMap for GMap {
    fn is_opaque(&self, idx: usize) -> bool {
        let w = self.grid.get_width();
        let c = commons::grid::index_to_coord(w, idx as i32);
        self.grid
            .get_at(&c)
            .map(|i| i.tile.is_opaque())
            .unwrap_or(true)
    }
}

struct ViewGrid<'a> {
    grids: Vec<(Entity, P2, &'a GMap)>,
}

impl<'a> ViewGrid<'a> {
    pub fn create_view(
        _locations: ReadStorage<'a, Location>,
        _gmaps: ReadStorage<'a, Location>,
        _entity: Entity,
    ) -> ViewGrid<'a> {
        todo!()
    }
}

pub const EMPTY_CELL: Cell = Cell {
    tile: GMapTile::Space,
};

#[derive(Component, Debug, Clone, Default)]
pub struct Cell {
    pub tile: GMapTile,
    // pub objects? // how will return ref?
}

impl Cell {
    pub fn new(tile: GMapTile) -> Self {
        Cell { tile }
    }
}

impl commons::grid::GridCell for Cell {
    fn is_empty(&self) -> bool {
        self.tile.is_nothing()
    }
}

impl Default for &Cell {
    fn default() -> Self {
        &EMPTY_CELL
    }
}

// impl From<Grid<Cell>> for GMap {
//     fn from(g: Grid<Cell>) -> Self {
//         GMap {
//             grid: NGrid::from_grid(g),
//             layers: vec![],
//         }
//     }
// }
