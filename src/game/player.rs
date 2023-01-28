use std::fmt;

/// Black or White.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Player {
    /// Player 1.
    Black,
    /// Player 2.
    White,
}

impl Player {
    /// The opponenet or enemy of the player.
    pub fn enemy(&self) -> Self {
        match *self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let color = match *self {
            Player::Black => "black",
            Player::White => "white",
        };
        write!(f, "{color}")
    }
}
