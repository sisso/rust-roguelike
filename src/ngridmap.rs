use crate::gmap::{Cell, TileType};
use crate::models::P2;

struct NGridMap {
    width: usize,
    height: usize,
    cells: Vec<Option<Cell>>,
    children: Vec<(P2, NGridMap)>,
}

impl Default for NGridMap {
    fn default() -> Self {
        NGridMap {
            width: 0,
            height: 0,
            cells: vec![],
            children: vec![],
        }
    }
}

impl rltk::Algorithm2D for NGridMap {
    fn dimensions(&self) -> rltk::Point {
        rltk::Point::new(self.width, self.height)
    }
}

impl rltk::BaseMap for NGridMap {
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
    use crate::ngridmap::NGridMap;
    use rltk::Algorithm2D;
    use rltk::BaseMap;
    use rltk::Point;

    fn new_basic() -> NGridMap {
        let mut gm = NGridMap::default();
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
        let gm = NGridMap::default();
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
