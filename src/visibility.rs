use crate::commons::v2i::V2I;
use crate::gridref::GridId;
use crate::models::Position;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default)]
pub struct Visibility {
    pub visible_tiles: Vec<V2I>,
    pub range: i32,
}

#[derive(Clone, Debug, Default)]
pub struct VisibilityMemory {
    pub know_tiles: HashMap<GridId, HashSet<V2I>>,
}

impl VisibilityMemory {
    pub fn is_know(&self, pos: Position) -> bool {
        let Some(set) = self.know_tiles.get(&pos.grid_id) else {
            return false;
        };

        set.contains(&pos.point)
    }
}
