use crate::gmap::GMap;
use crate::models::Position;
use crate::view::Viewshed;
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        ReadStorage<'a, GMap>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
    );

    fn run(&mut self, (gridmaps, mut viewshed, pos): Self::SystemData) {
        for (viewshed, pos) in (&mut viewshed, &pos).join() {
            let gridmap = gridmaps.get(pos.grid_id).unwrap();

            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = rltk::field_of_view(
                rltk::Point::new(pos.point.x, pos.point.y),
                viewshed.range,
                gridmap,
            );
            viewshed.visible_tiles.retain(|p| {
                p.x >= 0
                    && p.x < gridmap.get_grid().get_width()
                    && p.y >= 0
                    && p.y < gridmap.get_grid().get_height()
            });

            for pos in &viewshed.visible_tiles {
                viewshed.know_tiles.insert(*pos);
            }
        }
    }
}
