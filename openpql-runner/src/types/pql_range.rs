use super::*;

pub struct PQLRange(
    pub(crate) FnCheckRange,
    RangeSrc,
    PQLGame,
    Option<LiteralHand>,
);

impl fmt::Debug for PQLRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PQLRange")
            .field(&self.1)
            .field(&self.2)
            .finish()
    }
}

impl PQLRange {
    #[inline]
    pub fn is_satisfied(&self, cs: &[PQLCard]) -> bool {
        (self.0)(cs)
    }

    /// The exact cards this range matches, if it denotes a single
    /// fully-specified hand (e.g. `"AsKsQsJsTs9s"`, no wildcards/ranges).
    /// Lets the sampler deal the hand directly instead of rejection-sampling
    /// card by card.
    #[inline]
    pub(crate) fn literal(&self) -> Option<&[PQLCard]> {
        self.3.as_deref()
    }
}

/// # Panics
/// `PQLRange` ensures valid range text
impl Clone for PQLRange {
    fn clone(&self) -> Self {
        (self.2, self.1.as_str()).try_into().unwrap()
    }
}

#[cfg(test)]
impl PQLRange {
    /// AK != KA but good for assertions in tests
    pub fn src_eq(&self, other: &Self) -> bool {
        self.1 == other.1 && self.2 == other.2
    }
}

impl TryFrom<(PQLGame, &str)> for PQLRange {
    type Error = PQLErrorKind;

    fn try_from((game, src): (PQLGame, &str)) -> Result<Self, Self::Error> {
        fn create_range<const N: usize, const SD: bool>(
            checker: RangeChecker<N, SD>,
            src: &str,
            game: PQLGame,
        ) -> PQLRange
        where
            [u8; N]: smallvec::Array<Item = u8>,
        {
            let literal = literal_hand(src, N);

            PQLRange(
                Box::new(move |cs: &[PQLCard]| checker.is_satisfied(cs)),
                src.to_string(),
                game,
                literal,
            )
        }

        match game {
            PQLGame::Holdem => Ok(create_range(
                RangeChecker::<2, false>::from_src(src)?,
                src,
                game,
            )),
            PQLGame::Omaha => Ok(create_range(
                RangeChecker::<4, false>::from_src(src)?,
                src,
                game,
            )),
            PQLGame::Omaha5 => Ok(create_range(
                RangeChecker::<5, false>::from_src(src)?,
                src,
                game,
            )),
            PQLGame::Omaha6 => Ok(create_range(
                RangeChecker::<6, false>::from_src(src)?,
                src,
                game,
            )),
            PQLGame::ShortDeck => Ok(create_range(
                RangeChecker::<2, true>::from_src(src)?,
                src,
                game,
            )),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_debug() {
        let range = PQLRange::try_from((PQLGame::default(), "AA")).unwrap();

        assert_eq!(format!("{range:?}"), r#"PQLRange("AA", Holdem)"#);
    }

    #[test]
    fn test_err() {
        for game in [PQLGame::Holdem, PQLGame::Omaha, PQLGame::ShortDeck] {
            let res = PQLRange::try_from((game, "AAAAK")).unwrap_err();

            assert_eq!(
                res,
                RangeError::TooManyCardsInRange((0, "AAAAK".len())).into()
            );
        }
    }

    #[quickcheck]
    fn test_clone(cards: CardN<2, true>) {
        let res = PQLRange::try_from((PQLGame::default(), "BB")).unwrap();
        let cloned = res.clone();

        assert_eq!(
            res.is_satisfied(cards.as_slice()),
            cloned.is_satisfied(cards.as_slice()),
        );
    }

    #[test]
    fn test_literal() {
        let range = PQLRange::try_from((PQLGame::Holdem, "AsKs")).unwrap();

        assert_eq!(range.literal(), Some(&[card!("As"), card!("Ks")][..]));
    }

    #[test]
    fn test_literal_preserves_source_order() {
        let range = PQLRange::try_from((PQLGame::Omaha, "3c4c2c5c")).unwrap();

        assert_eq!(
            range.literal(),
            Some(&[card!("3c"), card!("4c"), card!("2c"), card!("5c")][..]),
        );
    }

    #[test]
    fn test_not_literal() {
        // wildcards, pairs-as-ranks, and partial hands all fall back to the
        // general range checker rather than the fast path.
        for src in ["*", "AA", "As", "AsKs*", "AsKs2s3s+", "[As,Ks]QsJs"] {
            let range = PQLRange::try_from((PQLGame::Omaha, src)).unwrap();

            assert_eq!(range.literal(), None, "{src} should not be literal");
        }
    }
}
