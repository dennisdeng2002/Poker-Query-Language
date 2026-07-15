use super::MAX_HOLECARDS;
use crate::StackVec;

/// A variable-length, sorted-on-demand hand of up to [`MAX_HOLECARDS`] cards of
/// type `C`, stored entirely on the stack.
pub type CardArray<C> = StackVec<C, MAX_HOLECARDS>;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use std::iter::once;

    use crate::{Card, CardN, Hand, IsomorphicHand, isocard};

    #[quickcheck]
    fn test_from_arr(cards: CardN<5>) {
        fn assert_eq<const N: usize>(cs: &[Card]) {
            let arr = <[Card; N]>::try_from(&cs[0..N]).unwrap();

            assert_eq!(Hand::from(arr), Hand::from_arr(arr));
        }

        assert_eq::<0>(cards.as_slice());
        assert_eq::<1>(cards.as_slice());
        assert_eq::<2>(cards.as_slice());
        assert_eq::<3>(cards.as_slice());
        assert_eq::<4>(cards.as_slice());
        assert_eq::<5>(cards.as_slice());
    }

    #[test]
    fn new_push_and_accessors() {
        let mut hand = IsomorphicHand::new();
        assert!(hand.as_slice().is_empty());

        hand.push(isocard!("Ax"));
        hand.push(isocard!("Kx"));
        assert_eq!(hand.as_slice(), &[isocard!("Ax"), isocard!("Kx")]);
        assert_eq!(hand.len(), 2);
        assert_eq!(hand[0], isocard!("Ax"));

        let collected: IsomorphicHand = once(isocard!("Ax")).collect();
        assert_eq!(collected.len(), 1);
    }
}

#[cfg(all(test, feature = "serde"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests_serde {
    use serde_test::{Token, assert_de_tokens_error, assert_tokens};

    use crate::{IsomorphicHand, isocard};

    #[test]
    fn serde_round_trip() {
        let hand = IsomorphicHand::from([isocard!("Ax"), isocard!("Kx")]);
        assert_tokens(
            &hand,
            &[
                Token::Seq { len: Some(2) },
                Token::Str("Ax"),
                Token::Str("Kx"),
                Token::SeqEnd,
            ],
        );
    }

    #[test]
    fn deserialize_propagates_error() {
        assert_de_tokens_error::<IsomorphicHand>(
            &[Token::Seq { len: Some(1) }, Token::Bool(true)],
            "invalid type: boolean `true`, expected an isomorphic card string like \"Ah\"",
        );
    }
}

#[cfg(all(test, feature = "speedy"))]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests_speedy {
    use speedy::{Readable, Writable};

    use crate::{IsomorphicHand, isocard};

    #[test]
    fn speedy_round_trip() {
        let hand = IsomorphicHand::from([isocard!("Ax"), isocard!("Kx"), isocard!("Qy")]);
        let bytes = hand.write_to_vec().unwrap();
        let back = IsomorphicHand::read_from_buffer(&bytes).unwrap();
        assert_eq!(hand, back);
    }
}
