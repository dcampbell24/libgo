//! The core Go logic.

/// A structure that maintains the board's arrangement of stones and properties derived from the
/// arrangement.
pub mod board;

/// A connected set of verticies in the same state.
pub mod chain;

/// A structure holding all of the chains on a board.
pub mod chains;

/// A structure that holds the state all of the verticies of the board in a matrix.
pub mod matrix;
/// Black or White.
pub mod player;
/// A structure for storing the x and y coordinates of a board cell.
pub mod vertex;
/// White, Empty, or black.
pub mod web;

use rand::{self, Rng};

use game::board::CHINESE_KOMI;
use game::board::{Board, Move};
use game::player::Player;
use game::vertex::Vertex;

/// The time settings for a game.
#[derive(Clone, Copy, Debug)]
pub enum Clock {
    /// Neither player can lose on time.
    Unlimited,
}

/// This structure includes everything needed for playing real Go games.
#[derive(Clone, Debug)]
pub struct Game {
    player_turn: Player,
    /// The current state of the board.
    board_history: Vec<Board>,
    move_history: Vec<Move>,
    /// The score handicap.
    pub komi: f64,
    time_settings: Clock,
    /// Has KGS told us a game just ended?
    pub kgs_game_over: bool,
    /// The variation of Go being played.
    pub rule_set: RuleSet,
}

impl Game {
    /// Returns a shared reference to the game board.
    pub fn board(&self) -> &Board {
        self.board_history.last().expect("expected board_history to not be empty")
    }

    /// Returns a unique reference to the game board.
    pub fn board_mut(&mut self) -> &mut Board {
        self.board_history.last_mut().expect("expected board_history to not be empty")
    }

    /// Clears all of the stones off the board and deletes the move history.
    pub fn clear_board(&mut self) {
        self.board_history.truncate(1);
        self.move_history.truncate(0);
    }

    /// Picks a move uniform randomly from all the the possible legal moves.
    pub fn genmove_random(&mut self, player: Player) -> Move {
        let mut possible_moves = self.board().empty_verts();
        let mut rng = rand::thread_rng();

        while !possible_moves.is_empty() {
            let index = rng.gen_range(0, possible_moves.len());
            let mov = Move { player: player, vertex: Some(possible_moves[index])};
            match self.play(&mov) {
                Ok(_) => { return mov; },
                Err(_) => { possible_moves.swap_remove(index); },
            }
        }
        Move { player: player, vertex: None }
    }

    /// Returns a vector containing all of the legal moves for a player.
    pub fn all_legal_moves(&self, player: Player) -> Vec<Vertex> {
        let mut legal_moves = Vec::new();
        for vertex in self.board().empty_verts() {
            if self.is_legal_move(&Move { player: player, vertex: Some(vertex) }) {
                legal_moves.push(vertex);
            }
        }
        legal_moves
    }

    /// Returns the difference in moves left for each player. Positive values mean Black is ahead.
    /// This may be extened to surreal numbers and combintorial game values to give a more precise
    /// description of the state of the game.
    pub fn value(&self) -> i32 {
          self.all_legal_moves(Player::Black).len() as i32
        - self.all_legal_moves(Player::White).len() as i32
    }

    /// Returns a new game.
    pub fn new() -> Self {
        Game {
            player_turn: Player::Black,
            board_history: vec![Board::new(19)],
            move_history: Vec::new(),
            komi: CHINESE_KOMI,
            time_settings: Clock::Unlimited,
            kgs_game_over: false,
            rule_set: RuleSet::Chinese,
        }
    }

    fn is_legal_move(&self, mov: &Move) -> bool {
        if let Some(vertex) = mov.vertex {
            // The vertex must exist and be empty.
            if !self.board().is_vacant(vertex) {
                return false;
            }

            // Also, check the suicide and ko rules:
            let mut test_board = self.board().clone();
            test_board.place_stone(mov.player, vertex);
            match self.rule_set {
                RuleSet::Chinese => {
                    // Check if the move commited suicide.
                    if test_board.is_vacant(vertex) {
                        return false;
                    }
                    // Check whether the super-ko rule was broken.
                    let test_arrangment = test_board.identity();
                    for board in &self.board_history {
                        if test_arrangment == board.identity() {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    /// Atempts to play a move.
    ///
    /// Returns Ok(()) if the move is legal, otherwise an error String.
    pub fn play(&mut self, mov: &Move) -> Result<(), String> {
        if !self.is_legal_move(mov) {
            return Err("illegal move".to_owned());
        }
        let board = self.board().clone();
        self.board_history.push(board);
        if let Some(vertex) = mov.vertex {
            self.board_mut().place_stone(mov.player, vertex);
        }
        self.player_turn = match mov.player {
            Player::Black => Player::White,
            Player::White => Player::Black,
        };
        self.move_history.push(mov.clone());
        Ok(())
    }

    /// Undo the last move. Fails if there are no moves to undo.
    pub fn undo(&mut self) -> Result<(), String> {
        if self.move_history.is_empty() {
            Err("move history is empty, can't undo".to_owned())
        } else {
            self.move_history.pop();
            self.board_history.pop();
            Ok(())
        }
    }
}

/// One of major Go variations.
#[derive(Clone, Copy, Debug)]
pub enum RuleSet {
    /// [Chinese ruleset][1]
    /// [1]: http://senseis.xmp.net/?ChineseRules
    Chinese,
}
