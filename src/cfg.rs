use crate::area::Tile;
use crate::models::ObjectsType;
use serde::{Deserialize, Serialize};

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
pub const SECTOR_SIZE: i32 = 11;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapParserCfg {
    pub raw_map_tiles: Vec<(char, Tile)>,
    pub raw_map_objects: Vec<(char, ObjectsType)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cfg {
    pub map_parser: MapParserCfg,
}

impl Cfg {
    pub fn new() -> Self {
        let raw_map_tiles: Vec<(char, Tile)> = vec![
            ('_', Tile::Space),
            ('.', Tile::Floor),
            ('#', Tile::Wall),
            ('E', Tile::Wall),
            ('-', Tile::Floor),
            ('|', Tile::Floor),
            ('@', Tile::Floor),
            ('!', Tile::Floor),
        ];

        let raw_map_objects: Vec<(char, ObjectsType)> = vec![
            ('E', ObjectsType::Engine),
            ('-', ObjectsType::Door { vertical: false }),
            ('|', ObjectsType::Door { vertical: true }),
            ('@', ObjectsType::Cockpit),
            ('!', ObjectsType::Door { vertical: true }),
        ];

        Cfg {
            map_parser: MapParserCfg {
                raw_map_tiles,
                raw_map_objects,
            },
        }
    }
}
