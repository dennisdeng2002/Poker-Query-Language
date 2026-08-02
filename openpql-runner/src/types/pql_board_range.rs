use super::*;

pub struct PQLBoardRange(
    pub(crate) FnCheckRange,
    RangeSrc,
    PQLGame,
    Option<LiteralHand>,
);

impl fmt::Debug for PQLBoardRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PQLBoardRange")
            .field(&self.1)
            .field(&self.2)
            .finish()
    }
}

impl PQLBoardRange {
    #[inline]
    pub fn is_satisfied(&self, cs: &[PQLCard]) -> bool {
        (self.0)(cs)
    }

    /// The exact board this range matches, if it denotes a single
    /// fully-specified 5-card board (no wildcards/ranges). Lets the sampler
    /// deal it directly instead of rejection-sampling card by card.
    #[inline]
    pub(crate) fn literal(&self) -> Option<&[PQLCard]> {
        self.3.as_deref()
    }
}

/// # Panics
/// `PQLBoardRange` ensures valid range text
impl Clone for PQLBoardRange {
    fn clone(&self) -> Self {
        (self.2, self.1.as_str()).try_into().unwrap()
    }
}

#[cfg(test)]
impl PQLBoardRange {
    /// AK != KA but good for assertions in tests
    pub fn src_eq(&self, other: &Self) -> bool {
        self.1 == other.1 && self.2 == other.2
    }
}

impl Default for PQLBoardRange {
    fn default() -> Self {
        Self(Box::new(|_| true), "*".into(), PQLGame::default(), None)
    }
}

impl TryFrom<(PQLGame, &str)> for PQLBoardRange {
    type Error = PQLErrorKind;

    fn try_from((game, src): (PQLGame, &str)) -> Result<Self, Self::Error> {
        fn create_range<const SD: bool>(
            checker: BoardRangeChecker<SD>,
            src: &str,
            game: PQLGame,
        ) -> PQLBoardRange {
            let literal = literal_hand(src, PQLBoard::N_RIVER as usize);

            PQLBoardRange(
                Box::new(move |cs: &[PQLCard]| checker.is_satisfied(cs)),
                src.to_string(),
                game,
                literal,
            )
        }

        if game == PQLGame::ShortDeck {
            Ok(create_range(
                BoardRangeChecker::<true>::from_src(src)?,
                src,
                game,
            ))
        } else {
            Ok(create_range(
                BoardRangeChecker::<false>::from_src(src)?,
                src,
                game,
            ))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_debug() {
        let range = PQLBoardRange::default();

        assert_eq!(format!("{range:?}"), r#"PQLBoardRange("*", Holdem)"#);
    }

    #[test]
    fn test_err() {
        for game in [PQLGame::Holdem, PQLGame::Omaha, PQLGame::ShortDeck] {
            let res = PQLBoardRange::try_from((game, "AAAAKK")).unwrap_err();

            assert_eq!(
                res,
                RangeError::TooManyCardsInRange((0, "AAAAKK".len())).into()
            );
        }
    }

    #[quickcheck]
    fn test_clone(cards: CardN<5, true>) {
        let res = PQLBoardRange::try_from((PQLGame::default(), "BB")).unwrap();
        let cloned = res.clone();

        assert_eq!(
            res.is_satisfied(cards.as_slice()),
            cloned.is_satisfied(cards.as_slice()),
        );

        let res = PQLBoardRange::try_from((PQLGame::ShortDeck, "BB")).unwrap();
        let cloned = res.clone();

        assert_eq!(
            res.is_satisfied(cards.as_slice()),
            cloned.is_satisfied(cards.as_slice()),
        );
    }

    #[test]
    fn test_literal_preserves_street_order() {
        // Board order is meaningful (flop is 0..3, turn is 3, river is 4),
        // unlike a hole-card hand, so this must not be reordered.
        let range = PQLBoardRange::try_from((PQLGame::Holdem, "AhKdQc2s3h")).unwrap();

        assert_eq!(
            range.literal(),
            Some(
                &[
                    card!("Ah"),
                    card!("Kd"),
                    card!("Qc"),
                    card!("2s"),
                    card!("3h"),
                ][..]
            ),
        );
    }

    #[test]
    fn test_not_literal() {
        // A partial board (flop only) still needs the turn/river sampled,
        // so it isn't a fast-path candidate even though the flop is fixed.
        for src in ["*", "AhKdQc", "AA"] {
            let range = PQLBoardRange::try_from((PQLGame::Holdem, src)).unwrap();

            assert_eq!(range.literal(), None, "{src} should not be literal");
        }
    }
}
