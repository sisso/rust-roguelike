use std::ops::{Add, Sub};

pub const ZERO: V2I = V2I { x: 0, y: 0 };

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq, PartialOrd, Default)]
pub struct V2I {
    pub x: i32,
    pub y: i32,
}

impl V2I {
    pub fn new(x: i32, y: i32) -> Self {
        V2I { x, y }
    }

    pub fn translate(&self, dx: i32, dy: i32) -> V2I {
        V2I::new(self.x + dx, self.y + dy)
    }

    pub fn as_array(&self) -> [i32; 2] {
        [self.x, self.y]
    }

    pub fn as_tuple(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn inverse(self) -> V2I {
        V2I {
            x: -self.x,
            y: -self.y,
        }
    }

    pub fn into_rlk_point(self) -> rltk::Point {
        self.into()
    }

    pub fn distance(&self, v: V2I) -> f32 {
        self.distance_sqr(v).sqrt()
    }

    pub fn distance_sqr(&self, v: V2I) -> f32 {
        let delta = v - *self;
        (delta.x * delta.x + delta.y * delta.y) as f32
    }
}

impl From<(i32, i32)> for V2I {
    fn from((x, y): (i32, i32)) -> Self {
        V2I { x: x, y: y }
    }
}

impl From<[i32; 2]> for V2I {
    fn from(array: [i32; 2]) -> Self {
        V2I {
            x: array[0],
            y: array[1],
        }
    }
}

impl Add for V2I {
    type Output = V2I;

    fn add(self, other: Self) -> Self::Output {
        V2I::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for V2I {
    type Output = V2I;

    fn sub(self, other: V2I) -> Self::Output {
        V2I::new(self.x - other.x, self.y - other.y)
    }
}

impl From<V2I> for rltk::Point {
    fn from(value: V2I) -> Self {
        rltk::Point::new(value.x, value.y)
    }
}

impl From<rltk::Point> for V2I {
    fn from(p: rltk::Point) -> Self {
        V2I { x: p.x, y: p.y }
    }
}
