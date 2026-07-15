use crate::{
    Board, Card, CardCount, HandN, HoleCardRule, IsomorphicBoardGto, IsomorphicHand,
    IsomorphicHandN, Suit, SuitMap,
    isomorphism::{
        FlushTextureFlop, FlushTextureRiver, FlushTextureTurn, IsomorphicFlopGto,
        IsomorphicRiverGto, IsomorphicTurnGto, util,
    },
};

type IsoBoard = IsomorphicHand;
impl Board {
    /// Canonical suit-isomorphic form (for GTO) of `board` and the [`crate::SuitMap`] that produced it.
    pub const fn to_isomorphic_pair(self, player: &[Card]) -> (IsoBoard, IsomorphicHand) {
        match self {
            Self::Preflop => (IsomorphicHand::new(), IsomorphicHand::preflop(player)),
            Self::Flop(HandN([f0, f1, f2])) => FlushTextureFlop::to_pair([f0, f1, f2], player),
            Self::Turn(HandN([f0, f1, f2]), t) => {
                FlushTextureTurn::to_pair([f0, f1, f2, t], player)
            }
            Self::River(HandN([f0, f1, f2]), t, r) => {
                FlushTextureRiver::to_pair([f0, f1, f2, t, r], player)
            }
        }
    }
    /// Canonical suit-isomorphic form (for GTO) of `board` and the [`SuitMap`] that produced it.
    pub const fn to_isomorphic(self) -> (IsomorphicBoardGto, SuitMap) {
        match (self.flop(), self.turn(), self.river()) {
            (Some(flop), None, _) => {
                let (flop, map) = IsomorphicFlopGto::from_flop(flop);

                (IsomorphicBoardGto::Flop(IsomorphicHandN(flop.0)), map)
            }
            (Some(flop), Some(turn), None) => {
                let (turn, map) = IsomorphicTurnGto::from_turn(flop, turn);

                (
                    IsomorphicBoardGto::Turn(IsomorphicHandN(turn.flop), turn.turn),
                    map,
                )
            }
            (Some(flop), Some(turn), Some(river)) => {
                let (river, map) = IsomorphicRiverGto::from_river(flop, turn, river);

                (
                    IsomorphicBoardGto::River(IsomorphicHandN(river.flop), river.turn, river.river),
                    map,
                )
            }
            _ => (IsomorphicBoardGto::Preflop, SuitMap::new()),
        }
    }
}

const N_FLUSH: CardCount = 5;
const N_OMAHA_FLUSH: CardCount = 2;

impl HoleCardRule {
    pub(crate) const fn can_make_flush(
        self,
        suit: Suit,
        board_count: CardCount,
        player: &[Card],
    ) -> bool {
        let n = util::count_suit(player, suit);

        match self {
            Self::UseAny => n + board_count >= N_FLUSH,
            Self::UseExactlyTwo => n >= N_OMAHA_FLUSH && board_count + N_OMAHA_FLUSH >= N_FLUSH,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::*;

    fn iso(s: &str) -> IsomorphicHand {
        isocards!(s).into()
    }

    fn assert_pair(board: &str, player: &str, expected: (&str, &str)) {
        let (b, p) = board!(board).to_isomorphic_pair(&cards!(player));

        assert_eq!(b, iso(expected.0));
        assert_eq!(p, iso(expected.1));
    }

    #[test]
    fn test_to_isomorphic_pair() {
        assert_pair("", "AdKc", ("", "KnAn"));
        assert_pair("", "AdKcQhJh", ("", "JxQxKnAn"));

        assert_pair("AhKsQd", "JcTh", ("QnKnAn", "TnJn"));
        assert_pair("AhKsQd", "JhTh", ("QnKnAx", "TxJx"));
        assert_pair("AhKhQh", "JsTd", ("QxKxAx", "TnJn"));

        assert_pair("AhKsQd9c", "JhTd", ("9nQnKnAn", "TnJn"));
        assert_pair("AhKhQs9c", "JhTh", ("9nQnKxAx", "TxJx"));

        assert_pair("AhKsQd9c2s", "JhTd", ("2n9nQnKnAn", "TnJn"));
        assert_pair("AhKhQh9s2c", "JhTh", ("2n9nQxKxAx", "TxJx"));
    }
}
