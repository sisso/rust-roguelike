use crate::{cfg, commons, P2};
use rltk::{Point, Rect};

use crate::commons::v2i::V2I;
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
    pub screen_point: P2,
    pub point: P2,
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

    pub fn from_center(p: P2) -> Self {
        let w = cfg::SCREEN_W;
        let h = cfg::SCREEN_H;

        Camera {
            x: p.x - w / 2,
            y: p.y - h / 2,
            w: w,
            h: h,
        }
    }

    pub fn global_rect(&self) -> commons::recti::RectI {
        commons::recti::RectI::new(self.x, self.y, self.w, self.h)
    }

    pub fn global_center(&self) -> P2 {
        self.global_rect().center()
    }

    pub fn global_to_screen(&self, p: P2) -> P2 {
        (p.x - self.x, p.y - self.y).into()
    }

    pub fn screen_to_global(&self, p: P2) -> P2 {
        (p.x + self.x, p.y + self.y).into()
    }

    pub fn is_global_in(&self, p: P2) -> bool {
        self.global_rect().is_inside(&p)
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

        let screen_point = P2 { x, y };
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
