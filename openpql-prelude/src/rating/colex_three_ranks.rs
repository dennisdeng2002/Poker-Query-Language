use super::{ColexThreeRanksIdx, RankPosition, colex_two_ranks::pos_msb};
use crate::Rank16;

#[inline]
const fn nc2(n: RankPosition) -> ColexThreeRanksIdx {
    let n = n as ColexThreeRanksIdx;
    n * (n - 1) / 2
}

#[inline]
const fn nc3(n: RankPosition) -> ColexThreeRanksIdx {
    nc2(n) * (n as ColexThreeRanksIdx - 2) / 3 // (n-2)*(n-1)*n/(3*2)
}

#[inline]
const fn colex_encode(positions: (RankPosition, RankPosition, RankPosition)) -> ColexThreeRanksIdx {
    let (high, mid, low) = positions;
    nc3(high) + nc2(mid) + low as ColexThreeRanksIdx
}

#[inline]
const fn largest_nc3_le(index: ColexThreeRanksIdx) -> RankPosition {
    let mut high = 2;
    while nc3(high + 1) <= index {
        high += 1;
    }
    high
}

#[inline]
#[expect(clippy::cast_possible_truncation, reason = "rank position < 16")]
pub const fn colex_decode(index: ColexThreeRanksIdx) -> (RankPosition, RankPosition, RankPosition) {
    let high = largest_nc3_le(index);
    let rem = index - nc3(high);

    let delta = 8 * rem + 1;
    let mid = delta.isqrt().div_ceil(2) as RankPosition;
    let low = (rem - nc2(mid)) as RankPosition;

    (high, mid, low)
}

#[derive(Debug, Clone, Copy)]
pub struct ColexThreeRanks(pub(crate) ColexThreeRanksIdx);

impl ColexThreeRanks {
    pub(crate) const MASK_USED: ColexThreeRanksIdx = 0b1_1111_1111;

    pub(crate) const fn from_r16(r16: Rank16) -> Self {
        let high = pos_msb(r16.0);
        let mid = pos_msb(r16.0 & !(1 << high));
        let low = pos_msb(r16.0 & !(1 << high | 1 << mid));

        Self(colex_encode((high, mid, low)))
    }

    pub(crate) const fn to_r16(self) -> Rank16 {
        let (high, mid, low) = colex_decode(self.0);

        Rank16((1 << high) | (1 << mid) | (1 << low))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_from_r16() {
        assert_eq!(ColexThreeRanks::from_r16(r16!("432")).0, 0);
        assert_eq!(ColexThreeRanks::from_r16(r16!("532")).0, 1);
        assert_eq!(ColexThreeRanks::from_r16(r16!("542")).0, 2);
        assert_eq!(ColexThreeRanks::from_r16(r16!("543")).0, 3);
        assert_eq!(ColexThreeRanks::from_r16(r16!("632")).0, 4);
        assert_eq!(ColexThreeRanks::from_r16(r16!("732")).0, 10);
        assert_eq!(ColexThreeRanks::from_r16(r16!("832")).0, 20);
        assert_eq!(ColexThreeRanks::from_r16(r16!("932")).0, 35);
        assert_eq!(ColexThreeRanks::from_r16(r16!("T32")).0, 56);
        assert_eq!(ColexThreeRanks::from_r16(r16!("J32")).0, 84);
        assert_eq!(ColexThreeRanks::from_r16(r16!("Q32")).0, 120);
        assert_eq!(ColexThreeRanks::from_r16(r16!("K32")).0, 165);
        assert_eq!(ColexThreeRanks::from_r16(r16!("A32")).0, 220);
        assert_eq!(ColexThreeRanks::from_r16(r16!("AKQ")).0, 285);
    }

    #[quickcheck]
    fn test_bijection(r1: Rank, r2: Rank, r3: Rank) -> TestResult {
        let r16 = Rank16::from([r1, r2, r3].as_slice());
        if r16.count() != 3 {
            return TestResult::discard();
        }

        let idx = ColexThreeRanks::from_r16(r16);

        TestResult::from_bool(r16 == idx.to_r16())
    }
}
