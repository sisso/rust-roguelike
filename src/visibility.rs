use crate::commons::v2i::V2I;
use crate::gridref::GridId;
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
