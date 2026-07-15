use super::{Rank16, RankCount, RoundBits, ShiftedRanks, WaughIndex};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PerSuitRanks<const ROUNDS: usize>(pub [Rank16; ROUNDS])
where
    [Rank16; ROUNDS]: Copy;

impl<const ROUNDS: usize> PerSuitRanks<ROUNDS> {
    pub const EMPTY: Self = Self([Rank16(0); ROUNDS]);

    #[inline]
    pub(crate) const fn canonical_lt(&self, other: &Self) -> bool {
        let a = self.to_rank_count();
        let b = other.to_rank_count();
        if a.const_lt(b) {
            return true;
        }
        if b.const_lt(a) {
            return false;
        }
        self.index() > other.index()
    }

    pub(crate) const fn to_rank_count(self) -> RankCount<ROUNDS> {
        let mut res = RankCount::EMPTY;
        let mut round = 0;

        while round < ROUNDS {
            res.0[round] = self.0[round].count();
            round += 1;
        }

        res
    }

    pub(crate) const fn to_shifted_ranks(self) -> ShiftedRanks<ROUNDS> {
        let mut res = ShiftedRanks::EMPTY;
        let mut used = RoundBits(0);
        let mut round = 0;

        while round < ROUNDS {
            let bits = RoundBits::from_rank16(self.0[round]);

            res.0[round] = bits.shift(used);
            used = used.const_or(bits);
            round += 1;
        }

        res
    }

    pub(crate) const fn index(&self) -> WaughIndex {
        self.to_shifted_ranks().index()
    }

    pub(crate) const fn unindex(rank_count: RankCount<ROUNDS>, idx: WaughIndex) -> Self {
        ShiftedRanks::unindex(rank_count, idx).to_ranks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<const ROUNDS: usize> quickcheck::Arbitrary for PerSuitRanks<ROUNDS> {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut ranks = [Rank16(0); ROUNDS];

            let available = Rank16::arbitrary(g);

            for rank in available.iter() {
                let round = usize::arbitrary(g) % ROUNDS;
                ranks[round].set(rank);
            }

            Self(ranks)
        }
    }
}
