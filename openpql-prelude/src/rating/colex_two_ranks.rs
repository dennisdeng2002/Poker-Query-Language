use super::{ColexTwoRanksIdx, RankPosition};
use crate::{Rank16, card::Rank16Inner};

#[inline]
const fn nc2(n: ColexTwoRanksIdx) -> ColexTwoRanksIdx {
    n * (n - 1) / 2
}

#[inline]
const fn colex_encode(positions: (RankPosition, RankPosition)) -> ColexTwoRanksIdx {
    let (high, low) = positions;
    nc2(high) + low
}

/// idx = hi * (hi - 1) / 2 => hi^2 - hi - 2 * idx = 0
#[inline]
#[expect(clippy::cast_possible_truncation, reason = "rank position < 16")]
const fn colex_decode(index: ColexTwoRanksIdx) -> (RankPosition, RankPosition) {
    type DoubleInt = u16;
    let delta = 8 * index as DoubleInt + 1;
    let high = delta.isqrt().div_ceil(2) as RankPosition;
    let low = index - nc2(high);

    (high, low)
}

/// Returns the position of the most significant set bit.
#[inline]
#[expect(clippy::cast_possible_truncation, reason = "rank position < 16")]
pub(super) const fn pos_msb(v: Rank16Inner) -> RankPosition {
    const MAX_ZEROS: RankPosition = 15;
    MAX_ZEROS - (v.leading_zeros() as RankPosition)
}

#[derive(Debug, Clone, Copy)]
pub struct ColexTwoRanks(pub(crate) ColexTwoRanksIdx);

impl ColexTwoRanks {
    pub(crate) const MASK_USED: ColexTwoRanksIdx = 0b0111_1111;

    pub(crate) const fn from_r16(r16: Rank16) -> Self {
        let high = pos_msb(r16.0);
        let low = pos_msb(r16.0 & !(1 << high));

        Self(colex_encode((high, low)))
    }

    pub(crate) const fn to_r16(self) -> Rank16 {
        let (high, low) = colex_decode(self.0);

        Rank16((1 << high) | (1 << low))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_from_r16() {
        assert_eq!(ColexTwoRanks::from_r16(r16!("32")).0, 0);
        assert_eq!(ColexTwoRanks::from_r16(r16!("42")).0, 1);
        assert_eq!(ColexTwoRanks::from_r16(r16!("43")).0, 2);
        assert_eq!(ColexTwoRanks::from_r16(r16!("52")).0, 3);
        assert_eq!(ColexTwoRanks::from_r16(r16!("62")).0, 6);
        assert_eq!(ColexTwoRanks::from_r16(r16!("72")).0, 10);
        assert_eq!(ColexTwoRanks::from_r16(r16!("82")).0, 15);
        assert_eq!(ColexTwoRanks::from_r16(r16!("92")).0, 21);
        assert_eq!(ColexTwoRanks::from_r16(r16!("T2")).0, 28);
        assert_eq!(ColexTwoRanks::from_r16(r16!("J2")).0, 36);
        assert_eq!(ColexTwoRanks::from_r16(r16!("Q2")).0, 45);
        assert_eq!(ColexTwoRanks::from_r16(r16!("K2")).0, 55);
        assert_eq!(ColexTwoRanks::from_r16(r16!("A2")).0, 66);
        assert_eq!(ColexTwoRanks::from_r16(r16!("AK")).0, 77);
    }

    #[quickcheck]
    fn test_bijection(r1: Rank, r2: Rank) -> TestResult {
        if r1 == r2 {
            return TestResult::discard();
        }

        let r16 = Rank16::from([r1, r2].as_slice());
        let idx = ColexTwoRanks::from_r16(r16);

        TestResult::from_bool(r16 == idx.to_r16())
    }
}
