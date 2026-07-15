use super::{
    IsomorphicCard, IsomorphicHandN, isomorphic_preflop_holdem, isomorphic_preflop_omaha,
    isomorphic_preflop_omaha5,
    util::{n_flush_suits, place_card},
};
use crate::{Card, card::CardArray};

/// A variable-length hand of suit-isomorphic [`IsomorphicCard`]s.
pub type IsomorphicHand = CardArray<IsomorphicCard>;

impl IsomorphicHand {
    /// Materializes this suit-isomorphic hand as a concrete [`Hand`] with
    /// suits placed in canonical order.
    ///
    /// [`Hand`]: crate::Hand
    #[must_use]
    pub const fn to_hand(self) -> crate::Hand {
        let cards = self.as_slice();
        let mut next = n_flush_suits(cards);

        let mut hand = crate::Hand::new();
        let mut i = 0;
        while i < cards.len() {
            let (card, k) = place_card(cards[i], next);
            next = k;
            hand.push(card);
            i += 1;
        }

        hand
    }

    #[must_use]
    pub const fn no_flush(cards: &[Card]) -> Self {
        match cards.len() {
            2 => Self::from_arr(IsomorphicHandN::<2>::no_flush(cards).0),
            3 => Self::from_arr(IsomorphicHandN::<3>::no_flush(cards).0),
            4 => Self::from_arr(IsomorphicHandN::<4>::no_flush(cards).0),
            5 => Self::from_arr(IsomorphicHandN::<5>::no_flush(cards).0),
            _ => unimplemented!(), // LCOV_EXCL_LINE
        }
    }

    #[must_use]
    pub(crate) const fn preflop(cards: &[Card]) -> Self {
        match cards.len() {
            2 => Self::from_arr(isomorphic_preflop_holdem(cards).0),
            4 => Self::from_arr(isomorphic_preflop_omaha(cards).0),
            5 => Self::from_arr(isomorphic_preflop_omaha5(cards).0),
            _ => unimplemented!(), // LCOV_EXCL_LINE
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::{IsomorphicHand, cards, isocards};

    fn assert_iso(cards: &str, expected: &str) {
        assert_eq!(
            IsomorphicHand::preflop(&cards!(cards)),
            isocards!(expected).into(),
        );
    }

    fn assert_to_hand(iso: &str, expected: &str) {
        assert_eq!(
            IsomorphicHand::from(isocards!(iso)).to_hand(),
            cards!(expected).into(),
        );
    }

    #[test]
    fn test_preflop() {
        assert_iso("AdKc", "KnAn");
        assert_iso("AdKd", "KxAx");

        assert_iso("AdKcQhJh", "JxQxKnAn");
        assert_iso("AsKsQhJh", "JxQxKyAy");
        assert_iso("AhKhQsJs", "JxQxKyAy");

        assert_iso("AhKhQsJsTs", "TxJxQxKyAy");
        assert_iso("AsKhQdJcTs", "TxJnQnKnAx");
    }

    #[test]
    fn test_to_hand_no_flush() {
        assert_to_hand("KnAn", "KsAh");
        assert_to_hand("AnKnQnJnTn", "AsKhQdJcTs");

        assert_to_hand("JxQxKnAn", "JsQsKhAd");
        assert_to_hand("JxQxKyAy", "JsQsKhAh");
        assert_to_hand("AzKnQn", "AdKhQd");
    }
}
