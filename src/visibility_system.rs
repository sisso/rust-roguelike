use crate::gridref::GridRef;
use crate::models::Position;
use crate::view::Viewshed;
use hecs::World;

pub fn run(world: &World) {
    for (_, (viewshed, pos)) in &mut world.query::<(&mut Viewshed, &Position)>() {
        let gridmap = GridRef::find_area(world, pos.grid_id).unwrap();

        viewshed.visible_tiles.clear();
        viewshed.visible_tiles = rltk::field_of_view(
            rltk::Point::new(pos.point.x, pos.point.y),
            viewshed.range,
            &*gridmap,
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
