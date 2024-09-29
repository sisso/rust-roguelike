use crate::{cfg, commons, P2};
use hecs::Entity;

use crate::commons::recti::RectI;
use crate::models::Position;
use rltk::Point;

pub type ScreenPoint = Point;

#[derive(Clone, Debug, Default)]
pub struct Camera {
    pub grid_id: Option<Entity>,
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
    pub fn from_center(pos: Position, size: RectI) -> Self {
        let w = size.get_width();
        let h = size.get_height();

        Camera {
            grid_id: Some(pos.grid_id),
            x: pos.point.x - w / 2,
            y: pos.point.y - h / 2,
            w,
            h,
        }
    }

    pub fn global_rect(&self) -> RectI {
        RectI::new(self.x, self.y, self.w, self.h)
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
