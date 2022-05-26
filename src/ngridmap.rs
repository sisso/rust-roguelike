use crate::commons::grid::{Coord, Grid, Index};
use crate::commons::recti::RectI;
use crate::gmap;
use crate::gmap::{Cell};
use crate::models::P2;

struct NGridMap {
    pos: P2,
    grid: Grid<Cell>,
    children: Vec<NGridMap>,
}

impl Default for NGridMap {
    fn default() -> Self {
        NGridMap {
            pos: Default::default(),
            grid: Grid::new(1, 1, || Cell::default()),
            children: vec![],
        }
    }
}

impl NGridMap {
    pub fn get(&self, index: Index) -> &Cell {
        let coords = self.grid.index_to_coords(index);
        self.get_at(&coords)
    }

    pub fn get_at(&self, coords: &Coord) -> &Cell {
        if !self.grid.is_valid_coords(&coords) {
            return &gmap::EMPTY_CELL;
        }

        for c in &self.children {
            let rect = c.get_rect();
            let local_coords = rect.to_local(coords);
            let tile = c.get_at(&local_coords);
            if !tile.tile.is_nothing() {
                return tile;
            }
        }

        self.grid.get_at(coords)
    }

    pub fn size(&self) -> P2 {
        self.grid.size()
    }

    pub fn get_rect(&self) -> RectI {
        RectI::new_2_points(self.pos.clone(), self.size().clone())
    }
}

impl rltk::Algorithm2D for NGridMap {
    fn dimensions(&self) -> rltk::Point {
        // global dimension
        rltk::Point::new(self.grid.width, self.grid.height)
    }
}

impl rltk::BaseMap for NGridMap {
    fn is_opaque(&self, idx: usize) -> bool {
        // resolve index by global index
        // TOOD: resolve from layers
        self.grid.get(idx as i32).tile.is_opaque()
    }
}

#[cfg(test)]
mod test {
    use crate::commons::grid::{Coord, Grid};
    
    
    use crate::gmap::{Cell, GMapTile};
    use crate::ngridmap::NGridMap;
    use crate::P2;
    use rltk::Algorithm2D;
    use rltk::BaseMap;
    use rltk::Point;

    fn new_basic() -> NGridMap {
        let mut gm = NGridMap {
            grid: Grid::new(2, 2, || Default::default()),
            ..Default::default()
        };

        gm.grid.list[0] = Cell {
            tile: GMapTile::Wall,
        };
        gm.grid.list[1] = Cell {
            tile: GMapTile::Floor,
        };

        gm
    }

    /*
    00000
    02210
    02113
    01110
    00000

    0 = space
    1 = ground
    2 = floor
    3 = wall
    */
    fn new_complex() -> NGridMap {
        let g3 = NGridMap {
            pos: P2::new(4, 2),
            grid: Grid::new(1, 1, || Cell::new(GMapTile::Wall)),
            ..Default::default()
        };

        let mut g2 = NGridMap {
            grid: Grid::new(2, 2, || Cell::new(GMapTile::Floor)),
            ..Default::default()
        };
        g2.grid
            .set_at(&Coord::new(1, 1), Cell::new(GMapTile::Space));

        let g1 = NGridMap {
            pos: P2::new(1, 1),
            grid: Grid::new(3, 3, || Cell::new(GMapTile::Ground)),
            children: vec![g2],
        };

        let g = NGridMap {
            pos: Default::default(),
            grid: Grid::new(5, 5, || Cell::new(GMapTile::Space)),
            children: vec![g1, g3],
        };

        g
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

    #[test]
    fn test_complex() {
        let g = new_complex();

        assert_eq!(5, g.dimensions().x);
        assert_eq!(5, g.dimensions().y);

        assert_eq!(GMapTile::Space, g.get(0).tile);
        assert_eq!(GMapTile::Space, g.get_at(&P2::new(0, 0)).tile);

        assert_eq!(GMapTile::Floor, g.get(5 + 1).tile);
        assert_eq!(GMapTile::Floor, g.get_at(&P2::new(1, 1)).tile);

        assert_eq!(GMapTile::Ground, g.get(5 * 2 + 2).tile);
        assert_eq!(GMapTile::Ground, g.get_at(&P2::new(2, 2)).tile);

        assert_eq!(GMapTile::Ground, g.get(5 * 3 + 3).tile);
        assert_eq!(GMapTile::Ground, g.get_at(&P2::new(3, 3)).tile);

        assert_eq!(GMapTile::Space, g.get(5 * 4 + 4).tile);
        assert_eq!(GMapTile::Space, g.get_at(&P2::new(4, 4)).tile);

        assert_eq!(GMapTile::Space, g.get(5 * 2 + 4).tile);
        assert_eq!(GMapTile::Wall, g.get_at(&P2::new(4, 2)).tile);
    }
}

#[cfg(test)]
mod test_acceptance {
    
    
    
    

    #[test]
    fn test_view_grid() {
        // let mut w = World::new();
        // w.register::<Position>();
        // w.register::<GridPosition>();
        // w.register::<Grid<GMapTile>>();
        //
        // /*
        // 00000
        // 02210
        // 02210
        // 01110
        // 00000
        //  */
        //
        // // 0
        // let ground_id = w
        //     .create_entity()
        //     .with(GridPosition::default())
        //     .with(Grid::new(5, 5, || GMapTile::Space))
        //     .build();
        //
        // // 1
        // let floor_id = w
        //     .create_entity()
        //     .with(GridPosition {
        //         grid_id: Some(ground_id),
        //         pos: V2I { x: 1, y: 1 },
        //     })
        //     .with(Grid::new(3, 3, || GMapTile::Ground))
        //     .build();
        //
        // // 2
        // let ship_id = w
        //     .create_entity()
        //     .with(GridPosition {
        //         grid_id: Some(floor_id),
        //         pos: V2I { x: 0, y: 0 },
        //     })
        //     .with(Grid::new(2, 2, || GMapTile::Floor))
        //     .build();
        //
        // w.create_entity().with(Position {
        //     grid_id: ship_id,
        //     point: V2I { x: 0, y: 0 },
        // });
    }
}
