use specs::prelude::*;
use specs_derive::*;

pub type Index = usize;

#[derive(Component, Debug)]
pub struct Avatar {}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    fn get_at(&self, dir: Dir) -> Position {
        let mut p = self.clone();

        match dir {
            Dir::N => p.y -= 1,
            Dir::S => p.y += 1,
            Dir::W => p.x -= 1,
            Dir::E => p.x += 1,
        }

        p
    }
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
