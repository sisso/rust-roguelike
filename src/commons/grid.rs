use super::recti;
use super::v2i::V2I;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Dir {
    N,
    E,
    S,
    W,
}

pub const DIR_ALL: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];
pub type Coord = V2I;
pub type Index = i32;

/**
    0 1 2
    3 4 5
    6 7 8
*/
#[derive(Debug, Clone)]
pub struct Grid<T> {
    pub width: i32,
    pub height: i32,
    pub list: Vec<T>,
}

impl<T> Grid<T> {
    pub fn new_square(size: i32, default: fn() -> T) -> Self {
        Grid::new(size, size, default)
    }

    pub fn new(width: i32, height: i32, default: fn() -> T) -> Self {
        let mut list = vec![];
        for _ in 0..width * height {
            list.push(default());
        }

        Grid {
            width,
            height,
            list,
        }
    }

    pub fn set(&mut self, index: Index, value: T) {
        assert!(self.is_valid_index(index));
        self.list[index as usize] = value;
    }

    pub fn set_at(&mut self, coord: &Coord, value: T) -> T {
        assert!(self.is_valid_coords(coord));
        let index = self.coords_to_index(coord);
        std::mem::replace(&mut self.list[index as usize], value)
    }

    pub fn get(&self, index: i32) -> &T {
        assert!(self.is_valid_index(index));
        &self.list[index as usize]
    }

    pub fn get_at(&self, coord: &Coord) -> &T {
        assert!(self.is_valid_coords(&coord));
        let index = self.coords_to_index(coord);
        &self.list[index as usize]
    }

    pub fn get_at_opt(&self, coord: &Coord) -> Option<&T> {
        let index = self.coords_to_index(coord);
        if self.is_valid_coords(coord) {
            Some(&self.list[index as usize])
        } else {
            None
        }
    }

    // not safe to use if you try to verify an X axis beyond grid bounds
    pub fn is_valid_index(&self, index: Index) -> bool {
        index < self.list.len() as i32
    }

    pub fn is_valid_coords(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.y >= 0 && coord.x < self.width as i32 && coord.y < self.height as i32
    }

    pub fn coords_to_index(&self, coords: &Coord) -> Index {
        coords_to_index(self.width, coords)
    }

    pub fn index_to_coords(&self, index: Index) -> Coord {
        index_to_coord(self.width, index)
    }

    pub fn get_valid_4_neighbours(&self, coords: &Coord) -> Vec<Coord> {
        get_4_neighbours(coords)
            .into_iter()
            .map(|(_, i)| i)
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn get_valid_8_neighbours(&self, coords: &Coord) -> Vec<Coord> {
        get_8_neighbours(coords)
            .into_iter()
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn raytrace(&self, pos: &Coord, dir_x: i32, dir_y: i32) -> Vec<Coord> {
        let mut current = *pos;
        let mut result = vec![];

        loop {
            let nx = current.x as i32 + dir_x;
            let ny = current.y as i32 + dir_y;
            if nx < 0 || ny < 0 {
                break;
            }

            current = V2I::new(nx, ny);

            if !self.is_valid_coords(&current) {
                break;
            }

            result.push(current);
        }

        result
    }
    pub fn size(&self) -> V2I {
        V2I::new(self.width, self.height)
    }
}

pub fn coords_to_index(width: i32, xy: &Coord) -> Index {
    xy.y * width + xy.x
}

pub fn index_to_coord(width: i32, index: Index) -> Coord {
    Coord::new(index % width, index / width)
}

/// return sequentially with DIR_ALL
pub fn get_4_neighbours(coords: &Coord) -> Vec<(Dir, Coord)> {
    vec![
        (Dir::N, coords.translate(0, -1)),
        (Dir::E, coords.translate(1, 0)),
        (Dir::S, coords.translate(0, 1)),
        (Dir::W, coords.translate(-1, 0)),
    ]
}

pub fn get_8_neighbours(coords: &Coord) -> Vec<Coord> {
    vec![
        coords.translate(0, -1),
        coords.translate(1, -1),
        coords.translate(1, 0),
        coords.translate(1, 1),
        coords.translate(0, 1),
        coords.translate(-1, 1),
        coords.translate(-1, 0),
        coords.translate(-1, -1),
    ]
}

#[derive(Debug, Clone)]
pub struct FlexGrid<T> {
    pub cells: HashMap<Coord, T>,
}

impl<T> FlexGrid<T> {
    pub fn new() -> Self {
        FlexGrid {
            cells: HashMap::new(),
        }
    }

    pub fn set_at(&mut self, coord: &Coord, value: Option<T>) {
        match value {
            Some(v) => self.cells.insert(coord.to_owned(), v),
            None => self.cells.remove(coord),
        };
    }

    pub fn get_at(&self, coord: &Coord) -> Option<&T> {
        self.cells.get(coord)
    }
}

#[derive(Clone, Debug)]
pub struct PGrid<T> {
    pub pos: V2I,
    pub grid: Grid<T>,
}

impl<T> PGrid<T> {
    pub fn new(x: i32, y: i32, width: i32, height: i32, default: fn() -> T) -> Self {
        PGrid {
            pos: V2I::new(x, y),
            grid: Grid::new(width, height, default),
        }
    }

    pub fn from_grid(coord: &V2I, grid: Grid<T>) -> Self {
        PGrid { pos: *coord, grid }
    }

    pub fn get_pos(&self) -> Coord {
        self.pos
    }

    pub fn set_pos(&mut self, pos: &V2I) {
        self.pos = *pos;
    }

    pub fn get_width(&self) -> i32 {
        self.grid.width
    }

    pub fn get_height(&self) -> i32 {
        self.grid.height
    }

    pub fn get_rect(&self) -> recti::RectI {
        recti::RectI::new(self.pos.x, self.pos.y, self.grid.width, self.grid.height)
    }

    pub fn to_local(&self, coord: &Coord) -> Coord {
        recti::to_local(&self.pos, coord)
    }

    pub fn to_global(&self, coord: &Coord) -> Coord {
        recti::to_global(&self.pos, coord)
    }

    pub fn set_at(&mut self, coord: &Coord, value: T) -> T {
        let local = self.to_local(coord);
        assert!(self.grid.is_valid_coords(&local));
        self.grid.set_at(&local, value)
    }

    pub fn get_at(&self, coord: &Coord) -> &T {
        let local = self.to_local(coord);
        assert!(self.grid.is_valid_coords(&local));
        self.grid.get_at(&local)
    }

    pub fn get_at_opt(&self, coord: &Coord) -> Option<&T> {
        let local = self.to_local(coord);
        self.grid.get_at_opt(&local)
    }

    pub fn is_valid_coords(&self, coord: &Coord) -> bool {
        self.get_rect().is_inside(coord)
    }

    pub fn get_valid_4_neighbours(&self, coords: &Coord) -> Vec<Coord> {
        get_4_neighbours(coords)
            .into_iter()
            .map(|(_, i)| i)
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn get_valid_8_neighbours(&self, coords: &Coord) -> Vec<Coord> {
        get_8_neighbours(coords)
            .into_iter()
            .filter(|i| self.is_valid_coords(i))
            .collect()
    }

    pub fn raytrace(&self, pos: &Coord, dir_x: i32, dir_y: i32) -> Vec<Coord> {
        let mut current = *pos;
        let mut result = vec![];

        loop {
            let nx = current.x as i32 + dir_x;
            let ny = current.y as i32 + dir_y;
            if nx < 0 || ny < 0 {
                break;
            }

            current = V2I::new(nx, ny);

            if !self.is_valid_coords(&current) {
                break;
            }

            result.push(current);
        }

        result
    }
}

impl<T> From<PGrid<T>> for Grid<T> {
    fn from(pgrid: PGrid<T>) -> Self {
        pgrid.grid
    }
}

pub trait GridCell {
    fn is_empty(&self) -> bool;
}

#[derive(Clone, Debug)]
pub struct NGrid<T: GridCell> {
    grids: Vec<PGrid<T>>,
}

impl<T: GridCell> NGrid<T> {
    pub fn new() -> Self {
        NGrid { grids: vec![] }
    }

    pub fn get_layer(&self, coord: &Coord) -> Option<usize> {
        let mut found = None;

        for (layer_id, g) in self.grids.iter().enumerate().rev() {
            match g.get_at_opt(coord) {
                Some(tile) if !tile.is_empty() => {
                    found = Some(layer_id);
                    break;
                }
                Some(_) => {
                    found = Some(layer_id);
                }
                _ => {}
            }
        }

        found
    }

    pub fn from_grid(grid: Grid<T>) -> Self {
        NGrid {
            grids: vec![PGrid::from_grid(&super::v2i::ZERO, grid)],
        }
    }

    pub fn get_size(&self) -> V2I {
        assert!(!self.grids.is_empty());
        (self.grids[0].get_width(), self.grids[0].get_height()).into()
    }

    pub fn get_width(&self) -> i32 {
        self.grids[0].get_width()
    }

    pub fn get_height(&self) -> i32 {
        self.grids[0].get_height()
    }
    pub fn is_valid(&self, coords: &Coord) -> bool {
        self.grids[0].is_valid_coords(coords)
    }

    pub fn get_at(&self, coord: &Coord) -> Option<&T> {
        let layer_id = self.get_layer(coord);
        layer_id.and_then(|index| {
            let grid = &self.grids[index];
            grid.get_at_opt(coord)
        })
    }

    pub fn push_surface_at(&mut self, coord: &V2I, surf: NGrid<T>) {
        for mut g in surf.grids {
            // translate new surface into local position
            let pos = g.get_pos().translate(coord.x, coord.y);
            g.set_pos(&pos);

            self.grids.push(g);
        }
    }

    pub fn len(&self) -> usize {
        self.grids.len()
    }

    pub fn remove(&mut self, index: usize, _default: T) -> NGrid<T> {
        assert!(index <= self.grids.len());

        let grid = self.grids.remove(index);
        NGrid::from_grid(grid.into())
    }

    pub fn merge(&mut self, gmap: NGrid<T>, pos: &Coord) {
        for mut grid in gmap.grids {
            grid.pos = grid.pos.translate(pos.x, pos.y);
            self.grids.push(grid);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_grid_get_neighbors() {
        let neighbours = get_4_neighbours(&Coord::new(0, 0));
        assert_eq!(
            neighbours,
            vec![
                (Dir::N, Coord::new(0, -1)),
                (Dir::E, Coord::new(1, 0)),
                (Dir::S, Coord::new(0, 1)),
                (Dir::W, Coord::new(-1, 0)),
            ]
        );
    }

    #[test]
    pub fn test_grid_get_valid_neighbors() {
        let grid = Grid::<i32>::new(2, 2, || 0);
        let neighbours = grid.get_valid_8_neighbours(&Coord::new(0, 0));
        assert_eq!(
            vec![Coord::new(1, 0), Coord::new(1, 1), Coord::new(0, 1),],
            neighbours
        );
    }

    #[test]
    pub fn test_grid_raytrace() {
        let mut grid = Grid::<i32>::new(4, 2, || 0);

        // X###
        // ###
        assert_eq!(grid.raytrace(&(0, 0).into(), -1, 0), Vec::<Coord>::new());

        // #X##
        // ####
        assert_eq!(grid.raytrace(&(1, 0).into(), -1, 0), Vec::<Coord>::new());

        // 0###
        // ####
        grid.set_at(&(0, 0).into(), 0);

        // 0X##
        // ####
        assert_eq!(grid.raytrace(&(1, 0).into(), -1, 0), vec![(0, 0).into()]);

        // 00##
        // ####
        grid.set_at(&(1, 0).into(), 0);

        // 00X#
        // ####
        assert_eq!(
            grid.raytrace(&(2, 0).into(), -1, 0),
            vec![(1, 0).into(), (0, 0).into()]
        );

        // 00#X
        // ####
        assert_eq!(grid.raytrace(&(3, 0).into(), -1, 0), vec![]);

        // X0##
        // ####
        assert_eq!(grid.raytrace(&(0, 0).into(), 1, 0), vec![(1, 0).into()]);
    }

    #[test]
    fn test_index_to_coords() {
        assert_eq!(V2I::new(0, 0), index_to_coord(3, 0));
        assert_eq!(V2I::new(1, 0), index_to_coord(3, 1));
        assert_eq!(V2I::new(2, 0), index_to_coord(3, 2));
        assert_eq!(V2I::new(0, 1), index_to_coord(3, 3));
        assert_eq!(V2I::new(1, 1), index_to_coord(3, 4));
        assert_eq!(V2I::new(2, 1), index_to_coord(3, 5));
        assert_eq!(V2I::new(0, 2), index_to_coord(3, 6));
        assert_eq!(V2I::new(1, 2), index_to_coord(3, 7));
        assert_eq!(V2I::new(2, 2), index_to_coord(3, 8));

        assert_eq!(V2I::new(0, 1), index_to_coord(4, 4));
        assert_eq!(V2I::new(1, 1), index_to_coord(4, 5));
        assert_eq!(V2I::new(2, 1), index_to_coord(4, 6));
        assert_eq!(V2I::new(3, 1), index_to_coord(4, 7));
    }

    #[test]
    fn test_ngrid_merge() {
        /*
         110
         022
         022
        */

        impl GridCell for i32 {
            fn is_empty(&self) -> bool {
                false
            }
        }

        let g0 = Grid::new(3, 3, || 0);
        let g1 = Grid::new(2, 1, || 1);
        let g2 = Grid::new(2, 2, || 2);

        let mut ng0 = NGrid::from_grid(g0);
        let ng1 = NGrid::from_grid(g1);
        let ng2 = NGrid::from_grid(g2);

        ng0.merge(ng1, &V2I::new(0, 0));
        ng0.merge(ng2, &V2I::new(1, 1));

        assert_eq!(3, ng0.get_width());
        assert_eq!(3, ng0.get_height());
        assert_eq!(Some(&1), ng0.get_at(&V2I::new(0, 0)));
        assert_eq!(Some(&1), ng0.get_at(&V2I::new(1, 0)));
        assert_eq!(Some(&0), ng0.get_at(&V2I::new(2, 0)));
        assert_eq!(Some(&0), ng0.get_at(&V2I::new(0, 1)));
        assert_eq!(Some(&2), ng0.get_at(&V2I::new(1, 1)));
        assert_eq!(Some(&2), ng0.get_at(&V2I::new(2, 1)));
        assert_eq!(Some(&0), ng0.get_at(&V2I::new(0, 2)));
        assert_eq!(Some(&2), ng0.get_at(&V2I::new(1, 2)));
        assert_eq!(Some(&2), ng0.get_at(&V2I::new(2, 2)));
    }
}
