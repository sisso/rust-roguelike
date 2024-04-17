use crate::commons::grid::Grid;

#[derive(Debug)]
pub enum ParseMapError {
    UnknownChar(char),
    FewLines,
    InvalidLineWidth(String),
    UnknownTileAt([i32; 2]),
}

pub trait Parser<Tile> {
    fn parse_tile(&self, ch: char) -> Option<Tile>;
}

pub fn parse_map<P, Tile>(parser: P, map_str: &str) -> Result<Grid<Tile>, ParseMapError>
where
    P: Parser<Tile>,
{
    let raw = parse_map_str(map_str)?;
    parse_rawmap(parser, &raw)
}

fn parse_rawmap<P, Tile>(parser: P, map: &Grid<char>) -> Result<Grid<Tile>, ParseMapError>
where
    P: Parser<Tile>,
{
    let mut cells = Vec::with_capacity(map.len());

    for ch in map.iter() {
        let tile = match parser.parse_tile(*ch) {
            Some(tile) => tile,
            None => return Err(ParseMapError::UnknownChar(*ch)),
        };

        cells.push(tile);
    }

    let grid = Grid::new_from(map.get_width(), map.get_height(), cells);
    Ok(grid)
}

/// All empty spaces are removed a can not be used
/// If first line is empty, is removed,
/// if last line is empty, is removed
fn parse_map_str(map: &str) -> Result<Grid<char>, ParseMapError> {
    let mut lines: Vec<String> = map.split("\n").map(|line| line.replace(" ", "")).collect();

    if lines.is_empty() {
        return Err(ParseMapError::FewLines);
    }

    if lines[0].is_empty() {
        lines.remove(0);
    }

    if lines.is_empty() {
        return Err(ParseMapError::FewLines);
    }

    if lines[lines.len() - 1].is_empty() {
        lines.remove(lines.len() - 1);
    }

    let width = lines[0].len() as i32;
    let height = lines.len() as i32;
    let mut cells = Vec::with_capacity((width * height) as usize);

    for (_y, line) in lines.iter().enumerate() {
        if line.len() != width as usize {
            return Err(ParseMapError::InvalidLineWidth(line.clone()));
        }

        for ch in line.chars() {
            cells.push(ch)
        }
    }

    Ok(Grid::new_from(width, height, cells))
}

pub trait GridSerialize<T> {
    fn serialize(&self, tile: &T) -> Option<char>;
}

pub fn serialize<F, T>(serialize: F, grid: &Grid<T>) -> Result<Vec<String>, ParseMapError>
where
    F: Fn(&T) -> Option<char>,
{
    let mut lines = Vec::with_capacity(grid.get_height() as usize);

    for y in 0..grid.get_height() {
        let mut line = String::with_capacity(grid.get_width() as usize);

        for x in 0..grid.get_width() {
            let tile = grid.get_at([x, y].into());
            let ch = match serialize(tile) {
                Some(ch) => ch,
                None => return Err(ParseMapError::UnknownTileAt([x, y])),
            };

            line.push(ch);
        }

        lines.push(line);
    }

    Ok(lines)
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestParser;

    #[derive(PartialEq, Debug, Copy, Clone)]
    enum Tile {
        Unknown,
        Wall,
        Floor,
    }

    impl Parser<Tile> for TestParser {
        fn parse_tile(&self, ch: char) -> Option<Tile> {
            if ch == 'X' {
                Some(Tile::Wall)
            } else if ch == '.' {
                Some(Tile::Floor)
            } else {
                Some(Tile::Unknown)
            }
        }
    }

    #[test]
    fn test_parse_map_and_serialize() {
        let map = r#"XXXX
            X..X
            XXXX"#;

        let grid = parse_map(TestParser, map).unwrap();
        assert_eq!(grid.get_width(), 4);
        assert_eq!(grid.get_height(), 3);
        assert_eq!(*grid.get_at([0, 0].into()), Tile::Wall);
        assert_eq!(*grid.get_at([1, 1].into()), Tile::Floor);

        let serializer = |tile: &Tile| match tile {
            Tile::Wall => Some('A'),
            Tile::Floor => Some('B'),
            _ => None,
        };

        let lines = serialize(serializer, &grid).unwrap();
        assert_eq!(lines[0], "AAAA");
        assert_eq!(lines[1], "ABBA");
        assert_eq!(lines[2], "AAAA");
    }
}
