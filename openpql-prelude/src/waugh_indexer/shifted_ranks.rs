use super::{
    MixedRadix, PerSuitRanks, Rank, Rank16, RankCount, RoundBits, RoundIndex, WaughIndex, util,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ShiftedRanks<const ROUNDS: usize>(pub [RoundBits; ROUNDS])
where
    [Rank16; ROUNDS]: Copy;

impl<const ROUNDS: usize> ShiftedRanks<ROUNDS> {
    pub const EMPTY: Self = Self([RoundBits(0); ROUNDS]);

    pub(crate) const fn to_ranks(self) -> PerSuitRanks<ROUNDS> {
        let mut res = PerSuitRanks::EMPTY;
        let mut used = RoundBits(0);
        let mut round = 0;

        while round < ROUNDS {
            let bits = self.0[round].unshift(used);
            res.0[round] = bits.to_rank16();
            used = used.const_or(bits);

            round += 1;
        }

        res
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

    const fn to_indices(self) -> [RoundIndex; ROUNDS] {
        let mut res = [0; ROUNDS];
        let mut round = 0;

        while round < ROUNDS {
            res[round] = self.0[round].index();
            round += 1;
        }

        res
    }

    pub(crate) const fn index(&self) -> WaughIndex {
        let radix = MixedRadix::new(util::to_wide(self.to_rank_count().to_block_sizes()));

        radix.encode(util::to_wide(self.to_indices()))
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "C(13,r) <= C(13,7) < u16::MAX"
    )]
    pub(crate) const fn unindex(rank_count: RankCount<ROUNDS>, idx: WaughIndex) -> Self {
        let radix = MixedRadix::new(util::to_wide(rank_count.to_block_sizes()));
        let digits = radix.decode(idx);

        let mut res = Self::EMPTY;
        let mut remaining_ranks = Rank::N_RANKS;
        let mut round = 0;

        while round < ROUNDS {
            let count = rank_count.0[round];

            res.0[round] = RoundBits::unindex(digits[round] as RoundIndex, remaining_ranks, count);
            remaining_ranks -= count;
            round += 1;
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<const ROUNDS: usize> quickcheck::Arbitrary for ShiftedRanks<ROUNDS> {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            PerSuitRanks::arbitrary(g).to_shifted_ranks()
        }
    }

    #[quickcheck]
    fn test_ranks_roundtrip(ranks: ShiftedRanks<4>) {
        assert_eq!(ranks, ranks.to_ranks().to_shifted_ranks());
    }

    #[quickcheck]
    fn test_index_roundtrip(ranks: ShiftedRanks<4>) {
        let rank_count = ranks.to_rank_count();
        assert_eq!(ShiftedRanks::unindex(rank_count, ranks.index()), ranks);
    }
}
