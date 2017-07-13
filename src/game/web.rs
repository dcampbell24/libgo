use game::player::Player;

/// The possible board vertex states.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WEB {
    /// A stone from second player.
    White = -1,
    /// No stone.
    Empty = 0,
    /// A stone from the first player.
    Black = 1,
}

impl Default for WEB {
    fn default() -> Self {
        WEB::Empty
    }
}

impl From<Player> for WEB {
    fn from(player: Player) -> Self {
        match player {
            Player::White => WEB::White,
            Player::Black => WEB::Black,
        }
    }
}
