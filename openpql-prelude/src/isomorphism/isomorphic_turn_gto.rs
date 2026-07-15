//! Suit-isomorphic canonical form of a flop-and-turn board.

use crate::{
    Board, Card, Flop, IsomorphicCard, SuitMap,
    isomorphism::{FlushTextureTurn, IsomorphicFlopGto},
};

/// Canonical suit-isomorphic representative of a flop-and-turn board.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct IsomorphicTurnGto {
    /// The three flop cards, suit-relabeled and sorted.
    pub(crate) flop: [IsomorphicCard; Board::N_FLOP as usize],
    /// The turn card, relabeled with the same suit map.
    pub(crate) turn: IsomorphicCard,
}

impl IsomorphicTurnGto {
    /// Canonical representative of `flop` plus `turn` and the [`SuitMap`] that produced it.
    #[inline]
    pub(crate) const fn from_turn(flop: Flop, turn: Card) -> (Self, SuitMap) {
        let [f0, f1, f2] = flop.0;
        let map = FlushTextureTurn::classify(&[f0, f1, f2, turn]).to_suit_map();

        (
            Self {
                flop: IsomorphicFlopGto::relabel(f0, f1, f2, map).0,
                turn: map.iso_card(turn),
            },
            map,
        )
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::*;

    fn iso_turn(s: &str) -> IsomorphicTurnGto {
        let cs = isocards!(s);

        IsomorphicTurnGto {
            flop: [cs[0], cs[1], cs[2]],
            turn: cs[3],
        }
    }

    fn from_str(s: &str) -> IsomorphicTurnGto {
        let b = board!(s);

        IsomorphicTurnGto::from_turn(b.flop().unwrap(), b.turn().unwrap()).0
    }

    fn assert_iso_turn(s: &str) {
        let (lhs, rhs) = s.split_once("->").unwrap();

        assert_eq!(from_str(lhs), iso_turn(rhs), "{lhs}->{rhs}");
    }

    fn gen_iso_turn() -> FxHashSet<IsomorphicTurnGto> {
        let mut set = FxHashSet::default();
        for flop in Flop::iter_all::<true>() {
            for &turn in Card::all::<true>() {
                if !flop.contains_card(turn) {
                    set.insert(IsomorphicTurnGto::from_turn(flop, turn).0);
                }
            }
        }
        set
    }

    #[test]
    fn test_iso_turn() {
        assert_eq!(gen_iso_turn().len(), 13_761);
    }

    #[test]
    fn test_monotone() {
        assert_iso_turn("AsKsQs Js -> QxKxAxJx");
        assert_iso_turn("AhKhQh Jh -> QxKxAxJx");
    }

    #[test]
    fn test_flop_monotone_turn_offsuit() {
        assert_iso_turn("AsKsQs Jh -> QxKxAxJn");
        assert_iso_turn("AsKsQs Jd -> QxKxAxJn");
    }

    #[test]
    fn test_rainbow() {
        assert_iso_turn("AsKhQd Jc -> QnKnAnJn");
        assert_iso_turn("AcKdJh Ts -> JnKnAnTn");
    }

    #[test]
    fn test_two_two() {
        assert_iso_turn("AsKsQh Jh -> QyKxAxJy");
        assert_iso_turn("AhKhQs Js -> QyKxAxJy");
    }

    #[test]
    fn test_twotone() {
        assert_iso_turn("AcKcQs Jh -> QnKxAxJn");
    }

    #[test]
    fn test_flop_and_turn_not_interchangeable() {
        assert_ne!(from_str("AsKhQd Jc"), from_str("AsKhJc Qd"));
    }

    #[quickcheck]
    fn test_suit_permutation_invariant(cards: CardN<4>, a: Suit, b: Suit) {
        let swap = |s: Suit| {
            if s == a {
                b
            } else if s == b {
                a
            } else {
                s
            }
        };
        let remap = |c: Card| Card::new(c.rank, swap(c.suit));

        let cs = cards.as_slice();
        let flop = Flop::from_slice(&cs[0..3]);
        let turn = cs[3];

        let permuted: Vec<Card> = cs[0..3].iter().copied().map(remap).collect();
        let permuted_flop = Flop::from_slice(&permuted);

        assert_eq!(
            IsomorphicTurnGto::from_turn(flop, turn).0,
            IsomorphicTurnGto::from_turn(permuted_flop, remap(turn)).0,
        );
    }
}
