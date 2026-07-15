use super::{CardCount, Rank, RoundIndex, WaughIndex, fmt, util};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RankCount<const ROUNDS: usize>(pub [CardCount; ROUNDS])
where
    [CardCount; ROUNDS]: Copy;

impl<const ROUNDS: usize> RankCount<ROUNDS> {
    pub const EMPTY: Self = Self([0; ROUNDS]);

    pub fn enumerate(num_cards: &[CardCount; ROUNDS]) -> Vec<Self> {
        let total: usize = num_cards.iter().map(|&n| n as usize + 1).product();
        let mut result = Vec::with_capacity(total);
        let mut current = [0; ROUNDS];

        loop {
            result.push(Self(current));

            let mut round = ROUNDS;
            loop {
                if round == 0 {
                    return result;
                }
                round -= 1;
                if current[round] < num_cards[round] {
                    current[round] += 1;
                    break;
                }
                current[round] = 0;
            }
        }
    }

    pub const fn to_block_sizes(self) -> [RoundIndex; ROUNDS] {
        let mut res = [0; ROUNDS];
        let mut round = 0;
        let mut remaining_cards = Rank::N_RANKS;

        while round < ROUNDS {
            res[round] = util::binom_coeff_u16(remaining_cards, self.0[round]);
            remaining_cards -= self.0[round];
            round += 1;
        }

        res
    }

    pub const fn size(&self) -> WaughIndex {
        let mut round = 0;
        let sizes = self.to_block_sizes();
        let mut acc = 1;

        while round < ROUNDS {
            acc *= sizes[round] as WaughIndex;
            round += 1;
        }

        acc
    }

    pub(crate) const fn const_eq(&self, other: Self) -> bool {
        let mut round = 0;

        while round < ROUNDS {
            if self.0[round] != other.0[round] {
                return false;
            }
            round += 1;
        }

        true
    }

    pub(crate) const fn const_lt(&self, other: Self) -> bool {
        let mut round = 0;

        while round < ROUNDS {
            if self.0[round] < other.0[round] {
                return true;
            } else if self.0[round] > other.0[round] {
                return false;
            }
            round += 1;
        }

        false
    }
}

impl<const ROUNDS: usize> fmt::Debug for RankCount<ROUNDS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, c) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ",")?;
            }
            write!(f, "{c:?}")?;
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::waugh_indexer::PerSuitRanks;

    impl<const ROUNDS: usize> quickcheck::Arbitrary for RankCount<ROUNDS> {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            PerSuitRanks::arbitrary(g).to_rank_count()
        }
    }

    #[test]
    fn test_enumerate() {
        let expected = [
            [0, 0],
            [0, 1],
            [0, 2],
            [1, 0],
            [1, 1],
            [1, 2],
            [2, 0],
            [2, 1],
            [2, 2],
            [3, 0],
            [3, 1],
            [3, 2],
        ]
        .map(RankCount);
        assert_eq!(expected.as_slice(), RankCount::enumerate(&[3, 2]));
    }

    #[test]
    fn test_const_eq() {
        assert!(RankCount([]).const_eq(RankCount([])));
        assert!(RankCount([1]).const_eq(RankCount([1])));
        assert!(RankCount([0, 3]).const_eq(RankCount([0, 3])));
        assert!(!RankCount([0, 3]).const_eq(RankCount([0, 2])));
    }

    const R: usize = 4;
    #[quickcheck]
    fn test_const_lt(a: RankCount<R>, b: RankCount<R>) {
        assert_eq!(a < b, a.const_lt(b));
    }

    #[test]
    fn test_debug() {
        let obj = RankCount([1, 3, 0, 2]);
        assert_eq!("(1,3,0,2)", format!("{obj:?}"));
    }
}
