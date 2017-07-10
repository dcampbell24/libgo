use std::fmt;
use game::web::WEB;

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
        write!(f, "{}", color)
    }
}

impl From<WEB> for Player {
    fn from(state: WEB) -> Self {
        match state {
            WEB::Black => Player::Black,
            WEB::White => Player::White,
            WEB::Empty => panic!("can't convert from {:?} to Player", state),
        }
    }
}
