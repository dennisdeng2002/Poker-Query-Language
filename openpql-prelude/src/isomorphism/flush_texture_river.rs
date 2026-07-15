//! Flush-relevant suit pattern across a complete five-card river board.

use crate::{
    Board, Card, CardCount, HoleCardRule, IsomorphicHand, Suit, SuitMap, isomorphism::util,
};

type River = [Card; Board::N_RIVER as usize];

#[derive(Clone, Copy)]
pub enum FlushTextureRiver {
    NoFlush,
    Flush(Suit, CardCount),
}

impl FlushTextureRiver {
    #[inline]
    pub(crate) const fn classify(cards: &[Card]) -> Self {
        Self::classify_suits(
            cards[0].suit,
            cards[1].suit,
            cards[2].suit,
            cards[3].suit,
            cards[4].suit,
        )
    }

    #[inline]
    const fn classify_suits(s0: Suit, s1: Suit, s2: Suit, s3: Suit, s4: Suit) -> Self {
        match Self::flush_suit(s0, s1, s2, s3, s4) {
            Some(suit) => Self::Flush(
                suit,
                suit.const_eq(s0) as CardCount
                    + suit.const_eq(s1) as CardCount
                    + suit.const_eq(s2) as CardCount
                    + suit.const_eq(s3) as CardCount
                    + suit.const_eq(s4) as CardCount,
            ),
            None => Self::NoFlush,
        }
    }

    #[inline]
    const fn flush_suit(s0: Suit, s1: Suit, s2: Suit, s3: Suit, s4: Suit) -> Option<Suit> {
        match (s0, s1, s2, s3, s4) {
            (a, b, c, _, _) if a.const_eq(b) && b.const_eq(c) => Some(a),
            (a, b, _, d, _) if a.const_eq(b) && b.const_eq(d) => Some(a),
            (a, b, _, _, e) if a.const_eq(b) && b.const_eq(e) => Some(a),
            (a, _, c, d, _) if a.const_eq(c) && c.const_eq(d) => Some(a),
            (a, _, c, _, e) if a.const_eq(c) && c.const_eq(e) => Some(a),
            (a, _, _, d, e) if a.const_eq(d) && d.const_eq(e) => Some(a),
            (_, b, c, d, _) if b.const_eq(c) && c.const_eq(d) => Some(b),
            (_, b, c, _, e) if b.const_eq(c) && c.const_eq(e) => Some(b),
            (_, b, _, d, e) if b.const_eq(d) && d.const_eq(e) => Some(b),
            (_, _, c, d, e) if c.const_eq(d) && d.const_eq(e) => Some(c),
            _ => None,
        }
    }

    pub(crate) const fn to_pair(river: River, player: &[Card]) -> (IsomorphicHand, IsomorphicHand) {
        match Self::classify(&river) {
            Self::NoFlush => util::no_flush_pair(&river, player),
            Self::Flush(suit, count) => {
                if HoleCardRule::from_cards(player).can_make_flush(suit, count, player) {
                    util::mapped_pair(SuitMap::map1(suit), &river, player)
                } else {
                    util::no_flush_pair(&river, player)
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::*;

    fn pair(board: &str, player: &str) -> (IsomorphicHand, IsomorphicHand) {
        let b = cards!(board);
        FlushTextureRiver::to_pair([b[0], b[1], b[2], b[3], b[4]], &cards!(player))
    }

    fn iso(s: &str) -> IsomorphicHand {
        isocards!(s).into()
    }

    fn assert_pair(input: (&str, &str), expected: (&str, &str)) {
        let (board, player) = pair(input.0, input.1);

        assert_eq!(board, iso(expected.0));
        assert_eq!(player, iso(expected.1));
    }

    #[test]
    fn no_flush_board_keeps_no_suit() {
        assert_pair(("AhKsQd9c2s", "JhTd"), ("2n9nQnKnAn", "TnJn"));
    }

    #[test]
    fn flush_board_holdem_player_cannot_flush() {
        assert_pair(("AhKhQh9s2c", "JsTd"), ("2n9nQnKnAn", "TnJn"));
    }

    #[test]
    fn flush_board_holdem_player_completes_flush() {
        assert_pair(("AhKhQh9s2c", "JhTh"), ("2n9nQxKxAx", "TxJx"));
    }

    #[test]
    fn flush_board_omaha_player_below_two_suited() {
        assert_pair(("AhKhQh9s2c", "JhTs8s7d"), ("2n9nQnKnAn", "7n8nTnJn"));
    }

    #[test]
    fn flush_board_omaha_player_two_suited() {
        assert_pair(("AhKhQh9s2c", "JhTh8s7d"), ("2n9nQxKxAx", "7n8nTxJx"));
    }

    #[test]
    fn five_card_flush_board_omaha() {
        assert_pair(("AhKhQhJh2h", "9h6h8s7d"), ("2xJxQxKxAx", "6x7n8n9x"));
    }

    #[quickcheck]
    fn test_flush_suit(cards: CardN<5>) {
        let c64 = Card64::from(cards.as_slice());
        let lhs = if c64.max_same_suit_count() >= 3 {
            Some(c64.most_frequent_suits())
        } else {
            None
        };

        match FlushTextureRiver::classify_suits(
            cards[0].suit,
            cards[1].suit,
            cards[2].suit,
            cards[3].suit,
            cards[4].suit,
        ) {
            FlushTextureRiver::NoFlush => assert!(lhs.is_none()),
            FlushTextureRiver::Flush(rhs, count) => {
                assert_eq!(lhs, Some(rhs.into()));
                assert_eq!(count, c64.max_same_suit_count());
            }
        }
    }
}
