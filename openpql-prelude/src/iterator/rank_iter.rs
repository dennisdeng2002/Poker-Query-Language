use crate::{CardCount, Rank, Rank16};

/// Iterator over every `Rank` in a `Rank16`.
#[derive(Debug, Clone)]
pub struct RankIter {
    r16: Rank16,
    idx: CardCount,
}

impl RankIter {
    pub(crate) const fn new(r16: Rank16) -> Self {
        Self { r16, idx: 0 }
    }
}

impl Iterator for RankIter {
    type Item = Rank;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < Rank::N_RANKS {
            let r = Rank::all::<false>()[self.idx as usize];
            self.idx += 1;

            if self.r16.contains_rank(r) {
                return Some(r);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = (self.r16.0 >> self.idx).count_ones() as usize;
        (count, Some(count))
    }
}

impl ExactSizeIterator for RankIter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order() {
        let iter = RankIter::new(Rank16::all::<false>());
        let values: Vec<_> = iter.collect();

        assert_eq!(Rank::all::<false>(), values);
    }

    #[test]
    fn test_size_hint_decrements() {
        let mut iter = RankIter::new(Rank16::all::<false>());
        let mut expected = Rank::N_RANKS as usize;

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
