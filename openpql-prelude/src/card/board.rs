use std::{fmt, hash::Hash};

use crate::{Card, Card64, CardCount, Flop, Suit4, card::util::const_cmp_opt};

/// Builds a [`Board`] from a string of cards.
#[macro_export]
macro_rules! board {
    ($s:expr) => {
        $crate::Board::from(cards!($s).as_slice())
    };
}

/// Community cards across flop, turn, and river.
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))] // LCOV_EXCL_LINE
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Board {
    /// No community cards dealt.
    #[default]
    Preflop,
    /// Three flop cards.
    Flop(Flop),
    /// Flop plus the turn card.
    Turn(Flop, Card),
    /// Flop, turn, and river cards.
    River(Flop, Card, Card),
}

impl Board {
    /// Slice index of the turn card.
    pub const IDX_TURN: usize = 3;
    /// Slice index of the river card.
    pub const IDX_RIVER: usize = 4;
    /// Card count preflop.
    pub const N_PREFLOP: CardCount = 0;
    /// Card count on the flop.
    pub const N_FLOP: CardCount = 3;
    /// Card count through the turn.
    pub const N_TURN: CardCount = 4;
    /// Card count through the river.
    pub const N_RIVER: CardCount = 5;

    #[inline]
    pub(crate) const fn new() -> Self {
        Self::Preflop
    }

    /// Returns the flop cards, or `None` preflop.
    #[must_use]
    #[inline]
    pub const fn flop(&self) -> Option<Flop> {
        match *self {
            Self::Preflop => None,
            Self::Flop(flop) | Self::Turn(flop, _) | Self::River(flop, _, _) => Some(flop),
        }
    }

    /// Returns the turn card, if dealt.
    #[must_use]
    #[inline]
    pub const fn turn(&self) -> Option<Card> {
        match *self {
            Self::Turn(_, turn) | Self::River(_, turn, _) => Some(turn),
            _ => None,
        }
    }

    /// Returns the river card, if dealt.
    #[must_use]
    #[inline]
    pub const fn river(&self) -> Option<Card> {
        match *self {
            Self::River(_, _, river) => Some(river),
            _ => None,
        }
    }

    /// Creates a board from a slice ordered flop, turn, river.
    #[must_use]
    pub fn from_slice(cards: &[Card]) -> Self {
        if cards.len() < Self::N_FLOP as usize {
            return Self::Preflop;
        }

        let flop = Flop::from_slice(&cards[0..Self::N_FLOP as usize]);

        match (
            cards.get(Self::IDX_TURN).copied(),
            cards.get(Self::IDX_RIVER).copied(),
        ) {
            (Some(turn), Some(river)) => Self::River(flop, turn, river),
            (Some(turn), None) => Self::Turn(flop, turn),
            _ => Self::Flop(flop),
        }
    }

    /// Returns `true` if no flop is dealt.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        matches!(self, Self::Preflop)
    }

    /// Returns the number of dealt cards (0, 3, 4, or 5).
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn count(&self) -> CardCount {
        match self {
            Self::Preflop => Self::N_PREFLOP,
            Self::Flop(_) => Self::N_FLOP,
            Self::Turn(..) => Self::N_TURN,
            Self::River(..) => Self::N_RIVER,
        }
    }

    /// Returns the number of cards still to be dealt (0, 1, 2, or 5).
    #[must_use]
    #[inline]
    pub const fn future_cards(&self) -> CardCount {
        Self::N_RIVER - self.count()
    }

    /// Same as [`Self::count`], as a `usize`.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.count() as usize
    }

    /// Returns suits still able to make a flush by the river.
    #[must_use]
    pub const fn flush_suits(&self) -> Suit4 {
        const N_FLUSH: CardCount = 5;

        self.to_card64()
            .suits_with_min_count(N_FLUSH - self.future_cards())
    }

    /// Returns the dealt cards as a vector.
    #[must_use]
    pub fn to_vec(&self) -> Vec<Card> {
        match *self {
            Self::River(flop, turn, river) => vec![flop[0], flop[1], flop[2], turn, river],
            Self::Turn(flop, turn) => vec![flop[0], flop[1], flop[2], turn],
            Self::Flop(flop) => flop.to_vec(),
            Self::Preflop => vec![],
        }
    }

    /// Returns an iterator over dealt cards.
    pub fn iter(&self) -> impl Iterator<Item = Card> + '_ {
        self.flop()
            .map(|flop| flop.0)
            .into_iter()
            .flatten()
            .chain(self.turn())
            .chain(self.river())
    }

    /// Returns `true` if the board contains `card`.
    #[must_use]
    pub const fn contains_card(&self, card: Card) -> bool {
        #[inline]
        const fn inner_eq(op: Option<Card>, rhs: Card) -> bool {
            match op {
                Some(lhs) => lhs.const_eq(rhs),
                None => false,
            }
        }

        if let Some(flop) = self.flop()
            && flop.contains_card(card)
        {
            return true;
        }

        inner_eq(self.turn(), card) || inner_eq(self.river(), card)
    }

    /// Const-context equality, equivalent to [`PartialEq::eq`].
    #[inline]
    #[must_use]
    pub const fn const_eq(self, other: Self) -> bool {
        const fn card_eq(a: Option<Card>, b: Option<Card>) -> bool {
            match (a, b) {
                (Some(x), Some(y)) => x.const_eq(y),
                (None, None) => true,
                _ => false,
            }
        }

        let flop_eq = match (self.flop(), other.flop()) {
            (Some(x), Some(y)) => x.const_eq(y),
            (None, None) => true,
            _ => false,
        };

        flop_eq && card_eq(self.turn(), other.turn()) && card_eq(self.river(), other.river())
    }

    /// Const-context less-than, equivalent to [`PartialOrd::lt`].
    ///
    /// Orders lexicographically by flop, turn, then river, with `None`
    /// ordered before `Some`.
    #[inline]
    #[must_use]
    pub const fn const_lt(self, other: Self) -> bool {
        const_cmp_opt!(self.flop(), other.flop());
        const_cmp_opt!(self.turn(), other.turn());
        const_cmp_opt!(self.river(), other.river());
        false
    }

    /// Returns a copy of the board with `card` set as the turn.
    ///
    /// Returns the board unchanged when no flop has been dealt.
    #[must_use]
    pub const fn with_turn(self, card: Card) -> Self {
        match self {
            Self::Flop(flop) | Self::Turn(flop, _) => Self::Turn(flop, card),
            Self::River(flop, _, river) => Self::River(flop, card, river),
            Self::Preflop => Self::Preflop,
        }
    }

    /// Returns a copy of the board with `card` set as the river.
    ///
    /// Returns the board unchanged when no turn has been dealt.
    #[must_use]
    pub const fn with_river(self, card: Card) -> Self {
        match self {
            Self::Turn(flop, turn) | Self::River(flop, turn, _) => Self::River(flop, turn, card),
            other => other,
        }
    }

    /// Returns the dealt cards packed into a [`Card64`] bitset.
    #[must_use]
    pub const fn to_card64(self) -> Card64 {
        Card64(self.to_c64_flop().0 | self.to_c64_turn().0 | self.to_c64_river().0)
    }

    pub(crate) const fn to_c64_flop(self) -> Card64 {
        match self.flop() {
            Some(flop) => flop.to_c64(),
            None => Card64::EMPTY,
        }
    }

    pub(crate) const fn to_c64_turn(self) -> Card64 {
        match self.turn() {
            Some(turn) => turn.to_c64(),
            None => Card64::EMPTY,
        }
    }

    pub(crate) const fn to_c64_river(self) -> Card64 {
        match self.river() {
            Some(river) => river.to_c64(),
            None => Card64::EMPTY,
        }
    }
}

impl From<&[Card]> for Board {
    fn from(xs: &[Card]) -> Self {
        Self::from_slice(xs)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter().try_for_each(|card| write!(f, "{card}"))
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Board<{self}>")
    }
}

impl From<Board> for Card64 {
    fn from(board: Board) -> Self {
        board.iter().collect()
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl quickcheck::Arbitrary for Board {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let cards = crate::CardN::<{ Self::N_RIVER as usize }>::arbitrary(g);
        let street = crate::Street::arbitrary(g);

        Self::from_slice(&cards.as_ref()[..street.board_card_count() as usize])
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_const_cmp() {
        assert!(board!("TsJsQsKsAs").const_lt(board!("TsJsQsKsAh")));
        assert!(!board!("TsJsQsKsAs").const_lt(board!("TsJsQsKsAs")));

        assert!(board!("").const_eq(board!("")));
        assert!(board!("TsJsQs").const_eq(board!("TsJsQs")));
        assert!(!board!("TsJsQs").const_eq(board!("")));
        assert!(board!("TsJsQsKsAs").const_eq(board!("TsJsQsKsAs")));
        assert!(!board!("TsJsQsKsAs").const_eq(board!("TsJsQsKsAh")));
        assert!(!board!("TsJsQsKsAs").const_eq(board!("TsJsQsKs")));
    }

    #[quickcheck]
    fn test_from_slice(cs: CardN<5>) {
        assert_eq!(
            Board::from([cs[0], cs[1], cs[2], cs[3], cs[4]].as_slice()),
            Board::from([cs[2], cs[1], cs[0], cs[3], cs[4]].as_slice()),
        );
        assert_ne!(
            Board::from([cs[0], cs[1], cs[2], cs[3], cs[4]].as_slice()),
            Board::from([cs[0], cs[1], cs[2], cs[4], cs[3]].as_slice()),
        );
    }

    #[test]
    fn test_empty() {
        assert!(Board::default().is_empty());
    }

    #[test]
    fn test_board_iter_and_len() {
        let board = board!("Qd As Kh Jc");

        assert_eq!(board.iter().collect::<Vec<_>>(), cards!("Qd Kh As Jc"));
        assert_eq!(board.len(), 4);
        assert_eq!(Board::default().len(), 0);
        assert_eq!(board!("AsKsQs").len(), 3);
        assert_eq!(board!("AsKsQsJs").len(), 4);
        assert_eq!(board!("AsKsQsJsTs").len(), 5);
    }

    #[quickcheck]
    fn test_board_contains_card(board: Board, card: Card) {
        assert_eq!(board.iter().any(|x| x == card), board.contains_card(card));
        assert!(!Board::default().contains_card(card));
    }

    #[quickcheck]
    fn test_to_vec(board: Board) {
        let expected = match Street::from(board) {
            Street::Preflop => vec![],
            Street::Flop => board.flop().unwrap().to_vec(),
            Street::Turn => board
                .flop()
                .unwrap()
                .0
                .into_iter()
                .chain([board.turn().unwrap()])
                .collect(),
            Street::River => board
                .flop()
                .unwrap()
                .0
                .into_iter()
                .chain([board.turn().unwrap(), board.river().unwrap()])
                .collect(),
        };

        assert_eq!(board.to_vec(), expected);
    }

    #[test]
    fn test_display() {
        let cards = cards!["AsKhQd3s2s"];
        let board = Board::from_slice(&cards);

        assert_eq!(format!("{board}"), "QdKhAs3s2s");
        assert_eq!(format!("{board:?}"), "Board<QdKhAs3s2s>");
    }

    #[test]
    fn test_with_turn_and_river() {
        let flop = board!("QdKhAs");
        let c0 = card!("Jc");
        let c1 = card!("Ts");
        let turn = flop.with_turn(c0);
        let river = turn.with_river(c1);

        assert_eq!(turn, board!("QdKhAs Jc"));
        assert_eq!(river, board!("QdKhAs Jc Ts"));
        assert_eq!(river.with_turn(card!("2s")), board!("QdKhAs 2s Ts"));
    }

    #[test]
    fn test_with_turn_and_river_no_op() {
        let c = card!("2s");

        assert_eq!(Board::Preflop.with_turn(c), Board::Preflop);
        assert_eq!(Board::Preflop.with_river(c), Board::Preflop);

        let flop = board!("QdKhAs");
        assert_eq!(flop.with_river(c), flop);
    }

    #[test]
    fn test_to_card64() {
        let board = board!("QdKhAsJcTs");

        assert_eq!(board.to_card64(), Card64::from(board));
        assert_eq!(Board::default().to_card64(), Card64::EMPTY);
    }

    #[test]
    fn test_to_c64() {
        let cards = cards!("AsKhQdJcTs");

        for i in 3..=5 {
            let board = Board::from_slice(&cards[0..i]);
            let card64 = Card64::from(board);

            for j in 0..i {
                assert!(card64.contains_card(cards[j]));
            }

            assert_eq!(card64.count() as usize, i);
        }
    }

    #[quickcheck]
    fn test_flush_suits(board: Board) {
        let cs = board.to_card64();

        assert_eq!(
            board.flush_suits(),
            Suit::ARR_ALL
                .into_iter()
                .filter(|&suit| { cs.count_by_suit(suit) + board.future_cards() >= 5 })
                .collect()
        );
    }
}
