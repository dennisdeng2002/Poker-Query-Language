//! The small enums every game is parameterized by: variant, street, and seats.

use crate::stack_vec::StackVec;

mod game;
mod hole_card_rule;
mod player;
mod street;

pub use game::Game;
pub use hole_card_rule::HoleCardRule;
pub use player::Player;
pub use street::Street;

/// Numeric index identifying a player.
pub type PlayerIdx = u8;

/// Maximum number of players in a game.
pub const MAX_PLAYERS: PlayerIdx = 10;

/// Per-seat values stored inline, bounded by [`MAX_PLAYERS`].
pub type PerPlayer<T> = StackVec<T, { MAX_PLAYERS as usize }>;
