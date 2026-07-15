//! Flush-relevant suit pattern of a flop's three cards.

use crate::{
    Board, Card, CardCount, HoleCardRule, IsomorphicHand, Suit, SuitMap, isomorphism::util,
};

/// The suit appearing twice in a two-tone flop.
type DoubleSuit = Suit;

const TURN_AND_RIVER_TO_COME: CardCount = 2;
const MONOTONE_COUNT: CardCount = 3;
const DOUBLE_COUNT: CardCount = 2;
const SINGLE_COUNT: CardCount = 1;

type FlopArray = [Card; Board::N_FLOP as usize];

/// Flush-relevant suit pattern of a flop's three cards.
#[derive(Clone, Copy)]
pub enum FlushTextureFlop {
    /// All three cards share one suit.
    Monotone(Suit),
    /// Two cards share a suit; carries the doubled suit then the lone suit.
    Twotone(DoubleSuit, Suit),
    /// All three suits differ.
    Rainbow(Suit, Suit, Suit),
}

impl FlushTextureFlop {
    /// Classifies the texture of three rank-sorted flop cards.
    #[inline]
    pub(crate) const fn classify(f0: Card, f1: Card, f2: Card) -> Self {
        let (s0, s1, s2) = (f0.suit, f1.suit, f2.suit);

        match (s0.const_eq(s1), s0.const_eq(s2), s1.const_eq(s2)) {
            (true, true, _) => Self::Monotone(s0),
            (true, false, _) => Self::Twotone(s0, s2),
            (false, true, _) => Self::Twotone(s0, s1),
            (false, false, true) => Self::Twotone(s1, s0),
            (false, false, false) => Self::Rainbow(s0, s1, s2),
        }
    }

    /// Builds the [`SuitMap`] that relabels this texture's suits canonically.
    #[inline]
    pub(crate) const fn to_suit_map(self) -> SuitMap {
        match self {
            Self::Monotone(s) => SuitMap::map1(s),
            Self::Twotone(dbl, s) => SuitMap::map2(dbl, s),
            Self::Rainbow(s0, s1, s2) => SuitMap::map3(s0, s1, s2),
        }
    }

    pub(crate) const fn to_pair(
        flop: FlopArray,
        player: &[Card],
    ) -> (IsomorphicHand, IsomorphicHand) {
        let mut flop = flop;
        Card::const_sort(&mut flop);
        let [f0, f1, f2] = flop;
        let rule = HoleCardRule::from_cards(player);

        match Self::classify(f0, f1, f2) {
            Self::Monotone(s) => {
                if rule.can_make_flush(s, MONOTONE_COUNT + TURN_AND_RIVER_TO_COME, player) {
                    util::mapped_pair(SuitMap::map1(s), &flop, player)
                } else {
                    util::no_flush_pair(&flop, player)
                }
            }
            Self::Twotone(dbl, single) => {
                let live_dbl =
                    rule.can_make_flush(dbl, DOUBLE_COUNT + TURN_AND_RIVER_TO_COME, player);
                let live_single =
                    rule.can_make_flush(single, SINGLE_COUNT + TURN_AND_RIVER_TO_COME, player);

                match (live_dbl, live_single) {
                    (false, false) => util::no_flush_pair(&flop, player),
                    (true, false) => util::mapped_pair(SuitMap::map1(dbl), &flop, player),
                    (false, true) => util::mapped_pair(SuitMap::map1(single), &flop, player),
                    (true, true) => util::mapped_pair(SuitMap::map2(dbl, single), &flop, player),
                }
            }
            Self::Rainbow(s0, s1, s2) => {
                let draw = SINGLE_COUNT + TURN_AND_RIVER_TO_COME;
                let l0 = rule.can_make_flush(s0, draw, player);
                let l1 = rule.can_make_flush(s1, draw, player);
                let l2 = rule.can_make_flush(s2, draw, player);

                let map = match (l0, l1, l2) {
                    (false, false, false) => return util::no_flush_pair(&flop, player),
                    (true, false, false) => SuitMap::map1(s0),
                    (false, true, false) => SuitMap::map1(s1),
                    (false, false, true) => SuitMap::map1(s2),
                    (true, true, false) => SuitMap::map2(s0, s1),
                    (true, false, true) => SuitMap::map2(s0, s2),
                    (false, true, true) => SuitMap::map2(s1, s2),
                    (true, true, true) => unimplemented!(), // LCOV_EXCL_LINE; omaha6
                };

                util::mapped_pair(map, &flop, player)
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
        FlushTextureFlop::to_pair([b[0], b[1], b[2]], &cards!(player))
    }

    fn iso(s: &str) -> IsomorphicHand {
        isocards!(s).into()
    }

    fn texture(board: &str) -> FlushTextureFlop {
        let b = cards!(board);
        FlushTextureFlop::classify(b[0], b[1], b[2])
    }

    fn assert_pair(input: (&str, &str), expected: (&str, &str)) {
        let (board, player) = pair(input.0, input.1);

        assert_eq!(board, iso(expected.0));
        assert_eq!(player, iso(expected.1));
    }

    #[test]
    fn rainbow_holdem_no_live_suit() {
        assert_pair(("AhKsQd", "JcTh"), ("QnKnAn", "TnJn"));
    }

    #[test]
    fn rainbow_holdem_one_live_suit() {
        assert_pair(("AhKsQd", "JhTh"), ("QnKnAx", "TxJx"));
        assert_pair(("AsKhQd", "JhTh"), ("QnKxAn", "TxJx"));
        assert_pair(("AdKsQh", "JhTh"), ("QxKnAn", "TxJx"));
    }

    #[test]
    fn monotone() {
        assert_pair(("AhKhQh", "JsTd"), ("QxKxAx", "TnJn"));
        assert_pair(("AhKhQh", "2s3h4d5c"), ("QnKnAn", "2n3n4n5n"));
    }

    #[test]
    fn twotone_holdem_single_suit_live() {
        assert_pair(("AhKhQs", "JsTs"), ("QxKnAn", "TxJx"));
    }

    #[test]
    fn twotone_holdem_doubled_suit_live() {
        assert_pair(("AhKhQs", "JhTh"), ("QnKxAx", "TxJx"));
    }

    #[test]
    fn twotone_omaha() {
        assert_pair(("AhKhQs", "JhTh9s8s"), ("QyKxAx", "8y9yTxJx"));
        assert_pair(("AhKhQs", "2s3h4d5c"), ("QnKnAn", "2n3n4n5n"));
    }

    #[test]
    fn rainbow_omaha_two_suits_live() {
        assert_pair(("AhKsQd", "JhTh9s8s"), ("QnKxAy", "8x9xTyJy"));
        assert_pair(("AhKdQs", "JhTh9s8s"), ("QxKnAy", "8x9xTyJy"));
        assert_pair(("AdKsQh", "JhTh9s8s"), ("QxKyAn", "8y9yTxJx"));
    }

    #[test]
    fn suits_are_symmetric() {
        let (board0, player0) = pair("AhKsQd", "JhTh");
        let (board1, player1) = pair("AsKhQd", "JsTs");

        assert_eq!(board0, board1);
        assert_eq!(player0, player1);
    }

    #[test]
    fn classifies_textures() {
        assert!(matches!(texture("AhKsQd"), FlushTextureFlop::Rainbow(..)));
        assert!(matches!(texture("AhKhQs"), FlushTextureFlop::Twotone(..)));
        assert!(matches!(texture("AhKhQh"), FlushTextureFlop::Monotone(_)));
    }

    #[quickcheck]
    fn test_from_flop_matches_counts(cards: CardN<3>) {
        let mut sorted = [cards[0], cards[1], cards[2]];
        Card::const_sort(&mut sorted);
        let [f0, f1, f2] = sorted;
        let c64 = Card64::from(cards.as_slice());

        match FlushTextureFlop::classify(f0, f1, f2) {
            FlushTextureFlop::Monotone(s) => {
                assert_eq!(c64.count_by_suit(s), 3);
            }
            FlushTextureFlop::Twotone(dbl, single) => {
                assert!(!dbl.const_eq(single));
                assert_eq!(c64.count_by_suit(dbl), 2);
                assert_eq!(c64.count_by_suit(single), 1);
            }
            FlushTextureFlop::Rainbow(s0, s1, s2) => {
                assert!(!s0.const_eq(s1) && !s0.const_eq(s2) && !s1.const_eq(s2));
                assert_eq!(c64.count_by_suit(s0), 1);
                assert_eq!(c64.count_by_suit(s1), 1);
                assert_eq!(c64.count_by_suit(s2), 1);
            }
        }
    }
}
