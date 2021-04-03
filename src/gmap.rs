use super::models::*;
use rltk::Point;
use specs::prelude::*;
use specs_derive::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Floor,
    Wall,
    Space,
    OutOfMap,
}

#[derive(Component, Debug, Clone)]
pub enum GMap {
    Infinite {
        tile: TileType,
    },
    Fixed {
        width: i32,
        height: i32,
        cells: Vec<TileType>,
    },
    Composed {
        maps: Vec<GMap>,
    },
}

impl rltk::Algorithm2D for GMap {
    fn dimensions(&self) -> Point {
        match self {
            GMap::Fixed { width, height, .. } => Point::new(*width, *height),
            _ => unimplemented!(),
        }
    }
}

impl rltk::BaseMap for GMap {
    fn is_opaque(&self, idx: usize) -> bool {
        match self {
            GMap::Fixed {
                width,
                height,
                cells,
            } => cells[idx] == TileType::Wall,
            _ => unimplemented!(),
        }
    }
}

impl GMap {
    pub fn get_cell(&self, index: Index) -> TileType {
        match self {
            GMap::Fixed {
                width,
                height,
                cells,
            } => cells[index],
            _ => unimplemented!(),
        }
    }

    pub fn get_cells_mut(&mut self) -> &mut Vec<TileType> {
        match self {
            GMap::Fixed { cells, .. } => cells,
            _ => unimplemented!(),
        }
    }

    pub fn center(&self) -> Point {
        match self {
            GMap::Fixed {
                width,
                height,
                cells,
            } => return Point::new(width / 2, height / 2),
            _ => unimplemented!(),
        }
    }
}

pub fn point2d_to_index(width: i32, x: i32, y: i32) -> usize {
    (x + y * width) as usize
}
