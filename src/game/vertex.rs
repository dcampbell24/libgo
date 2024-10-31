use std::fmt;
use std::str::FromStr;

const GOBAN_LETTERS: &str = "ABCDEFGHJKLMNOPQRST";

/// A structure for storing the x and y coordinates of a board cell.
///
/// (0, 0) is the bottom left corner of the board.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Vertex {
    /// The x coordinate.
    pub x: usize,
    /// The y coordinate.
    pub y: usize,
}

impl fmt::Debug for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let letter = GOBAN_LETTERS
            .chars()
            .nth(self.x)
            .expect("expected char to be in GOBAN_LETTERS");
        let number = (self.y + 1).to_owned();
        write!(f, "{letter}{number}")
    }
}

impl FromStr for Vertex {
    type Err = String;

    fn from_str(vertex: &str) -> Result<Self, Self::Err> {
        if vertex.len() < 2 {
            return Err("string too short to be a vertex".to_owned());
        }

        let letter = vertex
            .chars()
            .next()
            .expect("expected vertex to contain a letter");

        let Some(x) = GOBAN_LETTERS.find(letter) else {
            return Err(format!("invalid coordinate letter {letter:?}"));
        };

        let number: String = vertex.chars().skip(1).collect();
        let y = match number.parse::<u32>() {
            Ok(y) => y as usize,
            Err(_) => return Err("number is not a u32".to_owned()),
        };

        if y == 0 {
            return Err("number must be greater than zero".to_owned());
        }
        Ok(Vertex { x, y: y - 1 })
    }
}

/// A collection of Vertices. This is a wrapper type for providing traits such as Display.
#[derive(Debug)]
pub struct Vertices(pub Vec<Vertex>);

impl fmt::Display for Vertices {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, vert) in self.0.iter().enumerate() {
            if index == 0 {
                write!(f, "{}", &vert)?;
            } else {
                write!(f, " {vert}")?;
            }
        }
        Ok(())
    }
}
