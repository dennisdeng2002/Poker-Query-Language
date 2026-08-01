//! Suit-isomorphic canonical form of a complete six-card hole-card set.

use crate::{Card, Card64, CardCount, Game, IsomorphicCard, Suit, SuitMap, isomorphism::util};

const N_OMAHA_FLUSH: CardCount = 2;

#[derive(Clone, Copy)]
pub enum FlushTextureOmaha6 {
    Flush(Suit),
    DoubleFlush(Suit, Suit),
    TripleFlush(Suit, Suit, Suit),
}

impl FlushTextureOmaha6 {
    #[inline]
    const fn classify(cards: [Card; Game::OMAHA6_CARDS as usize]) -> Self {
        let c64 = Card64::from_arr(cards);
        let suits = c64.suits_with_min_count(N_OMAHA_FLUSH);

        match (suits.first_suit(), suits.second_suit(), suits.third_suit()) {
            (Some(s0), Some(s1), Some(s2)) => Self::TripleFlush(s0, s1, s2),
            (Some(s0), Some(s1), None) => Self::DoubleFlush(s0, s1),
            (Some(s), None, None) => Self::Flush(s),
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
        c5: Card,
    ) -> [IsomorphicCard; Game::OMAHA6_CARDS as usize] {
        let cards = [c0, c1, c2, c3, c4, c5];

        let map = match Self::classify(cards) {
            Self::Flush(s) => SuitMap::map1(s),
            Self::DoubleFlush(s0, s1) => util::double_flush_map(s0, s1, cards),
            Self::TripleFlush(s0, s1, s2) => util::triple_flush_map(s0, s1, s2, cards),
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
        let [c0, c1, c2, c3, c4, c5]: [Card; 6] = cards!(input).try_into().unwrap();
        assert_eq!(
            iso(expected).as_slice(),
            FlushTextureOmaha6::preflop_iso(c0, c1, c2, c3, c4, c5).as_slice()
        );
    }

    #[test]
    fn test_omaha6_single_flush() {
        // 3 spades + 1 heart + 1 diamond + 1 club: only spades reach >=2.
        assert_iso("2s3s4s5h6d7c", "2x3x4x5n6n7n");
    }

    #[test]
    fn test_omaha6_double_flush() {
        // 2 spades + 2 hearts + 1 diamond + 1 club.
        assert_iso("2s3s4h5h6d7c", "2x3x4y5y6n7n");
    }

    #[test]
    fn test_omaha6_triple_flush() {
        // 2 spades + 2 hearts + 2 diamonds.
        assert_iso("2s3s4h5h6d7d", "2x3x4y5y6z7z");
    }

    #[test]
    fn suits_are_symmetric() {
        assert_eq!(
            FlushTextureOmaha6::preflop_iso(
                card!("2s"),
                card!("3s"),
                card!("4h"),
                card!("5h"),
                card!("6d"),
                card!("7d"),
            )
            .as_slice(),
            FlushTextureOmaha6::preflop_iso(
                card!("2h"),
                card!("3h"),
                card!("4d"),
                card!("5d"),
                card!("6s"),
                card!("7s"),
            )
            .as_slice(),
        );
    }
}
