use crate::{
    Board, Card64, FlopHandCategory, Game, HandRating,
    eval::{
        flop::{eval_flop_holdem, eval_flop_omaha},
        rating::{eval_holdem, eval_omaha, eval_omaha5, eval_shortdeck},
    },
};

impl Game {
    /// Returns the rating of `player` against `board` for this variant.
    #[must_use]
    pub fn eval_rating(self, player: Card64, board: Card64) -> HandRating {
        match self {
            Self::Holdem => eval_holdem(player | board),
            Self::ShortDeck => eval_shortdeck(player | board),
            Self::Omaha => eval_omaha(player, board),
            Self::Omaha5 => eval_omaha5(player, board),
        }
    }

    /// Returns the flop-hand category of `player` against `board` for this variant.
    #[must_use]
    pub fn eval_flop_category(self, player: Card64, board: Board) -> FlopHandCategory {
        match self {
            Self::Holdem | Self::ShortDeck => eval_flop_holdem(player, board),
            Self::Omaha | Self::Omaha5 => eval_flop_omaha(player, board),
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::*;

    #[quickcheck]
    fn test_eval_rating_args(cs: CardN<5>) {
        let c5 = Card64::from(cs.as_slice());
        let (c3, c2): (CardN<3>, CardN<2>) = cs.into();
        let c3 = Card64::from(c3);
        let c2 = Card64::from(c2);

        let r1 = Game::Holdem.eval_rating(c3, c2);
        let r2 = Game::Holdem.eval_rating(c5, Card64::default());
        let r3 = Game::Holdem.eval_rating(Card64::default(), c5);

        assert_eq!(r1, r2);
        assert_eq!(r1, r3);
    }

    #[test]
    fn test_eval_rating() {
        assert_eq!(
            Game::Omaha.eval_rating(c64!("Ks Qh 8s 9h"), c64!("7h 7c 7d As Ah")),
            mk_rating(HandType::Trips, "7", "KQ")
        );
        assert_eq!(
            Game::Omaha5.eval_rating(c64!("Ks Qh 8s 9h 2c"), c64!("7h 7c 7d As Ah")),
            mk_rating(HandType::Trips, "7", "KQ")
        );
        assert_eq!(
            Game::ShortDeck.eval_rating(c64!("Kh As Ah Ac Ad 6d 6c"), Card64::default()),
            mk_rating(HandType::Quads, "A", "K")
        );
    }

    #[test]
    fn test_eval_flop_cat() {
        assert_eq!(
            Game::Holdem.eval_flop_category(c64!("7c 8c"), board!("7s 8h Tc")),
            FlopHandCategory::BottomTwo,
        );

        assert_eq!(
            Game::Omaha.eval_flop_category(c64!("7c 8c 2s 3s"), board!("7s 8h Tc")),
            FlopHandCategory::BottomTwo,
        );
    }
}
