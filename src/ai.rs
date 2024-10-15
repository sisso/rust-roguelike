use crate::actions::{WantAttack, WantMove};
use crate::commons::grid::{BaseGrid, Index};
use crate::gridref::AreaRef;
use crate::models::Position;
use crate::visibility::Visibility;
use hecs::{CommandBuffer, Entity, World};

#[derive(Clone, Default, Debug)]
pub struct Ai {}
#[derive(Clone, Default, Debug)]
pub struct HasATurn;

pub fn give_turn_to_ai(world: &mut World) {
    let mut buffer = CommandBuffer::new();
    for (e, _) in world.query_mut::<&Ai>() {
        buffer.insert_one(e, HasATurn);
    }
    buffer.run_on(world);
}

pub fn run_ai_mob_system(world: &mut World, player_id: Entity) {
    let Ok(player_pos) = world.query_one_mut::<&Position>(player_id).cloned() else {
        return;
    };
    let mut buffer = CommandBuffer::new();

    for (e, (pos, view, _)) in &mut world.query::<(&Position, &Visibility, &HasATurn)>() {
        buffer.remove_one::<HasATurn>(e);

        if pos.grid_id != player_pos.grid_id {
            continue;
        }

        if !view.visible_tiles.contains(&player_pos.point) {
            continue;
        }

        let gmap = AreaRef::resolve_area(world, player_pos.grid_id).unwrap();

        let from_index = gmap.get_grid().coords_to_index(pos.point) as usize;
        let to_index = gmap.get_grid().coords_to_index(player_pos.point) as usize;

        let rs = rltk::a_star_search(from_index, to_index, gmap.get_grid());
        if !rs.success {
            log::warn!("{:?} fail to find path to {:?}", e, player_id);
            continue;
        }

        if rs.steps.len() <= 2 {
            buffer.insert_one(
                e,
                WantAttack {
                    target_id: player_id,
                },
            );
            continue;
        }

        let next_index = rs.steps[1];
        let next_pos = gmap.get_grid().index_to_coords(next_index as Index);
        let delta_pos = next_pos - pos.point;
        buffer.insert_one(e, WantMove { dir: delta_pos });
    }

    buffer.run_on(world);
}
#[cfg(test)]
mod test {
    use crate::area::Area;
    use crate::commons::grid::BaseGrid;
    use crate::commons::grid_string;
    use crate::loader::new_parser;
    use crate::{cfg, loader};
    use hecs::Entity;

    #[test]
    fn test_pathfinding() {
        env_logger::builder()
            .filter(None, log::LevelFilter::Debug)
            .init();

        pub const raw_grid: &str = r"
....
#.##
#.#.
....
";
        let cfg = cfg::Cfg::new();
        let ast = grid_string::parse_map(new_parser(cfg.clone()), raw_grid)
            .expect("fail to load house map");
        log::debug!("ast: {:?}", ast);
        let grid = loader::new_grid_from_ast(&ast);
        let area = Area::from(Entity::DANGLING, grid);

        let from = area.get_grid().coords_to_index([0, 0].into());
        assert_eq!(0, from);
        let to = area.get_grid().coords_to_index([3, 3].into());
        assert_eq!(15, to);

        let rs = rltk::a_star_search(from, to, area.get_grid());
        log::debug!("path found {:?}", rs.steps);
        for index in &rs.steps {
            log::debug!(
                "- path {:?}",
                area.get_grid().index_to_coords(*index as i32)
            );
        }

        assert_eq!(0, rs.steps[0]); // include first
        assert_eq!(5, rs.steps.len());
        assert_eq!(15, rs.steps[rs.steps.len() - 1]); // include last
    }
}
