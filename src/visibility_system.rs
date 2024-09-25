use crate::commons::grid::BaseGrid;
use crate::commons::v2i::V2I;
use crate::gridref::GridRef;
use crate::models::Position;
use crate::view::{Visibility, VisibilityMemory};
use hecs::World;

pub fn run(world: &World) {
    for (_, (viewshed, pos, memory)) in
        &mut world.query::<(&mut Visibility, &Position, Option<&mut VisibilityMemory>)>()
    {
        let gridmap = GridRef::find_area(world, pos.grid_id).unwrap();

        viewshed.visible_tiles.clear();
        viewshed.visible_tiles = rltk::field_of_view(
            rltk::Point::new(pos.point.x, pos.point.y),
            viewshed.range,
            gridmap.get_grid(),
        )
        .into_iter()
        .map(|rlp| V2I::from(rlp))
        .collect();
        viewshed.visible_tiles.retain(|p| {
            p.x >= 0
                && p.x < gridmap.get_grid().get_width()
                && p.y >= 0
                && p.y < gridmap.get_grid().get_height()
        });

        if let Some(memory) = memory {
            let know_tiles = memory.know_tiles.entry(pos.grid_id).or_default();
            for pos in &viewshed.visible_tiles {
                know_tiles.insert(*pos);
            }
        }
    }
}
