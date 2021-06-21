use crate::gmap::{Cell, TileType};
use crate::models::P2;

struct GridMap {
    width: usize,
    height: usize,
    cells: Vec<Option<Cell>>,
    children: Vec<(P2, GridMap)>,
}

impl Default for GridMap {
    fn default() -> Self {
        GridMap {
            width: 0,
            height: 0,
            cells: vec![],
            children: vec![],
        }
    }
}

impl rltk::Algorithm2D for GridMap {
    fn dimensions(&self) -> rltk::Point {
        rltk::Point::new(self.width, self.height)
    }
}

impl rltk::BaseMap for GridMap {
    fn is_opaque(&self, idx: usize) -> bool {
        self.cells[idx]
            .as_ref()
            .map(|i| i.tile.is_opaque())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use crate::gmap::{Cell, TileType};
    use crate::GridMap::GridMap;
    use rltk::Algorithm2D;
    use rltk::BaseMap;
    use rltk::Point;

    fn new_basic() -> GridMap {
        let mut gm = GridMap::default();
        gm.width = 2;
        gm.height = 2;
        gm.cells = vec![
            Some(Cell {
                tile: TileType::Wall,
            }),
            Some(Cell {
                tile: TileType::Space,
            }),
            None,
            Some(Cell {
                tile: TileType::Space,
            }),
        ];
        gm
    }

    #[test]
    fn test_dimension_empty() {
        let gm = GridMap::default();
        let p = gm.dimensions();
        assert_eq!(Point::new(0, 0), p);
    }

    #[test]
    fn test_dimension() {
        let gm = new_basic();
        let p = gm.dimensions();
        assert_eq!(Point::new(2, 2), p);
    }

    #[test]
    fn test_opaque() {
        let gm = new_basic();

        assert_eq!(true, gm.is_opaque(0));
        assert_eq!(false, gm.is_opaque(1));
        assert_eq!(false, gm.is_opaque(2));
        assert_eq!(false, gm.is_opaque(3));
    }
}
