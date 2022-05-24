use super::models::*;
use crate::commons::grid::Grid;
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
        *self == GMapTile::Wall
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
    pub width: i32,
    pub height: i32,
    pub cells: Vec<Cell>,
}

impl rltk::Algorithm2D for GMap {
    fn dimensions(&self) -> rltk::Point {
        rltk::Point::new(self.width, self.height)
    }
}

impl rltk::BaseMap for GMap {
    fn is_opaque(&self, idx: usize) -> bool {
        self.cells[idx].tile.is_opaque()
    }
}

impl GMap {
    pub fn get_cell(&self, index: Index) -> &Cell {
        &self.cells[index]
    }

    pub fn get_cell_mut(&mut self, index: Index) -> &mut Cell {
        &mut self.cells[index]
    }
}

struct ViewGrid<'a> {
    grids: Vec<(Entity, P2, &'a GMap)>,
}

impl<'a> ViewGrid<'a> {
    pub fn create_view(
        locations: ReadStorage<'a, Location>,
        gmaps: ReadStorage<'a, Location>,
        entity: Entity,
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

impl Component for Grid<GMapTile> {
    type Storage = DenseVecStorage<Self>;
}
