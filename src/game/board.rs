use std::collections::HashSet;
use std::fmt;

use game::player::Player;
use game::vertex::Vertex;
use game::matrix::{Matrix, Node};

/// The compensation in points White gets for going second under Chinese rules.
pub const CHINESE_KOMI: f64 = 7.5;
const BOARD_MAX_SIZE: usize = 19;
const BOARD_MIN_SIZE: usize = 1;
const BOARD_LETTERS: &'static str = "ABCDEFGHJKLMNOPQRST";

/// A representation of the board state.
#[derive(Clone)]
pub struct Board {
    /// A matrix holding the state of each vertex on the board.
    matrix: Matrix<State>,
    chains: Chains,
}

type Chains = Vec<Chain>;

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
            Some(&state) => state == State::Empty,
            None => false,
        }
    }

    /// Returns a list of all the empty verticies.
    pub fn empty_verts(&self) -> Vec<Vertex> {
        self.matrix.verts_in_state(State::Empty)
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
                chains: Vec::new(),
            })
        }
    }

    fn neighbors(&self, player: Player, vert: Vertex) -> Neighbors {
        let mut adjacencies = self.matrix.adjacent_vertices(vert);
        let mut blacks = adjacencies.clone();
        blacks.retain(|v: &Vertex| self.matrix[v] == State::Black);
        let mut whites = adjacencies.clone();
        whites.retain(|v: &Vertex| self.matrix[v] == State::White);
        adjacencies.retain(|v: &Vertex| self.matrix[v] == State::Empty);

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
        self.matrix[&vertex] = State::from(player);

        // Remove the liberty from chains on the board.
        for chain in &mut self.chains {
            if chain.player != player && chain.libs.remove(&vertex) {
                chain.filled_libs.insert(vertex);
            }
        }

        let neighbors = self.neighbors(player, vertex);
        let mut new_chain = Chain::new(player, vertex, &neighbors);
        for vert in &neighbors.good {
            if let Some(old_chain) = self.remove_chain(vert) {
                new_chain.eat(old_chain);
            }
        }
        self.chains.push(new_chain);

        self.remove_captures(player);
        // Remove suicides.
        self.remove_captures(player.enemy());
    }

    /// Removes all enemy Chains from the board that have 0 liberties.
    fn remove_captures(&mut self, capturer: Player) {
        let empty_verts = self.remove_dead_chains(capturer.enemy());
        for v in &empty_verts {
            self.matrix[v] = State::Empty;
        }
    }

    fn push_letters(&self, board: &mut String) {
        board.push_str("  ");
        for letter in BOARD_LETTERS.chars().take(self.matrix.size()) {
            board.push(' ');
            board.push(letter);
        }
        board.push_str("   ");
    }

    /// Returns the current size of the board.
    pub fn size(&self) -> usize {
        self.matrix.size()
    }

    /// Returns all small enclosed regions of the player.
    ///
    /// A small black enclosed region R is a region such that:
    /// 1. R is surrounded by black stones.
    /// 2. The interior contains only white stones.
    /// 3. The border contains only white stones and empty intersections.
    pub fn small_enclosed_regions(&self, player: Player) -> Vec<HashSet<Node>> {
        let mut exterior_verts: Matrix<bool> = Matrix::with_size(self.size());
        for chain in self.chains.iter() {
            if chain.player == player {
                for vertex in chain.libs.iter().chain(chain.filled_libs.iter()) {
                    exterior_verts[vertex] = true;
                }
            }
        }
        let regions = self.matrix.get_regions(|node| node != &State::from(player));
        regions
            .into_iter()
            .filter(|region| {
                for &node in region.iter() {
                    if !exterior_verts[node] && self.matrix[node] == State::Empty {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// The score according to ancient rules (count of black stones minus count of white stones).
    pub fn score_ancient(&self) -> i32 {
        self.matrix.values().fold(0, |acc, &state| {
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
                let c = match self.matrix[&vertex] {
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

    // Chains //

    /// Removes the chain that contains vertex from the set of chains.
    fn remove_chain(&mut self, vertex: &Vertex) -> Option<Chain> {
        let mut idx = None;
        for (i, chain) in self.chains.iter().enumerate() {
            if chain.verts.contains(vertex) {
                idx = Some(i);
                break;
            }
        }
        if let Some(idx) = idx {
            Some(self.chains.swap_remove(idx))
        } else {
            None
        }
    }

    /// Removes all chains with zero liberties of a chosen player and returns their verticies.
    fn remove_dead_chains(&mut self, player: Player) -> Vec<Vertex> {
        let mut empty_verts = Vec::new();
        for chain in &self.chains {
            if chain.player == player && chain.libs.is_empty() {
                empty_verts.extend(&chain.verts);
            }
        }
        // Remove the dead chains before updating liberties to avoid updating dead chains.
        self.chains
            .retain(|chain| chain.player != player || !chain.libs.is_empty());
        for vertex in &empty_verts {
            for chain in &mut self.chains {
                if chain.player != player && chain.filled_libs.remove(vertex) {
                    chain.libs.insert(*vertex);
                }
            }
        }
        empty_verts
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

/// A connected set of stones of the same color.
#[derive(Clone, Debug)]
struct Chain {
    /// The state all of the verticies of the chain are in.
    player: Player,
    /// The set of verticies in the chain.
    verts: HashSet<Vertex>,
    /// The set of neighboring verticies that are empty.
    libs: HashSet<Vertex>,
    /// The set of neighboring verticies that are filled (by the opponent).
    filled_libs: HashSet<Vertex>,
}

impl Chain {
    /// Create a new chain initialized with a vertex and its neighbors.
    pub fn new(player: Player, vertex: Vertex, neighbors: &Neighbors) -> Self {
        let mut verts = HashSet::new();
        let mut libs = HashSet::new();
        let mut filled_libs = HashSet::new();

        verts.insert(vertex);
        libs.extend(&neighbors.empty);
        filled_libs.extend(&neighbors.evil);

        Chain {
            player: player,
            verts: verts,
            libs: libs,
            filled_libs: filled_libs,
        }
    }

    /// Update a chain with the consumed union of another.
    pub fn eat(&mut self, chain: Chain) {
        self.verts.extend(chain.verts);
        self.libs.extend(chain.libs);
        self.filled_libs.extend(chain.filled_libs);
    }
}
