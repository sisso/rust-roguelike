use super::models::*;
use rltk::Point;
use specs::prelude::*;
use specs_derive::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Floor,
    Wall,
    Space,
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
        self.cells[idx].tile == TileType::Wall
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
    pub index: Index,
    pub tile: TileType,
}
