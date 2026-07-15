use super::card_array::CardArray;
use crate::Card;

/// A variable-length hand of concrete [`Card`]s.
pub type Hand = CardArray<Card>;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use crate::{Hand, cards};

    #[test]
    fn hand_over_concrete_cards() {
        let mut hand = Hand::new();
        for c in cards!("KsAs") {
            hand.push(c);
        }
        let sorted = Hand::sorted(hand);
        assert!(sorted.as_slice().is_sorted());
        assert_eq!(sorted.len(), 2);
    }
}
