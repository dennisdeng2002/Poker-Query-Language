use super::binom;
use crate::{Card, Card64, CardCount, HandN};

/// Iterator over every `N`-card combination, short-deck when `SD` is true.
#[derive(Debug, Clone)]
pub struct HandIter<const SD: bool, const N: usize> {
    indices: [CardCount; N],
    done: bool,
    pub(crate) dead: Card64,
    yielded: usize,
}

#[allow(clippy::cast_possible_truncation)]
impl<const SD: bool, const N: usize> Default for HandIter<SD, N> {
    fn default() -> Self {
        let mut indices = [0; N];
        for i in 0..N as CardCount {
            indices[i as usize] = i;
        }
        Self {
            indices,
            done: N == 0,
            dead: Card64::EMPTY,
            yielded: 0,
        }
    }
}

impl<const SD: bool, const N: usize> HandIter<SD, N> {
    /// Exclude hands containing any of these dead cards.
    #[must_use]
    pub fn with_dead(mut self, dead: impl Into<Card64>) -> Self {
        self.dead |= dead.into();
        self
    }
}

impl<const SD: bool, const N: usize> Iterator for HandIter<SD, N> {
    type Item = HandN<N>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.done {
                return None;
            }

            let all = Card::all::<SD>();
            let max_i = all.len();

            let mut cards = [Card::default(); N];
            for i in 0..N {
                cards[i] = all[self.indices[i] as usize];
            }

            let mut pos = N - 1;
            self.indices[pos] += 1;

            while self.indices[pos] as usize >= max_i - (N - 1 - pos) {
                if pos == 0 {
                    self.done = true;
                    let hand = HandN::new(cards);
                    if (self.dead & Card64::from(hand)).is_empty() {
                        self.yielded += 1;
                        return Some(hand);
                    }
                    return None;
                }

                pos -= 1;
                self.indices[pos] += 1;
            }

            for i in (pos + 1)..N {
                self.indices[i] = self.indices[i - 1] + 1;
            }

            let hand = HandN::new(cards);
            if (self.dead & Card64::from(hand)).is_empty() {
                self.yielded += 1;
                return Some(hand);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }

        let n = const { if SD { Card::N_CARDS_SD } else { Card::N_CARDS } };
        let dead_count = (self.dead & Card64::all::<SD>()).count();
        let r = N;

        let len = binom(n.saturating_sub(dead_count) as usize, r);

        let remaining = len - self.yielded;
        (remaining, Some(remaining))
    }
}

impl<const SD: bool, const N: usize> ExactSizeIterator for HandIter<SD, N> {}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::*;

    fn handiter_vec<const N: usize, const SD: bool>() -> Vec<Vec<Card>> {
        HandN::<N>::iter_all::<SD>()
            .map(|hand| hand.to_vec())
            .collect()
    }

    fn itertool_vec<const N: usize, const SD: bool>() -> Vec<Vec<Card>> {
        Card::all::<SD>().iter().copied().combinations(N).collect()
    }

    #[test]
    fn test_hand_iter_holdem() {
        const SD: bool = false;
        assert_eq!(handiter_vec::<2, SD>(), itertool_vec::<2, SD>());
        assert_eq!(handiter_vec::<3, SD>(), itertool_vec::<3, SD>());
    }

    #[test]
    fn test_hand_iter_shortdeck() {
        const SD: bool = true;
        assert_eq!(handiter_vec::<2, SD>(), itertool_vec::<2, SD>());
        assert_eq!(handiter_vec::<3, SD>(), itertool_vec::<3, SD>());
    }

    #[test]
    fn test_hand_iter_with_dead() {
        let dead = c64!("As Kd");
        let iter = HandN::<2>::iter_all::<false>().with_dead(dead);
        assert_eq!(iter.len(), binom(50, 2));

        let hands: Vec<_> = iter.collect();
        assert_eq!(hands.len(), binom(50, 2));

        let as_card = Card::from_str("As").unwrap();
        let kd_card = Card::from_str("Kd").unwrap();
        for hand in hands {
            assert!(!hand.to_vec().contains(&as_card));
            assert!(!hand.to_vec().contains(&kd_card));
        }
    }

    #[test]
    fn test_size_hint_decrements() {
        let mut iter = HandN::<2>::iter_all::<false>().with_dead(c64!("As Kd"));
        let mut expected = binom(50, 2);

        assert_eq!(iter.size_hint(), (expected, Some(expected)));
        assert_eq!(iter.len(), expected);

        while iter.next().is_some() {
            expected -= 1;
            assert_eq!(iter.size_hint(), (expected, Some(expected)));
            assert_eq!(iter.len(), expected);
        }

        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn test_size_hint_when_done() {
        let mut iter = HandN::<2>::iter_all::<false>().with_dead(c64!("Ad Ac"));
        while iter.next().is_some() {}
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn test_last_hand_filtered_by_dead() {
        let dead = c64!("Ad Ac");
        let hands: Vec<_> = HandN::<2>::iter_all::<false>().with_dead(dead).collect();

        let ad = Card::from_str("Ad").unwrap();
        let ac = Card::from_str("Ac").unwrap();
        for hand in &hands {
            assert!(!hand.to_vec().contains(&ad));
            assert!(!hand.to_vec().contains(&ac));
        }
        assert_eq!(hands.len(), binom(50, 2));
    }
}
