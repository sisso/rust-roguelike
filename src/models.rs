use crate::gmap::TileType;
use rltk::Point;
use specs::prelude::*;
use specs_derive::*;

pub type Index = usize;
pub type P2 = Point;

#[derive(Component, Debug)]
pub struct Avatar {}

#[derive(Component, Debug)]
pub struct Player {
    pub avatar: Entity,
    pub avatar_queue: Vec<Entity>,
}

impl Player {
    pub fn new(current: Entity) -> Self {
        Player {
            avatar: current,
            avatar_queue: Default::default(),
        }
    }

    pub fn get_avatarset(&self) -> BitSet {
        let mut bs = BitSet::new();
        bs.add(self.avatar.id());
        bs
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Position {
    pub point: rltk::Point,
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

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum Dir {
    N,
    S,
    W,
    E,
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
    bodies: Vec<Entity>,
}

#[derive(Component, Debug, Clone)]
pub enum SectorBody {
    Planet { pos: P2 },
    Station { pos: P2 },
    Jump { pos: P2, target: Entity },
}

#[derive(Component, Debug, Clone)]
pub struct Surface {
    width: u32,
    height: u32,
    tiles: Vec<TileType>,
}

pub enum Location {
    // flying through the sector
    Sector {
        sector: Entity,
        pos: P2,
    },
    // orbiting a body
    Orbit {
        sector: Entity,
        body: Entity,
    },
    // at surface, in big map scale (ship is a dot)
    BodySurface {
        body: Entity,
        pos: P2,
    },
    // at surface, low scale map (ship is full model)
    BodySurfacePlace {
        body: Entity,
        surface_pos: P2,
        place: P2,
    },
}
