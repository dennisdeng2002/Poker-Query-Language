//! Suit-isomorphic canonical form of a complete five-card board.

use crate::{Card, Card64, CardCount, Game, IsomorphicCard, Suit, SuitMap, isomorphism::util};

const N_OMAHA_FLUSH: CardCount = 2;

#[derive(Clone, Copy)]
pub enum FlushTextureOmaha5 {
    Flush(Suit),
    DoubleFlush(Suit, Suit),
}

impl FlushTextureOmaha5 {
    #[inline]
    const fn classify(cards: [Card; Game::OMAHA5_CARDS as usize]) -> Self {
        let c64 = Card64::from_arr(cards);
        let suits = c64.suits_with_min_count(N_OMAHA_FLUSH);

        match (suits.first_suit(), suits.second_suit()) {
            (Some(s0), Some(s1)) => Self::DoubleFlush(s0, s1),
            (Some(s), _) => Self::Flush(s),
            _ => unreachable!(), // LCOV_EXCL_LINE
        }
    }

    #[inline]
    pub(crate) const fn preflop_iso(
        c0: Card,
        c1: Card,
        c2: Card,
        c3: Card,
        c4: Card,
    ) -> [IsomorphicCard; Game::OMAHA5_CARDS as usize] {
        let cards = [c0, c1, c2, c3, c4];

        let map = match Self::classify(cards) {
            Self::Flush(s) => SuitMap::map1(s),
            Self::DoubleFlush(s0, s1) => util::double_flush_map(s0, s1, cards),
        };

        util::relabel_sorted(map, cards)
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
        let [c0, c1, c2, c3, c4]: [Card; 5] = cards!(input).try_into().unwrap();
        assert_eq!(
            iso(expected).as_slice(),
            FlushTextureOmaha5::preflop_iso(c0, c1, c2, c3, c4).as_slice()
        );
    }

    #[test]
    fn test_omaha5() {
        assert_iso("2s3s4h5d6c", "2x3x4n5n6n");
        assert_iso("2s3s7s5d6c", "2x3x5n6n7x");
        assert_iso("2s3h4h5s6c", "2x3y4y5x6n");
        assert_iso("2s3s4s5h6h", "2x3x4x5y6y");
        assert_iso("2h3s4s5h6c", "2x3y4y5x6n");
    }
}
