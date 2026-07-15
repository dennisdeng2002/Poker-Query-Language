//! Flush-relevant suit pattern across a flop and turn board for equity.

use crate::{
    Board, Card, CardCount, HoleCardRule, IsomorphicHand, Suit, SuitMap,
    isomorphism::{FlushTextureFlop, util},
};

const RIVER_TO_COME: CardCount = 1;
const FLUSH_DRAW_COUNT: CardCount = 2;

type Turn = [Card; Board::N_TURN as usize];

/// Flush-relevant suit pattern across a flop and turn board for equity.
#[derive(Clone, Copy)]
pub enum FlushTextureTurn {
    /// No suit can reach a flush by the river.
    NoFlush,
    /// One suit has a live flush draw; carries the suit and its board count.
    Flush(Suit, CardCount),
    /// Two suits each have a live two-card flush draw.
    DoubleFlush(Suit, Suit),
}

impl FlushTextureTurn {
    /// Classifies the texture of a flop plus turn (four board cards).
    #[inline]
    pub(crate) const fn classify(cards: &[Card]) -> Self {
        let t = cards[3].suit;

        match FlushTextureFlop::classify(cards[0], cards[1], cards[2]) {
            FlushTextureFlop::Monotone(s) => Self::Flush(s, util::count_suit(cards, s)),
            FlushTextureFlop::Twotone(f, s) if t.const_eq(s) => Self::DoubleFlush(f, s),
            FlushTextureFlop::Twotone(f, _) => Self::Flush(f, util::count_suit(cards, f)),
            FlushTextureFlop::Rainbow(s0, _, _) if s0.const_eq(t) => {
                Self::Flush(s0, util::count_suit(cards, s0))
            }
            FlushTextureFlop::Rainbow(_, s1, _) if s1.const_eq(t) => {
                Self::Flush(s1, util::count_suit(cards, s1))
            }
            FlushTextureFlop::Rainbow(_, _, s2) if s2.const_eq(t) => {
                Self::Flush(s2, util::count_suit(cards, s2))
            }
            FlushTextureFlop::Rainbow(_, _, _) => Self::NoFlush,
        }
    }

    /// Builds the [`SuitMap`] that relabels this texture's suits canonically.
    #[inline]
    pub(crate) const fn to_suit_map(self) -> SuitMap {
        match self {
            Self::NoFlush => SuitMap::map0(),
            Self::Flush(suit, _) => SuitMap::map1(suit),
            Self::DoubleFlush(flop, turn) => SuitMap::map2(flop, turn),
        }
    }

    pub(crate) const fn to_pair(turn: Turn, player: &[Card]) -> (IsomorphicHand, IsomorphicHand) {
        let rule = HoleCardRule::from_cards(player);

        match Self::classify(&turn) {
            Self::NoFlush => util::no_flush_pair(&turn, player),
            Self::Flush(suit, count) => {
                if rule.can_make_flush(suit, count + RIVER_TO_COME, player) {
                    util::mapped_pair(SuitMap::map1(suit), &turn, player)
                } else {
                    util::no_flush_pair(&turn, player)
                }
            }
            Self::DoubleFlush(s0, s1) => {
                let board_count = FLUSH_DRAW_COUNT + RIVER_TO_COME;
                let draw0 = rule.can_make_flush(s0, board_count, player);
                let draw1 = rule.can_make_flush(s1, board_count, player);

                match (draw0, draw1) {
                    (false, false) => util::no_flush_pair(&turn, player),
                    (true, false) => util::mapped_pair(SuitMap::map1(s0), &turn, player),
                    (false, true) => util::mapped_pair(SuitMap::map1(s1), &turn, player),
                    (true, true) => {
                        let map = util::double_flush_map(s0, s1, turn);

                        (
                            IsomorphicHand::from_arr(util::relabel_sorted(map, turn)),
                            map.iso_cards(player),
                        )
                    }
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
        FlushTextureTurn::to_pair([b[0], b[1], b[2], b[3]], &cards!(player))
    }

    fn iso(s: &str) -> IsomorphicHand {
        isocards!(s).into()
    }

    fn texture(board: &str) -> FlushTextureTurn {
        FlushTextureTurn::classify(&cards!(board))
    }

    fn assert_pair(input: (&str, &str), expected: (&str, &str)) {
        let (board, player) = pair(input.0, input.1);

        assert_eq!(board, iso(expected.0));
        assert_eq!(player, iso(expected.1));
    }

    #[test]
    fn rainbow_board_keeps_no_suit() {
        assert_pair(("AhKsQd9c", "JhTd"), ("9nQnKnAn", "TnJn"));
    }

    #[test]
    fn two_card_draw_holdem_player_cannot_complete() {
        assert_pair(("AhKhQs9c", "JhTd"), ("9nQnKnAn", "TnJn"));
    }

    #[test]
    fn two_card_draw_holdem_player_completes() {
        assert_pair(("AhKhQs9c", "JhTh"), ("9nQnKxAx", "TxJx"));
    }

    #[test]
    fn three_card_draw_holdem_needs_one() {
        assert_pair(("AhKhQh9c", "JhTd"), ("9nQxKxAx", "TnJx"));
    }

    #[test]
    fn four_card_board_holdem_is_always_live() {
        assert_pair(("AhKhQhJh", "9s8d"), ("JxQxKxAx", "8n9n"));
    }

    #[test]
    fn four_card_board_omaha() {
        assert_pair(("AhKhQhJh", "7s8h9dTc"), ("JnQnKnAn", "7n8n9nTn"));
    }

    #[test]
    fn double_draw_holdem_player_cannot_complete() {
        assert_pair(("AhKhQsJs", "ThTd"), ("JnQnKnAn", "TnTn"));
    }

    #[test]
    fn double_draw_omaha_one_suit_live() {
        assert_pair(("AhKhQsJs", "ThTd9h8d"), ("JnQnKxAx", "8n9xTxTn"));
        assert_pair(("AhKhQsJs", "TsTd9s8d"), ("JxQxKnAn", "8n9xTxTn"));
    }

    #[test]
    fn double_draw_omaha_both_suits_live() {
        assert_pair(("AhKhQsJs", "Th9s8h7s"), ("JxQxKyAy", "7x8y9xTy"));
    }

    #[test]
    fn double_draw_suits_are_symmetric() {
        let (board0, player0) = pair("AhKhQsJs", "Th9h8s7s");
        let (board1, player1) = pair("AsKsQhJh", "Ts9s8h7h");

        assert_eq!(board0, board1);
        assert_eq!(player0, player1);
    }

    #[test]
    fn classifies_textures() {
        assert!(matches!(texture("AhKsQd9c"), FlushTextureTurn::NoFlush));
        assert!(matches!(texture("AhKhQs9c"), FlushTextureTurn::Flush(_, 2)));
        assert!(matches!(texture("AhKhQh9c"), FlushTextureTurn::Flush(_, 3)));
        assert!(matches!(texture("AhKhQhJh"), FlushTextureTurn::Flush(_, 4)));
        assert!(matches!(
            texture("AhKhQsJs"),
            FlushTextureTurn::DoubleFlush(_, _)
        ));
    }

    #[quickcheck]
    fn test_from_turn_matches_counts(cards: CardN<4>) {
        let c64 = Card64::from(cards.as_slice());

        match FlushTextureTurn::classify(cards.as_slice()) {
            FlushTextureTurn::NoFlush => {
                assert!(c64.max_same_suit_count() <= 1);
            }
            FlushTextureTurn::Flush(suit, count) => {
                assert_eq!(c64.count_by_suit(suit), count);
                assert!(count >= 2);
                assert_eq!(c64.max_same_suit_count(), count);
            }
            FlushTextureTurn::DoubleFlush(s0, s1) => {
                assert!(!s0.const_eq(s1));
                assert_eq!(c64.count_by_suit(s0), 2);
                assert_eq!(c64.count_by_suit(s1), 2);
            }
        }
    }
}
