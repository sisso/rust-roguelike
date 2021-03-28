use crate::gmap::GMap;
use crate::models::Position;
use crate::view::Viewshed;
use specs::prelude::*;
use specs_derive::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        ReadExpect<'a, GMap>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (map, mut viewshed, pos): Self::SystemData) {
        for (viewshed, pos) in (&mut viewshed, &pos).join() {
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles =
                rltk::field_of_view(rltk::Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed
                .visible_tiles
                .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

            for pos in &viewshed.visible_tiles {
                viewshed.know_tiles.insert(*pos);
            }
        }
    }
}
