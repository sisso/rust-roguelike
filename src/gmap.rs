use super::models::*;
use crate::commons;
use crate::commons::grid::{Grid, NGrid};
use crate::commons::v2i::V2I;
use specs::prelude::*;
use specs_derive::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GMapTile {
    Ground,
    Floor,
    Wall,
    Space,
    // normally used by returns to avoid option, TODO: remove?
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

#[derive(Component, Debug, Clone)]
pub struct GMap {
    grid: NGrid<Cell>,
    layers: Vec<Entity>,
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

impl GMap {
    pub fn get_grid(&self) -> &NGrid<Cell> {
        &self.grid
    }
    pub fn new(grid: NGrid<Cell>, layers: Vec<Entity>) -> Self {
        Self { grid, layers }
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

impl From<Grid<Cell>> for GMap {
    fn from(g: Grid<Cell>) -> Self {
        GMap {
            grid: NGrid::from_grid(g),
            layers: vec![],
        }
    }
}
