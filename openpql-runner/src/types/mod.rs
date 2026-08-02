pub use openpql_core::{
    PQLBoard, PQLCard, PQLCardCount, PQLCardSet, PQLDouble, PQLEquity, PQLFlopHandCategory,
    PQLFraction, PQLGame, PQLHandType, PQLHiRating, PQLLoRating, PQLPlayer, PQLPlayerCount,
    PQLRank, PQLRankSet, PQLStreet, PQLSuit, PQLSuitSet,
};
use openpql_prelude as prelude;
// Range Values:
pub use pql_board_range::PQLBoardRange;
pub use pql_range::PQLRange;

use super::*;

// Primitives
pub type PQLInteger = PQLLong;
pub type PQLLong = i64;
pub type PQLBoolean = bool;
pub type PQLString = String;
pub use pql_numeric::PQLNumeric;

// Card Values:
pub type DeadCards = PQLCardSet;
pub type PQLFlop = prelude::Flop;

// Category Values:
pub struct PQLHandRanking {}

// Game Values:

mod pql_board_range;
mod pql_numeric;
mod pql_range;
mod pql_type;

pub use pql_type::*;

/// Cards for a single fully-specified hand, small enough to always live
/// inline (never heap-allocated): a board is 5 cards, a hole-card hand at
/// most 6 (Omaha6/PLO6).
type LiteralHand = smallvec::SmallVec<[PQLCard; 6]>;

/// Parses `src` as a single fully-specified hand (e.g. `"AsKsQsJsTs9s"`),
/// preserving source order (board street slots are positional: flop is
/// cards 0..3, turn is 3, river is 4), and returning `None` unless it
/// decodes to exactly `n` distinct cards with no leftover/invalid
/// characters. Ranges with wildcards, spans, rank or suit variables, etc.
/// always fail to parse this way and return `None`.
fn literal_hand(src: &str, n: usize) -> Option<LiteralHand> {
    let mut cards = LiteralHand::new();
    let mut seen = PQLCardSet::default();
    let mut chars = src.chars().filter(|c| !c.is_whitespace());

    while let Some(rank) = chars.next() {
        let suit = chars.next()?;
        let card = PQLCard::new(PQLRank::from_char(rank)?, PQLSuit::from_char(suit)?);

        if seen.contains_card(card) {
            return None;
        }

        seen.set(card);
        cards.push(card);
    }

    (cards.len() == n).then_some(cards)
}
