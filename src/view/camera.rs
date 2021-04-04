use crate::cfg;
use rltk::{Point, Rect};

use specs::prelude::*;
use specs_derive::*;

pub type ScreenPoint = Point;

#[derive(Component, Clone, Debug)]
pub struct Camera {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

pub struct CameraCell {
    pub screen_point: Point,
    pub point: Point,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            x: 0,
            y: 0,
            w: cfg::SCREEN_W,
            h: cfg::SCREEN_H,
        }
    }

    pub fn fromCenter(p: Point) -> Self {
        let w = cfg::SCREEN_W;
        let h = cfg::SCREEN_H;

        Camera {
            x: p.x - w / 2,
            y: p.y - h / 2,
            w: w,
            h: h,
        }
    }

    pub fn globla_rect(&self) -> Rect {
        Rect::with_size(self.x, self.y, self.w, self.h)
    }

    pub fn global_center(&self) -> Point {
        self.globla_rect().center()
    }

    pub fn global_to_screen(&self, p: Point) -> Point {
        (p.x - self.x, p.y - self.y).into()
    }

    pub fn screen_to_global(&self, p: Point) -> Point {
        (p.x + self.x, p.y + self.y).into()
    }

    pub fn is_global_in(&self, p: Point) -> bool {
        self.globla_rect().point_in_rect(p)
    }

    pub fn list_cells<'a>(&'a self) -> impl Iterator<Item = CameraCell> + 'a {
        CameraIterator {
            camera: self,
            current: 0,
        }
    }
}

struct CameraIterator<'a> {
    camera: &'a Camera,
    current: i32,
}

impl<'a> Iterator for CameraIterator<'a> {
    type Item = CameraCell;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.current % self.camera.w;
        let y = self.current / self.camera.w;

        let screen_point = Point { x, y };
        if y >= self.camera.h {
            return None;
        }

        let point = self.camera.screen_to_global(screen_point);
        let next = Some(CameraCell {
            screen_point: screen_point,
            point: point,
        });

        self.current += 1;
        next
    }
}
