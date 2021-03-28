use super::models::*;
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
    pub fn is_valid_xy(&self, x: i32, y: i32) -> bool {
        self.width as i32 > x && x >= 0 && self.height as i32 > y && y >= 0
    }

    pub fn is_valid(&self, index: Index) -> bool {
        index < self.cells.len()
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        xy_idx((self.width) as i32, x, y)
    }

    pub fn idx_xy(&self, index: Index) -> Position {
        idx_xy((self.width) as i32, index)
    }
}

fn xy_idx(width: i32, x: i32, y: i32) -> usize {
    ((y * width as i32) + x) as usize
}

fn idx_xy(width: i32, index: Index) -> Position {
    Position {
        x: index as i32 % width as i32,
        y: index as i32 / width as i32,
    }
}
#[derive(Component, Debug, Clone)]
pub struct Cell {
    pub index: Index,
    pub tile: TileType,
}

#[cfg(test)]
mod test {
    use super::super::cfg;
    use super::*;

    #[test]
    fn test_idx_xy_and_xy_idx() {
        let index = xy_idx(cfg::SCREEN_W, 3, 5);
        let coords = idx_xy(cfg::SCREEN_W, index);
        assert_eq!(coords, Position { x: 3, y: 5 });
    }
}
