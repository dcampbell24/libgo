use std::collections::HashSet;
use std::fmt;

use game::chain::Chain;
use game::chains::Chains;
use game::player::Player;
use game::vertex::Vertex;
use game::web::WEB;
use game::matrix::Matrix;

/// The compensation in points White gets for going second under Chinese rules.
pub const CHINESE_KOMI: f64 = 7.5;
const BOARD_MAX_SIZE: usize = 19;
const BOARD_MIN_SIZE: usize = 1;
const BOARD_LETTERS: &'static str = "ABCDEFGHJKLMNOPQRST";

/// A representation of the board state.
#[derive(Clone)]
pub struct Board {
    /// A matrix holding the state of each vertex on the board.
    matrix: Matrix<WEB>,
    chains: Chains,
}

impl PartialEq for Board {
    fn eq(&self, other: &Board) -> bool {
        self.matrix == other.matrix
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

    /// Returns a list of handicap verticies given a board size and desired number of stones. The
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
        self.chains.is_empty()
    }

    /// Returns true if the vertex exists and is empty.
    pub fn is_vacant(&self, vertex: Vertex) -> bool {
        match self.matrix.get(vertex) {
            Some(web) => web == &WEB::Empty,
            None => false,
        }
    }

    /// Returns a list of all the empty verticies.
    pub fn empty_verts(&self) -> Vec<Vertex> {
        self.matrix.verts_in_state(WEB::Empty)
    }

    /// Returns a list of all the **unconditionally alive** chains on the board.
    ///
    /// A chain on stones is **alive** when there is no seqeunce of
    /// moves from the opponent that can capture the chain if the owner responds correctly.
    ///
    /// A chain is **unconditionally alive** or **pass alive** if there is no sequence of moves
    /// solely from the opponent that can capture the chain.
    pub fn pass_alive_chains(&self) -> Vec<Vertex> {
        unimplemented!();
    }

    /// Removes all of the stones from the board.
    pub fn clear(&mut self) {
        self.matrix.reset();
        self.chains.clear();
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
            Ok(Board {
                matrix: Matrix::with_size(size),
                chains: Chains::new(),
            })
        }
    }

    fn neighbors(&self, player: Player, vert: Vertex) -> Neighbors {
        let mut adjacencies = self.matrix.adjacencies(vert);
        let mut blacks = adjacencies.clone();
        blacks.retain(|v: &Vertex| self.matrix[v] == WEB::Black);
        let mut whites = adjacencies.clone();
        whites.retain(|v: &Vertex| self.matrix[v] == WEB::White);
        adjacencies.retain(|v: &Vertex| self.matrix[v] == WEB::Empty);

        match player {
            Player::White => Neighbors {
                good: whites,
                evil: blacks,
                empty: adjacencies,
            },
            Player::Black => Neighbors {
                good: blacks,
                evil: whites,
                empty: adjacencies,
            },
        }
    }

    /// Updates the board with a move. The move is assumed to be valid and legal.
    pub fn place_stone(&mut self, player: Player, vertex: Vertex) {
        self.add_stone(player, &vertex);
        let neighbors = self.neighbors(player, vertex);

        let mut new_chain = Chain::new(player, vertex, &neighbors);
        for vert in &neighbors.good {
            if let Some(old_chain) = self.chains.remove_chain(vert) {
                new_chain.eat(old_chain);
            }
        }
        self.chains.push(new_chain);
        self.remove_captures(player);
        // Remove suicides.
        self.remove_captures(player.enemy());
    }

    fn push_letters(&self, board: &mut String) {
        board.push_str("  ");
        for letter in BOARD_LETTERS.chars().take(self.matrix.size()) {
            board.push(' ');
            board.push(letter);
        }
        board.push_str("   ");
    }

    fn add_stone(&mut self, player: Player, vertex: &Vertex) {
        self.matrix[vertex] = WEB::from(player);
        self.chains.add_stone(player, vertex);
    }

    /// Returns the current size of the board.
    pub fn size(&self) -> usize {
        self.matrix.size()
    }

    /// Removes all enemy Chains from the board that have 0 liberties.
    fn remove_captures(&mut self, capturer: Player) {
        let empty_verts = self.chains.remove_dead_chains(capturer.enemy());
        for v in &empty_verts {
            self.matrix[v] = WEB::Empty;
        }
    }

    /// Returns all small enclosed regions of the player.
    ///
    /// A small black enclosed region R is a region such that:
    /// 1. R is surrounded by black stones.
    /// 2. The interior contains only white stones.
    /// 3. The border contains only white stones and empty intersections.
    pub fn small_enclosed_regions(&self, player: Player) -> Vec<HashSet<Vertex>> {
        let mut exterior_verts: Matrix<bool> = Matrix::with_size(self.size());
        for chain in self.chains.iter() {
            if chain.player == player {
                for vertex in chain.libs.iter().chain(chain.filled_libs.iter()) {
                    exterior_verts[vertex] = true;
                }
            }
        }
        let regions = self.matrix
            .get_regions(|vertex| vertex != &WEB::from(player));
        regions
            .into_iter()
            .filter(|region| {
                for vertex in region {
                    if !exterior_verts[vertex] && self.matrix[vertex] == WEB::Empty {
                        return false;
                    }
                }
                true
            })
            .collect()
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
                let c = match self.matrix[&vertex] {
                    WEB::Empty => {
                        if star_points.contains(&vertex) {
                            '+'
                        } else {
                            '.'
                        }
                    }
                    WEB::Black => 'x',
                    WEB::White => 'o',
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
        write!(f, "{}\r\nChains = {:?}", self, self.chains)
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

/// A structure holding the verticies neighboring a vertex, chain, or region, grouped by state.
#[derive(Clone, Debug)]
pub struct Neighbors {
    /// The player's stones.
    pub good: Vec<Vertex>,
    /// The opponent's stones.
    pub evil: Vec<Vertex>,
    /// No stones.
    pub empty: Vec<Vertex>,
}
