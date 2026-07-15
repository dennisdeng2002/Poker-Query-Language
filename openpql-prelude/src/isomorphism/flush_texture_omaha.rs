//! Suit-isomorphic canonical form of a complete five-card board.

use crate::{
    Board, Card, IsomorphicCard, SuitMap,
    isomorphism::{FlushTextureTurn, util},
};

pub struct FlushTextureOmaha {}

impl FlushTextureOmaha {
    /// Canonical suit-isomorphic form of four sorted-on-entry board cards.
    #[inline]
    pub(crate) const fn preflop_iso(
        mut cards: [Card; Board::N_TURN as usize],
    ) -> [IsomorphicCard; Board::N_TURN as usize] {
        Card::const_sort(&mut cards);
        match FlushTextureTurn::classify(&cards) {
            FlushTextureTurn::NoFlush => util::relabel_sorted(SuitMap::map0(), cards),
            FlushTextureTurn::Flush(suit, _) => util::relabel_sorted(SuitMap::map1(suit), cards),
            FlushTextureTurn::DoubleFlush(s0, s1) => {
                util::relabel_sorted(util::double_flush_map(s0, s1, cards), cards)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn iso(s: &str) -> IsomorphicHand {
        isocards!(s).into()
    }

    fn assert_iso(input: &str, expected: &str) {
        assert_eq!(
            iso(expected).as_slice(),
            FlushTextureOmaha::preflop_iso(cards!(input).try_into().unwrap()).as_slice()
        );
    }

    #[test]
    fn test_omaha() {
        assert_iso("2s3h4d5c", "2n3n4n5n");
        assert_iso("2s3h4c5c", "2n3n4x5x");
        assert_iso("2s3d4d5s", "2x3y4y5x");
    }
}
