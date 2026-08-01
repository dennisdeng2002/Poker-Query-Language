//! Card, rank, suit, and board primitives.

mod board;
mod card;
mod card64;
mod card_array;
mod card_idx;
mod flop;
mod hand;
mod hand_n;
mod rank;
mod rank16;
mod rank_idx;
mod suit;
mod suit4;
mod suit_idx;
mod util;

pub use board::Board;
pub use card::Card;
pub use card_array::CardArray;
pub use card_idx::CardIdx;
pub use card64::Card64;
pub use flop::Flop;
pub use hand::Hand;
pub use hand_n::HandN;
pub use rank::Rank;
pub use rank_idx::RankIdx;
pub use rank16::Rank16;
pub use suit::Suit;
pub use suit_idx::SuitIdx;
pub use suit4::Suit4;

type Suit4Inner = u8;
pub type Rank16Inner = u16;
type Card64Inner = u64;

/// Card count type used throughout the crate.
pub type CardCount = u8;
/// Signed integer representation of a card, rank, or suit.
pub type Idx = i8;

/// Maximum number of hole cards stored inline.
pub const MAX_HOLECARDS: usize = 6;
