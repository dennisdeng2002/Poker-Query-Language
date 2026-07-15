use crate::{Card, Card64, CardCount};

/// Iterator over cards in a `Card64`, ordered by rank then suit.
#[derive(Debug, Clone)]
pub struct CardIter {
    c64: Card64,
    idx: CardCount, // need this for column-major iter
    remaining: CardCount,
}

impl CardIter {
    pub(crate) const fn new(c64: Card64) -> Self {
        Self {
            c64,
            idx: 0,
            remaining: c64.count(),
        }
    }
}

impl Iterator for CardIter {
    type Item = Card;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < Card::N_CARDS {
            let c = Card::all::<false>()[self.idx as usize];
            self.idx += 1;

            if self.c64.contains_card(c) {
                self.remaining -= 1;
                return Some(c);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.remaining as usize;
        (count, Some(count))
    }
}

impl ExactSizeIterator for CardIter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order() {
        let iter = CardIter::new(Card64::all::<false>());
        let cards: Vec<_> = iter.collect();

        assert_eq!(cards, Card::all::<false>());
    }

    #[test]
    fn test_size_hint_decrements() {
        let mut iter = CardIter::new(Card64::all::<false>());
        let mut expected = Card::N_CARDS as usize;

        assert_eq!(iter.size_hint(), (expected, Some(expected)));
        assert_eq!(iter.len(), expected);

        while iter.next().is_some() {
            expected -= 1;
            assert_eq!(iter.size_hint(), (expected, Some(expected)));
            assert_eq!(iter.len(), expected);
        }

        assert_eq!(iter.size_hint(), (0, Some(0)));
    }
}
