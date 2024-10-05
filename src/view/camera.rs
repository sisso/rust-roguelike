use crate::P2;
use hecs::Entity;

use crate::commons::grid;
use crate::commons::recti::RectI;
use crate::models::Position;
use rltk::Point;

pub type ScreenPoint = Point;

#[derive(Clone, Debug)]
pub struct Camera {
    pub grid_id: Entity,
    screen_rect: RectI,
    world_rect: RectI,
}

#[derive(Debug, Clone, Copy)]
pub struct CameraCell {
    pub screen_pos: P2,
    pub world_pos: P2,
}

impl Camera {
    pub fn from_center(pos: Position, screen_rect: RectI) -> Self {
        Camera {
            grid_id: pos.grid_id,
            screen_rect: screen_rect,
            world_rect: RectI::new(
                pos.point.x - screen_rect.get_width() / 2,
                pos.point.y - screen_rect.get_height() / 2,
                screen_rect.get_width(),
                screen_rect.get_height(),
            ),
        }
    }

    pub fn get_world_rect(&self) -> RectI {
        self.world_rect
    }

    pub fn get_screen_rect(&self) -> RectI {
        self.screen_rect
    }

    pub fn get_world_center(&self) -> P2 {
        self.get_world_rect().center()
    }

    pub fn world_to_screen(&self, p: P2) -> P2 {
        p + self.screen_rect.get_xy() - self.world_rect.get_xy()
    }

    pub fn screen_to_world(&self, p: P2) -> P2 {
        p - self.screen_rect.get_xy() + self.world_rect.get_xy()
    }

    pub fn in_world(&self, p: P2) -> bool {
        self.get_world_rect().is_inside(&p)
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
        let ipos = grid::index_to_coord(self.camera.get_screen_rect().get_width(), self.current);
        if ipos.y >= self.camera.get_screen_rect().get_height() {
            return None;
        }

        let screen_pos = ipos + self.camera.screen_rect.get_xy();

        let world_pos = self.camera.screen_to_world(screen_pos);
        let next = Some(CameraCell {
            screen_pos,
            world_pos: world_pos,
        });

        self.current += 1;
        next
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::commons::v2i::V2I;

    fn new_camera(
        center_x: i32,
        center_y: i32,
        screen_x: i32,
        screen_y: i32,
        w: i32,
        h: i32,
    ) -> Camera {
        let camera = Camera::from_center(
            Position {
                grid_id: Entity::DANGLING,
                point: V2I::new(center_x, center_y),
            },
            RectI::new(screen_x, screen_y, w, h),
        );
        camera
    }
    /*
       Screen
       000
       000
       000

       World
       00000
       0%%%0
       0%@%0
       0%%%0
       00000

    */
    #[test]
    fn test_camera_on_camera_on_0_0() {
        let camera = new_camera(2, 2, 0, 0, 3, 3);

        assert_eq!(V2I::new(0, 0), camera.world_to_screen(V2I::new(1, 1)));
        assert_eq!(V2I::new(1, 1), camera.world_to_screen(V2I::new(2, 2)));
        assert_eq!(V2I::new(2, 2), camera.world_to_screen(V2I::new(3, 3)));

        assert_eq!(V2I::new(1, 1), camera.screen_to_world(V2I::new(0, 0)));
        assert_eq!(V2I::new(2, 2), camera.screen_to_world(V2I::new(1, 1)));
        assert_eq!(V2I::new(3, 3), camera.screen_to_world(V2I::new(2, 2)));
    }

    /*
       Screen
       .....
       .....
       ..000
       ..000
       ..000

       World
       00000
       0%%%0
       0%@%0
       0%%%0
       00000

    */
    #[test]
    fn test_camera_on_camera_on_2_2() {
        let camera = new_camera(2, 2, 2, 2, 3, 3);

        assert_eq!(V2I::new(2, 2), camera.world_to_screen(V2I::new(1, 1)));
        assert_eq!(V2I::new(3, 3), camera.world_to_screen(V2I::new(2, 2)));
        assert_eq!(V2I::new(4, 4), camera.world_to_screen(V2I::new(3, 3)));

        assert_eq!(V2I::new(1, 1), camera.screen_to_world(V2I::new(2, 2)));
        assert_eq!(V2I::new(2, 2), camera.screen_to_world(V2I::new(3, 3)));
        assert_eq!(V2I::new(3, 3), camera.screen_to_world(V2I::new(4, 4)));

        let list: Vec<_> = camera.list_cells().into_iter().collect();
        assert_eq!(9, list.len());
        eprintln!("total {:?}", list);
        assert_eq!(V2I::new(2, 2), list[0].screen_pos);
        assert_eq!(V2I::new(1, 1), list[0].world_pos);
        assert_eq!(V2I::new(3, 3), list[4].screen_pos);
        assert_eq!(V2I::new(2, 2), list[4].world_pos);
        assert_eq!(V2I::new(4, 4), list[8].screen_pos);
        assert_eq!(V2I::new(3, 3), list[8].world_pos);
    }
}
