use std::fmt;

use game::player::Player;
use game::graph::Graph;
use game::vertex::Vertex;

const BOARD_MAX_SIZE: usize = 19;
const BOARD_MIN_SIZE: usize = 1;
const BOARD_LETTERS: &'static str = "ABCDEFGHJKLMNOPQRST";

/// A representation of the board state.
#[derive(Clone)]
pub struct Board {
    /// A graph holding the state of each vertex on the board.
    graph: Graph<State>,
}

impl PartialEq for Board {
    fn eq(&self, other: &Board) -> bool {
        self.graph == other.graph
    }
}

impl Board {
    /// Returns the center point (天元 tengen) of the board. Note that even size boards don't have a
    /// center point.
    pub fn center_point(&self) -> Option<Vertex> {
        let board_size = self.size();

        if board_size % 2 == 0 {
            None
        } else {
            let center = board_size / 2;
            Some(Vertex {
                x: center,
                y: center,
            })
        }
    }

    /// Returns the edge star points (星 hoshi), which are traditionally marked with a small dot on
    /// the board.
    pub fn star_points(&self) -> Vec<Vertex> {
        let board_size = self.size();

        if board_size < 7 {
            return Vec::new();
        }
        let min_line = if board_size > 12 { 3 } else { 2 };
        let max_line = board_size - min_line - 1;
        let mut star_points = vec![
            Vertex {
                x: min_line,
                y: min_line,
            },
            Vertex {
                x: max_line,
                y: max_line,
            },
            Vertex {
                x: min_line,
                y: max_line,
            },
            Vertex {
                x: max_line,
                y: min_line,
            },
        ];
        if board_size == 7 {
            return star_points;
        }

        let center_line = match self.center_point() {
            Some(center) => center.x,
            None => return star_points,
        };

        star_points.append(&mut vec![
            Vertex {
                x: min_line,
                y: center_line,
            },
            Vertex {
                x: max_line,
                y: center_line,
            },
            Vertex {
                x: center_line,
                y: min_line,
            },
            Vertex {
                x: center_line,
                y: max_line,
            },
        ]);
        star_points
    }

    /// Returns a list of handicap vertices given a board size and desired number of stones. The
    /// number of handicaps returned will be as large as possible given the number of valid
    /// handicaps, but may be less than requested.
    pub fn fixed_handicaps(&self, stones: usize) -> Vec<Vertex> {
        let board_size = self.size();

        let mut handicaps = self.star_points();
        if board_size > 7 && (stones == 5 || stones == 7 || stones >= 9) {
            handicaps.truncate(stones - 1);
            match self.center_point() {
                Some(center) => handicaps.push(center),
                None => (),
            }
        } else {
            handicaps.truncate(stones);
        }
        handicaps
    }

    /// Returns true if there are no stones on the board.
    pub fn is_empty(&self) -> bool {
        self.graph.regions(State::Black).map_or(true, |blocks| blocks.is_empty()) &&
        self.graph.regions(State::White).map_or(true, |blocks| blocks.is_empty())
    }

    /// Returns true if the vertex exists and is empty.
    pub fn is_vacant(&self, vertex: Vertex) -> bool {
        match self.graph.get(vertex) {
            Some(&state) => state == State::Empty,
            None => false,
        }
    }

    /// Returns a list of all the empty vertices.
    pub fn empty_verts(&self) -> Vec<Vertex> {
        self.graph.vertices().filter(|&vertex| self.graph[vertex] == State::Empty).collect()
    }

    /// Removes all of the stones from the board.
    pub fn clear(&mut self) {
        self.graph.reset();
    }

    /// Creates a new board with the given size. A full size game is 19, but 13 and 9 are also
    /// common. Returns None if the board size is not supported.
    pub fn with_size(size: usize) -> Result<Self, String> {
        if size < BOARD_MIN_SIZE || size > BOARD_MAX_SIZE {
            Err(format!(
                "Board size must be between {} and {}, but is {}.",
                BOARD_MIN_SIZE,
                BOARD_MAX_SIZE,
                size
            ))
        } else {
            Ok(Board { graph: Graph::with_matrix_order(size) })
        }
    }

    /// Updates the board with a move. The move is assumed to be valid and legal.
    pub fn place_stone(&mut self, player: Player, vertex: Vertex) {
        self.graph.set(vertex, State::from(player));

        self.remove_captures(player);
        // If there were any captures, use a set_region function rather than setting each individually,
        // Remove suicides.
        self.remove_captures(player.enemy());
    }

    /// Removes all enemy Chains from the board that have 0 liberties. Returns true if any groups
    /// were removed.
    fn remove_captures(&mut self, capturer: Player) -> bool {
        self.graph.shift_regions(State::from(capturer.enemy()), State::Empty)
    }

    fn push_letters(&self, board: &mut String) {
        board.push_str("  ");
        for letter in BOARD_LETTERS.chars().take(self.graph.grid_length()) {
            board.push(' ');
            board.push(letter);
        }
        board.push_str("   ");
    }

    /// Returns the current size of the board.
    pub fn size(&self) -> usize {
        self.graph.grid_length()
    }

    /// The score according to ancient rules (count of black stones minus count of white stones).
    pub fn score_ancient(&self) -> i32 {
        self.graph.values().fold(0, |acc, &state| {
            match state {
                State::Empty => acc,
                State::Black => acc + 1,
                State::White => acc - 1,
            }
        })
    }

    /// Returns a human readable ASCII representation of the board.
    pub fn to_ascii(&self) -> String {
        let size = self.size();
        let star_points = self.star_points();
        let mut board = String::new();
        self.push_letters(&mut board);
        board.push_str("\r\n");
        for y in (0..size).rev() {
            board.push_str(&format!("{:02}", y + 1));
            for x in 0..size {
                board.push(' ');
                let vertex = Vertex { x: x, y: y };
                let c = match self.graph[vertex] {
                    State::Empty => {
                        if star_points.contains(&vertex) {
                            '+'
                        } else {
                            '.'
                        }
                    }
                    State::Black => 'x',
                    State::White => 'o',
                };
                board.push(c);
            }
            board.push_str(&format!(" {:02}\r\n", y + 1));
        }
        self.push_letters(&mut board);
        board
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_ascii())
    }
}

/// Includes a player and a location on the board, or None for pass.
#[derive(Clone, Copy, Debug)]
pub struct Move {
    /// The player taking the move.
    pub player: Player,
    /// A coordinate on the Go board.
    pub vertex: Option<Vertex>,
}

/// The possible board states.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum State {
    /// A stone from second player.
    White = -1,
    /// No stone.
    Empty = 0,
    /// A stone from the first player.
    Black = 1,
}

impl Default for State {
    fn default() -> Self {
        State::Empty
    }
}

impl From<Player> for State {
    fn from(player: Player) -> Self {
        match player {
            Player::White => State::White,
            Player::Black => State::Black,
        }
    }
}
