use crate::commons;
use crate::commons::grid::Coord;
use crate::commons::v2i::V2I;
use specs::prelude::*;
use specs_derive::*;

pub type Index = usize;
pub type P2 = V2I;

#[derive(Component, Debug)]
pub struct Avatar {}

#[derive(Component, Debug, Clone)]
pub struct Label {
    pub name: String,
}

#[derive(Debug)]
pub struct Player {
    avatar_id: Entity,
    bscurrent: BitSet,
}

impl Player {
    pub fn new(current: Entity) -> Self {
        let mut bsc = BitSet::new();
        bsc.add(current.id());

        Player {
            avatar_id: current,
            bscurrent: bsc,
        }
    }

    pub fn get_avatar_id(&self) -> Entity {
        return self.avatar_id;
    }

    pub fn get_avatarset(&self) -> &BitSet {
        &self.bscurrent
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Position {
    pub grid_id: Entity,
    pub point: Coord,
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct GridPosition {
    pub grid_id: Option<Entity>,
    pub pos: Coord,
}

impl Default for GridPosition {
    fn default() -> Self {
        GridPosition {
            grid_id: None,
            pos: V2I { x: 0, y: 0 },
        }
    }
}

impl Position {
    // fn get_at(&self, dir: Dir) -> HasPos {
    //     let mut p = self.clone();
    //
    //     match dir {
    //         Dir::N => p.y -= 1,
    //         Dir::S => p.y += 1,
    //         Dir::W => p.x -= 1,
    //         Dir::E => p.x += 1,
    //     }
    //
    //     p
    // }
}

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    pub fn inv(&self) -> Self {
        match self {
            Dir::N => Dir::S,
            Dir::S => Dir::N,
            Dir::E => Dir::W,
            Dir::W => Dir::E,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Dir::N => "n",
            Dir::S => "s",
            Dir::E => "e",
            Dir::W => "w",
        }
    }

    pub fn as_vec(&self) -> (i32, i32) {
        match self {
            Dir::N => (0, -1),
            Dir::S => (0, 1),
            Dir::W => (-1, 0),
            Dir::E => (1, 0),
        }
    }
}
#[derive(Component, PartialEq, Copy, Clone, Debug)]
pub enum ObjectsType {
    Door { vertical: bool },
    Engine,
    Cockpit,
}

#[derive(Component, Debug, Clone)]
pub struct Galaxy {}

#[derive(Component, Debug, Clone)]
pub struct Sector {
    pub bodies: Vec<Entity>,
}

impl Default for Sector {
    fn default() -> Self {
        Sector { bodies: vec![] }
    }
}

#[derive(Component, Debug, Clone)]
pub enum SectorBody {
    Planet,
    Station,
    Jump { target_pos: P2, target: Entity },
    Ship,
}

#[derive(Component, Debug, Clone)]
pub struct SurfaceZone {}

#[derive(Debug, Clone, Copy)]
pub enum SurfaceTileKind {
    Plain,
    Structure,
}

#[derive(Component, Debug, Clone)]
pub struct Surface {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<SurfaceTileKind>,
    pub zones: Vec<Entity>,
}

impl Surface {
    pub fn find_surface_body(
        entities: &Entities,
        storage: &ReadStorage<Surface>,
        surface_id: Entity,
    ) -> Option<Entity> {
        for (e, s) in (entities, storage).join() {
            if s.zones.contains(&surface_id) {
                return Some(e);
            }
        }

        None
    }

    pub fn get_tile(&self, x: i32, y: i32) -> Option<SurfaceTileKind> {
        let index = commons::grid::coords_to_index(self.width, Coord::new(x, y));
        self.tiles.get(index as usize).copied()
    }
}

#[derive(Component, Debug, Clone)]
pub enum Location {
    // flying through the sector
    Sector {
        sector_id: Entity,
        pos: P2,
    },
    // orbiting a body
    Orbit {
        target_id: Entity,
    },
    // at surface, in big map scale (ship is a dot)
    // TODO: probably deprecate it?
    BodySurface {
        body_id: Entity,
        place_coords: P2,
    },
    // at surface, low scale map (ship is full model)
    BodySurfacePlace {
        body_id: Entity,
        // place in big scale map
        place_coords: P2,
        // pos in surface
        grid_pos: P2,
    },
}
