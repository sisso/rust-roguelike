use crate::gmap::TileType;
use crate::models::ObjectsType;
use specs::prelude::*;
use specs_derive::*;

pub const SHIP_MAP: &str = r"
___________________________
___________________________
_______####________________
_______EEE#________________
_______##.#________________
________#.#________________
______###-####-#######_____
______#.....#...#....!_____
______#.@...|...#....!_____
______#.....#...|....!_____
______###-############_____
________#.#________________
_______##.#________________
_______EEE#________________
_______####________________
___________________________
___________________________
";

pub const HOUSE_MAP: &str = r"
#########
#.......#
|.......|
#...#####
#...|...#
###-#####
#.......#
|.......|
#.......#
###-#####
";

pub const SCREEN_W: i32 = 80;
pub const SCREEN_H: i32 = 50;

#[derive(Component, Debug)]
pub struct Cfg {
    pub raw_map_tiles: Vec<(char, TileType)>,
    pub raw_map_objects: Vec<(char, ObjectsType)>,
}

impl Cfg {
    pub fn new() -> Self {
        let raw_map_tiles: Vec<(char, TileType)> = vec![
            ('_', TileType::Space),
            ('.', TileType::Floor),
            ('#', TileType::Wall),
            ('E', TileType::Wall),
            ('-', TileType::Floor),
            ('|', TileType::Floor),
            ('@', TileType::Floor),
            ('!', TileType::Floor),
        ];

        let raw_map_objects: Vec<(char, ObjectsType)> = vec![
            ('E', ObjectsType::Engine),
            ('-', ObjectsType::Door { vertical: false }),
            ('|', ObjectsType::Door { vertical: true }),
            ('@', ObjectsType::Cockpit),
            ('!', ObjectsType::Door { vertical: true }),
        ];
        Cfg {
            raw_map_tiles,
            raw_map_objects,
        }
    }
}
