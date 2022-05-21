use super::models::*;
use specs::prelude::*;
use specs_derive::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GMapTile {
    Ground,
    Floor,
    Wall,
    Space,
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

#[derive(Component, Debug, Clone)]
pub struct Cell {
    pub tile: GMapTile,
}
