//! The core Go logic.

/// A structure that maintains the board's arrangement of stones and properties derived from the
/// arrangement.
pub mod board;

/// A structure that holds the state all of the verticies of the board in a matrix.
pub mod matrix;
/// Black or White.
pub mod player;
/// A structure for storing the x and y coordinates of a board cell.
pub mod vertex;

use rand::{self, Rng};
use std::collections::HashSet;

use crate::game::board::{Board, Move};
use crate::game::player::Player;
use crate::game::vertex::Vertex;

/// The compensation in points White gets for going second under Chinese rules.
pub const CHINESE_KOMI: f64 = 7.5;
const DEFAULT_BOARD_SIZE: usize = 19;
const MAX_MOVES: usize = 512;

/// Fixed or Free placement of the handicap stones.
#[derive(Clone, Copy, Debug)]
pub enum Handicap {
    /// Placement of stones along the star points.
    Fixed,
    /// Placement determined by the egine or client.
    Free,
}

/// The time settings for a game.
#[derive(Clone, Copy, Debug)]
pub enum Clock {
    /// Neither player can lose on time.
    Unlimited,
}

/// This structure includes everything needed for playing real Go games.
#[derive(Clone, Debug)]
pub struct Game {
    /// The current state of the board.
    board: Board,
    /// All previous board states.
    previous_boards: Vec<Board>,
    /// All moves in the game record.
    move_history: Vec<Move>,
    /// The score handicap.
    pub komi: f64,
    _time_settings: Clock,
    /// Has KGS told us a game just ended?
    pub kgs_game_over: bool,
    /// The variation of Go being played.
    pub rule_set: RuleSet,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// Returns a shared reference to the game board.
    #[must_use]
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Clears all of the stones off the board and deletes the move history.
    pub fn clear_board(&mut self) {
        self.previous_boards.clear();
        self.move_history.clear();
        self.board.clear();
    }

    /// Picks a move uniform randomly from all the the possible legal moves.
    ///
    /// # Panics
    /// Failed to pass, programming error.
    pub fn genmove_random(&mut self, player: Player) -> Move {
        let mut possible_moves = self.board.empty_verts();
        let mut rng = rand::thread_rng();

        while !possible_moves.is_empty() {
            let index = rng.gen_range(0..possible_moves.len());
            let mov = Move {
                player,
                vertex: Some(possible_moves[index]),
            };
            match self.play(&mov) {
                Ok(()) => {
                    return mov;
                }
                Err(_) => {
                    possible_moves.swap_remove(index);
                }
            }
        }

        let pass = Move {
            player,
            vertex: None,
        };
        self.play(&pass).expect("failed to pass");
        pass
    }

    /// Returns a vector containing all of the legal moves for a player.
    #[must_use]
    pub fn all_legal_moves(&self, player: Player) -> Vec<Vertex> {
        let mut legal_moves = Vec::new();
        for vertex in self.board.empty_verts() {
            if self.is_legal_move(&Move {
                player,
                vertex: Some(vertex),
            }) {
                legal_moves.push(vertex);
            }
        }
        legal_moves
    }

    /// Returns the difference in moves left for each player. Positive values mean Black is ahead.
    /// This may be extended to surreal numbers and combinatorial game values to give a more precise
    /// description of the state of the game.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn value(&self) -> i32 {
        i32::try_from(self.all_legal_moves(Player::Black).len()).unwrap()
            - i32::try_from(self.all_legal_moves(Player::White).len()).unwrap()
    }

    /// Returns a new game with the given board size.
    ///
    /// # Errors
    ///
    /// If the board size is not supported.
    pub fn with_board_size(board_size: usize) -> Result<Self, String> {
        Board::with_size(board_size).map(|board| Game {
            board,
            previous_boards: Vec::new(),
            move_history: Vec::new(),
            komi: CHINESE_KOMI,
            _time_settings: Clock::Unlimited,
            kgs_game_over: false,
            rule_set: RuleSet::Chinese,
        })
    }

    /// Returns a new game with the default board size.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new() -> Self {
        Game::with_board_size(DEFAULT_BOARD_SIZE).unwrap()
    }

    fn is_legal_move(&self, mov: &Move) -> bool {
        if let Some(vertex) = mov.vertex {
            // The vertex must exist and be empty.
            if !self.board.is_vacant(vertex) {
                return false;
            }

            // Also, check the suicide and ko rules:
            let mut test_board = self.board.clone();
            test_board.place_stone(mov.player, vertex);
            match self.rule_set {
                RuleSet::Chinese => {
                    // Check if the move committed suicide.
                    if test_board.is_vacant(vertex) {
                        return false;
                    }
                    // Check whether the super-ko rule was broken.
                    for board in &self.previous_boards {
                        if test_board == *board {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    /// Attempts to play a move.
    ///
    /// # Errors
    ///
    /// The move is illegal.
    pub fn play(&mut self, mov: &Move) -> Result<(), String> {
        if !self.is_legal_move(mov) {
            return Err("illegal move".to_owned());
        }

        if let Some(vertex) = mov.vertex {
            self.previous_boards.push(self.board.clone());
            self.board.place_stone(mov.player, vertex);
        }

        self.move_history.push(*mov);
        Ok(())
    }

    /// Undo the last move.
    ///
    /// # Errors
    ///
    /// Fails if there are no moves to undo.
    #[allow(clippy::missing_panics_doc)]
    pub fn undo(&mut self) -> Result<(), String> {
        match self.move_history.pop() {
            Some(mov) => {
                if mov.vertex.is_some() {
                    self.board = self.previous_boards.pop().unwrap();
                }
                Ok(())
            }
            None => Err("move history is empty, can't undo".to_owned()),
        }
    }

    /// Places handicap stones in fixed locations based on the number requested and the size of
    /// the board.
    ///
    /// # Errors
    ///
    /// Fails if the board is not empty or an invalid number of stones are requested.
    pub fn place_handicap(
        &mut self,
        stones: usize,
        handicap: Handicap,
    ) -> Result<Vec<Vertex>, String> {
        if stones < 2 {
            return Err("a handicap must be at least two stones".to_owned());
        }

        if let Handicap::Free = handicap {
            let max_handicaps = self.board.size() * self.board.size() - 1;
            if stones > max_handicaps {
                return Err(format!(
                    "The number of handicaps requested must be less than {max_handicaps}"
                ));
            }
        }

        if !self.board.is_empty() {
            return Err("board not empty".to_owned());
        }
        let verts = self.board.fixed_handicaps(stones);

        if let Handicap::Fixed = handicap {
            if stones > verts.len() {
                return Err(format!(
                    "a board of size {} may not have more than {} fixed handicaps",
                    self.board.size(),
                    verts.len()
                ));
            }
        }

        for vert in &verts {
            self.board.place_stone(Player::Black, *vert);
        }
        Ok(verts)
    }

    /// Places the given set of vertices as handicaps on the board.
    ///
    /// # Errors
    ///
    /// Fails if any vertices are not on the board, the board is not empty,
    /// less than two vertices are given, or so many are given that placing
    /// them would commit whole board suicide.
    pub fn set_free_handicap(&mut self, verts: &HashSet<Vertex>) -> Result<(), String> {
        if verts.len() < 2 {
            return Err("a handicap must be at least two stones".to_owned());
        }
        let max_handicaps = self.board.size() * self.board.size() - 1;
        if verts.len() > max_handicaps {
            return Err(format!(
                "The number of handicaps requested must less than {max_handicaps}"
            ));
        }

        for vertex in verts {
            if self.board.is_vacant(*vertex) {
                self.board.place_stone(Player::Black, *vertex);
            } else {
                return Err(format!("{vertex} is not on the board"));
            }
        }
        Ok(())
    }

    /// Whose turn it is to play next.
    #[must_use]
    pub fn player_turn(&self) -> Player {
        let len = self.move_history.len();
        if len > 0 {
            self.move_history[len - 1].player.enemy()
        } else if self.board.is_empty() {
            Player::Black
        } else {
            Player::White
        }
    }

    /// Whether the game has ended or not.
    #[must_use]
    pub fn is_over(&self) -> bool {
        let move_count = self.move_history.len();

        move_count > MAX_MOVES
            || move_count > 1
                && self.move_history[move_count - 1].vertex.is_none()
                && self.move_history[move_count - 2].vertex.is_none()
    }
}

/// One of major Go variations.
#[derive(Clone, Copy, Debug)]
pub enum RuleSet {
    /// [Chinese ruleset](http://senseis.xmp.net/?ChineseRules)
    Chinese,
}
