use super::{CardCount, Rank16, RoundBitsInner, RoundIndex, colex_decode, colex_encode, fmt, str};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RoundBits(pub RoundBitsInner);

impl RoundBits {
    pub const fn from_rank16(rank16: Rank16) -> Self {
        Self(rank16.0)
    }

    pub const fn to_rank16(self) -> Rank16 {
        Rank16(self.0)
    }

    pub const fn index(self) -> RoundIndex {
        colex_encode(self.0)
    }

    pub const fn unindex(index: RoundIndex, n: CardCount, r: CardCount) -> Self {
        Self(colex_decode(index, n, r))
    }

    #[expect(clippy::cast_possible_truncation, reason = "num of bits < u8::MAX")]
    pub const fn count(self) -> CardCount {
        self.0.count_ones() as CardCount
    }

    #[must_use]
    pub const fn shift(self, other: Self) -> Self {
        debug_assert!(self.0 & other.0 == 0);
        let mut bits = self.0;
        let mut res: RoundBitsInner = 0;
        while bits != 0 {
            let pos = bits.trailing_zeros();
            let below = (other.0 & (((1 as RoundBitsInner) << pos) - 1)).count_ones();
            res |= (1 as RoundBitsInner) << (pos - below);
            bits &= bits - 1;
        }
        Self(res)
    }

    #[must_use]
    pub const fn unshift(self, other: Self) -> Self {
        let mut res: RoundBitsInner = 0;
        let mut new_pos = 0;
        let mut pos = 0;
        while pos < RoundBitsInner::BITS {
            if other.0 >> pos & 1 == 0 {
                if self.0 >> new_pos & 1 == 1 {
                    res |= (1 as RoundBitsInner) << pos;
                }
                new_pos += 1;
            }
            pos += 1;
        }
        Self(res)
    }

    pub(crate) const fn const_or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl fmt::Debug for RoundBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bits = format!("{:0width$b}", self.0, width = RoundBitsInner::BITS as usize);
        let grouped: String = bits
            .as_bytes()
            .chunks(4)
            .map(|c| str::from_utf8(c).unwrap())
            .collect::<Vec<_>>()
            .join("_");

        write!(f, "{grouped}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        let bits = RoundBits(0b0000_0000_1010_0101);

        assert_eq!("0000_0000_1010_0101", format!("{bits:?}"));
    }

    #[quickcheck]
    fn test_index_roundtrip(v: RoundBitsInner) {
        let bits = RoundBits(v);
        let n = CardCount::try_from(RoundBitsInner::BITS).unwrap();
        let r = CardCount::try_from(v.count_ones()).unwrap();

        assert_eq!(bits, RoundBits::unindex(bits.index(), n, r));
    }

    #[quickcheck]
    fn test_shift_roundtrip(a: RoundBitsInner, b: RoundBitsInner) {
        let bits = RoundBits(a & !b);
        let used = RoundBits(b);

        assert_eq!(bits.shift(used).unshift(used), bits);
    }

    #[quickcheck]
    fn test_shift(a: RoundBitsInner, b: RoundBitsInner) {
        let bits = RoundBits(a & !b);
        let used = RoundBits(b);

        let mut expected: RoundBitsInner = 0;
        let mut new_pos = 0;
        for pos in 0..RoundBitsInner::BITS {
            if used.0 >> pos & 1 == 1 {
                continue;
            }
            if bits.0 >> pos & 1 == 1 {
                expected |= (1 as RoundBitsInner) << new_pos;
            }
            new_pos += 1;
        }

        assert_eq!(bits.shift(used).0, expected);
    }
}
