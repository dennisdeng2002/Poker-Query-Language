use std::fmt;

use crate::{
    Board, Card, FlushingSuit, HandN, IsomorphicCard, IsomorphicHandN, Rank, Rank16, Suit, Suit4,
    card::Rank16Inner,
    isomorphism::util::{n_flush_suits, place_card},
};

/// Suit-isomorphic representative of the flop cards.
pub type IsomorphicFlop = IsomorphicHandN<{ Board::N_FLOP as usize }>;

/// Canonical suit-isomorphic representative of a board at any street.
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))] // LCOV_EXCL_LINE
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum IsomorphicBoardGto {
    /// No community cards dealt.
    #[default]
    Preflop,
    /// The flop cards, suit-relabeled and sorted.
    Flop(IsomorphicFlop),
    /// Flop plus the turn card, relabeled with the same suit map.
    Turn(IsomorphicFlop, IsomorphicCard),
    /// Flop, turn, and river cards, relabeled with the same suit map.
    River(IsomorphicFlop, IsomorphicCard, IsomorphicCard),
}

impl IsomorphicBoardGto {
    /// Returns the flop cards, or `None` preflop.
    #[must_use]
    #[inline]
    pub const fn flop(&self) -> Option<IsomorphicFlop> {
        match *self {
            Self::Preflop => None,
            Self::Flop(flop) | Self::Turn(flop, _) | Self::River(flop, _, _) => Some(flop),
        }
    }

    /// Returns the turn card, if dealt.
    #[must_use]
    #[inline]
    pub const fn turn(&self) -> Option<IsomorphicCard> {
        match *self {
            Self::Turn(_, turn) | Self::River(_, turn, _) => Some(turn),
            _ => None,
        }
    }

    /// Returns the river card, if dealt.
    #[must_use]
    #[inline]
    pub const fn river(&self) -> Option<IsomorphicCard> {
        match *self {
            Self::River(_, _, river) => Some(river),
            _ => None,
        }
    }

    /// Returns the set of ranks present on the board.
    pub const fn ranks(self) -> Rank16 {
        let mut res = Rank16(0);

        if let Some(IsomorphicHandN([f0, f1, f2])) = self.flop() {
            res.set(f0.rank);
            res.set(f1.rank);
            res.set(f2.rank);
        }

        if let Some(turn) = self.turn() {
            res.set(turn.rank);
        }

        if let Some(river) = self.river() {
            res.set(river.rank);
        }

        res
    }

    /// Returns the board ranks strictly above `rank`.
    pub const fn ranks_above(self, rank: Rank) -> Rank16 {
        let mask = !((1 << (rank as Rank16Inner + 1)) - 1);
        Rank16(self.ranks().0 & mask)
    }

    /// Iterates the relabeled cards in board order.
    pub fn iter(&self) -> impl Iterator<Item = IsomorphicCard> + '_ {
        self.flop()
            .map(|flop| flop.0)
            .into_iter()
            .flatten()
            .chain(self.turn())
            .chain(self.river())
    }

    /// Convert to a `Board` with concrete suits
    pub const fn to_board(&self) -> Board {
        match *self {
            Self::Flop(f) => create_board(&place_flop(f.0)),
            Self::Turn(f, t) => create_board(&place_turn(f.0, t)),
            Self::River(f, t, r) => create_board(&place_river(f.0, t, r)),
            Self::Preflop => Board::new(),
        }
    }
}

#[inline]
const fn create_board(cs: &[Card]) -> Board {
    match cs.len() {
        n if n == Board::N_FLOP as usize => Board::Flop(HandN([cs[0], cs[1], cs[2]])),

        n if n == Board::N_TURN as usize => Board::Turn(HandN([cs[0], cs[1], cs[2]]), cs[3]),

        _ => Board::River(HandN([cs[0], cs[1], cs[2]]), cs[3], cs[4]),
    }
}

#[inline]
const fn place_flop(
    flop: [IsomorphicCard; Board::N_FLOP as usize],
) -> [Card; Board::N_FLOP as usize] {
    let [f0, f1, f2] = flop;

    [
        Card::new(f0.rank, to_suit(f0.suit)),
        Card::new(f1.rank, to_suit(f1.suit)),
        Card::new(f2.rank, to_suit(f2.suit)),
    ]
}

#[inline]
const fn place_turn(
    flop: [IsomorphicCard; Board::N_FLOP as usize],
    turn: IsomorphicCard,
) -> [Card; Board::N_TURN as usize] {
    let [f0, f1, f2] = flop;
    let k = n_flush_suits(&[f0, f1, f2, turn]);

    let (c0, k) = place_card(f0, k);
    let (c1, k) = place_card(f1, k);
    let (c2, k) = place_card(f2, k);
    let (c3, _) = place_card(turn, k);

    [c0, c1, c2, c3]
}

#[inline]
const fn place_river(
    flop: [IsomorphicCard; Board::N_FLOP as usize],
    turn: IsomorphicCard,
    river: IsomorphicCard,
) -> [Card; Board::N_RIVER as usize] {
    let [c0, c1, c2, c3] = place_turn(flop, turn);

    let river_suit = match river.suit {
        FlushingSuit::N => {
            let n_flush = n_flush_suits(&[flop[0], flop[1], flop[2], turn]);
            let taken = taken_by_rank(river.rank, [c0, c1, c2, c3]);

            match (
                n_flush,
                taken.contains_suit(Suit::S),
                taken.contains_suit(Suit::H),
                taken.contains_suit(Suit::D),
            ) {
                (0, false, _, _) => Suit::S,
                (0..=1, _, false, _) => Suit::H,
                (0..=2, _, _, false) => Suit::D,
                _ => Suit::C,
            }
        }
        _ => to_suit(river.suit),
    };

    [c0, c1, c2, c3, Card::new(river.rank, river_suit)]
}

#[inline]
const fn taken_by_rank(rank: Rank, cs: [Card; 4]) -> Suit4 {
    let mut res = Suit4(0);
    if rank.const_eq(cs[0].rank) {
        res.set(cs[0].suit);
    }
    if rank.const_eq(cs[1].rank) {
        res.set(cs[1].suit);
    }
    if rank.const_eq(cs[2].rank) {
        res.set(cs[2].suit);
    }
    if rank.const_eq(cs[3].rank) {
        res.set(cs[3].suit);
    }
    res
}

#[inline]
const fn to_suit(s: FlushingSuit) -> Suit {
    match s {
        FlushingSuit::X => Suit::S,
        FlushingSuit::Y => Suit::H,
        FlushingSuit::N => Suit::D, // LCOV_EXCL_LINE — place_flop only sees X/Y/Z
        FlushingSuit::Z => Suit::C,
    }
}

impl fmt::Display for IsomorphicBoardGto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter().try_for_each(|card| write!(f, "{card}"))
    }
}

impl fmt::Debug for IsomorphicBoardGto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IsomorphicBoardGto<{self}>")
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::*;

    fn assert_iso(s: &str) {
        let (lhs, rhs) = s.split_once("->").unwrap();
        let (got, _) = board!(lhs).to_isomorphic();
        assert_eq!(
            got.to_string(),
            rhs.replace(char::is_whitespace, ""),
            "{:?}->{rhs}; but got {got}",
            board!(lhs),
        );
    }

    #[test]
    fn test_flop() {
        assert_iso("6s6hAs -> 6x6yAx");
        assert_iso("AsKsQs -> QxKxAx");
        assert_iso("AsKhQd -> QxKyAz");
    }

    #[test]
    fn test_turn() {
        assert_iso("AsKsQs Js -> QxKxAxJx");
        assert_iso("AsKsQs Jh -> QxKxAxJn");
        assert_iso("AsKhQd Jc -> QnKnAnJn");
        assert_iso("AsKsQh Jh -> QyKxAxJy");
    }

    #[test]
    fn test_river() {
        assert_iso("AsKsQs Js Ts -> QxKxAxJxTx");
        assert_iso("AsKsQh Jd Ts -> QnKxAxJnTx");
        assert_iso("AsKhQd Jc Ts -> QnKnAnJnTn");
    }

    /// `to_board` must produce a board that re-isomorphizes to `iso`.
    fn assert_roundtrip(s: &str) {
        let (iso, _) = board!(s).to_isomorphic();
        let (back, _) = iso.to_board().to_isomorphic();

        assert_eq!(back, iso, "{s}: {iso} -> {} -> {back}", iso.to_board());
    }

    #[test]
    fn test_place_river_suit_c_arm() {
        assert_roundtrip("AsAhAd Kh Ac");
    }

    #[test]
    fn test_to_board_roundtrip() {
        // Flop textures.
        assert_roundtrip("AsKsQs");
        assert_roundtrip("AsKsQh");
        assert_roundtrip("AsKhQd");
        // Turn: flush draw, double draw, and no-flush (all irrelevant).
        assert_roundtrip("AsKsQs Jh");
        assert_roundtrip("AsKsQh Jh");
        assert_roundtrip("AsKhQd Jc");
        // River: flush completes, flush on flop only, and no flush at all.
        assert_roundtrip("AsKsQh Jd Ts");
        assert_roundtrip("AsKsQs Jh Td");
        assert_roundtrip("AsKhQd Jc Ts");
    }

    #[quickcheck]
    fn test_to_board_roundtrip_all(board: Board) {
        let (iso, _) = board.to_isomorphic();
        let (back, _) = iso.to_board().to_isomorphic();

        assert_eq!(
            board.to_card64().count(),
            iso.to_board().to_card64().count()
        );
        assert_eq!(back, iso);
    }

    #[test]
    fn test_preflop_is_empty() {
        let (got, map) = Board::default().to_isomorphic();
        assert_eq!(got, IsomorphicBoardGto::default());
        assert_eq!(map.0, SuitMap::new().0);
    }

    #[test]
    fn test_ranks_above() {
        let (iso, _) = board!("AsKhQd Jc Ts").to_isomorphic();

        assert_eq!(iso.ranks_above(Rank::RA), Rank16::default());
        assert_eq!(iso.ranks_above(Rank::RK), r16!("A"));
        assert_eq!(iso.ranks_above(Rank::RQ), r16!("KA"));
        assert_eq!(iso.ranks_above(Rank::RT), r16!("JQKA"));
        assert_eq!(iso.ranks_above(Rank::R9), r16!("TJQKA"));
        assert_eq!(iso.ranks_above(Rank::R2), r16!("TJQKA"));
    }

    #[test]
    fn test_debug() {
        let v = board!("AsKhQd Jc Js").to_isomorphic().0;
        assert_eq!("IsomorphicBoardGto<QnKnAnJnJn>", format!("{v:?}"));
    }

    #[test]
    fn test_place_river() {
        assert_eq!(
            board!("AsKhQd Jc Js")
                .to_isomorphic()
                .0
                .to_board()
                .to_card64()
                .count(),
            5
        );

        assert_eq!(
            board!("AsKhQd Jc Qc")
                .to_isomorphic()
                .0
                .to_board()
                .to_card64()
                .count(),
            5
        );
    }

    #[test]
    fn test_ranks_above_empty() {
        let iso = IsomorphicBoardGto::default();
        assert_eq!(iso.ranks_above(Rank::R2), Rank16::default());
        assert_eq!(iso.ranks_above(Rank::RA), Rank16::default());
    }
}
