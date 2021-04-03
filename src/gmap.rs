use rltk::{Point, Rect};
use specs::prelude::*;
use specs_derive::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Floor,
    Wall,
    Space,
    Empty,
}

#[derive(Component, Debug, Clone)]
pub enum GMap {
    Fixed {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        cells: Vec<TileType>,
    },
    Composed {
        /// the index 0 is the topmost map and is the first to be evaluated
        maps: Vec<GMap>,
    },
}

impl rltk::Algorithm2D for GMap {
    fn dimensions(&self) -> Point {
        let r = self.rect();
        Point::new(r.width(), r.height())
    }
}

impl rltk::BaseMap for GMap {
    fn is_opaque(&self, idx: usize) -> bool {
        match self {
            GMap::Fixed {
                cells, x, y, width, ..
            } => {
                let mut pos = index_to_point(*width, idx);
                pos.x - x;
                pos.y - y;
                cells[idx] == TileType::Wall
            }
            GMap::Composed { maps } => {
                let pos = index_to_point(self.rect().width(), idx);

                maps.iter()
                    .find(|map| map.is_valid(pos))
                    .unwrap()
                    .is_valid(pos)
            }
        }
    }
}

impl GMap {
    pub fn is_valid(&self, p: Point) -> bool {
        self.rect().point_in_rect(p)
    }

    pub fn get_cell(&self, point: Point) -> Option<TileType> {
        match self {
            GMap::Fixed {
                cells, width, x, y, ..
            } => {
                let local_x = point.x - x;
                let local_y = point.y - y;
                let index = point2d_to_index(*width, local_x, local_y);
                Some(cells[index])
            }
            GMap::Composed { maps } => {
                for m in maps {
                    if m.is_valid(point) {
                        return m.get_cell(point);
                    }
                }
                None
            }
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
            GMap::Fixed { width, height, .. } => return Point::new(width / 2, height / 2),
            GMap::Composed { .. } => Point::zero(),
        }
    }

    pub fn rect(&self) -> Rect {
        match self {
            GMap::Fixed {
                x,
                y,
                width,
                height,
                ..
            } => Rect::with_size(*x, *y, *width, *height),
            GMap::Composed { maps } => {
                let mut r = maps[0].rect();
                for m in &maps[1..] {
                    let r2 = m.rect();
                    r.x1 = std::cmp::min(r.x1, r2.x1);
                    r.y1 = std::cmp::min(r.y1, r2.y1);
                    r.x2 = std::cmp::max(r.x2, r2.x2);
                    r.y2 = std::cmp::max(r.y2, r2.y2);
                }
                r
            }
        }
    }

    pub fn to_local(&self, p: Point) -> Point {
        match self {
            GMap::Fixed { x, y, .. } => Point::new(x + p.x, y + p.y),
            _ => unimplemented!(),
        }
    }
}

pub fn point2d_to_index(width: i32, x: i32, y: i32) -> usize {
    (x + y * width) as usize
}

pub fn index_to_point(width: i32, i: usize) -> Point {
    Point::new(i as i32 % width, i as i32 / width)
}

#[cfg(test)]
mod test {
    use super::*;
    /*

    X 0 1 2 3 4
    0 X X X X X
    1 X X X X X
    2 X @ @ @ X
    3 X @ @ @ X
    4 X X X X X

    X = 0,0 5,5
    @ = 1,2 3,2

    */
    pub fn create_test_map() -> GMap {
        let area_map = GMap::Fixed {
            x: 0,
            y: 0,
            width: 5,
            height: 5,
            cells: vec![TileType::Wall; 5 * 5],
        };

        let inner_map = GMap::Fixed {
            x: 1,
            y: 2,
            width: 3,
            height: 2,
            cells: vec![TileType::Floor; 3 * 2],
        };
        GMap::Composed {
            maps: vec![inner_map, area_map],
        }
    }
    #[test]
    pub fn test_gmap_rect() {
        let map = create_test_map();

        let r = map.rect();
        assert_eq!(0, r.x1);
        assert_eq!(0, r.y1);
        assert_eq!(5, r.x2);
        assert_eq!(5, r.y2);
    }
    #[test]
    pub fn test_gmap_get_cell() {
        let map = create_test_map();

        // check walls
        for x in 0..=4 {
            for y in 0..=1 {
                assert_eq!(
                    TileType::Wall,
                    map.get_cell(Point::new(x, y)).unwrap(),
                    "invalid type at ({},{})",
                    x,
                    y
                );
            }
        }

        for y in 2..=4 {
            assert_eq!(
                TileType::Wall,
                map.get_cell(Point::new(0, y)).unwrap(),
                "invalid type at ({},{})",
                0,
                y
            );
            assert_eq!(
                TileType::Wall,
                map.get_cell(Point::new(4, y)).unwrap(),
                "invalid type at ({},{})",
                0,
                y
            );
        }

        for x in 0..=4 {
            assert_eq!(
                TileType::Wall,
                map.get_cell(Point::new(x, 4)).unwrap(),
                "invalid type at ({},{})",
                x,
                4
            );
        }

        // floors
        for x in 1..=3 {
            for y in 2..=3 {
                assert_eq!(
                    TileType::Floor,
                    map.get_cell(Point::new(x, y)).unwrap(),
                    "invalid type at ({},{})",
                    x,
                    y
                );
            }
        }
    }
}
