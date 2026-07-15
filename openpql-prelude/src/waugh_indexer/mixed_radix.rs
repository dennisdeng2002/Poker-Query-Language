use super::{WaughIndex, util};

#[derive(Clone, Debug)]
pub struct MixedRadix<const N: usize>
where
    [WaughIndex; N]: Copy,
{
    pub block_sizes: [WaughIndex; N],
    offsets: [WaughIndex; N],
}

impl<const N: usize> MixedRadix<N> {
    pub const fn new(block_sizes: [WaughIndex; N]) -> Self {
        let mut offsets = block_sizes;
        util::suffix_product(&mut offsets);

        Self {
            block_sizes,
            offsets,
        }
    }

    pub const fn size(&self) -> WaughIndex {
        self.offsets[0] * self.block_sizes[0]
    }

    pub const fn encode(&self, digits: [WaughIndex; N]) -> WaughIndex {
        util::dot_product(digits, self.offsets)
    }

    pub const fn decode(&self, mut idx: WaughIndex) -> [WaughIndex; N] {
        let mut digits = [0; N];
        let mut round = 0;

        while round < N {
            digits[round] = idx / self.offsets[round];
            idx %= self.offsets[round];
            round += 1;
        }

        digits
    }
}

#[cfg(test)]
mod tests {
    use std::array;

    use super::*;
    use crate::waugh_indexer::RoundIndex;

    #[quickcheck]
    fn test_roundtrip(block_sizes: [RoundIndex; 4], digits: [RoundIndex; 4]) {
        let block_sizes = block_sizes.map(|b| WaughIndex::from(b) + 1);
        let digits = array::from_fn(|i| WaughIndex::from(digits[i]) % block_sizes[i]);

        let radix = MixedRadix::new(block_sizes);

        assert_eq!(radix.decode(radix.encode(digits)), digits);
    }

    #[test]
    fn test_encode() {
        let indexer = MixedRadix::new([200, 30, 5]);
        assert_eq!(100 * 30 * 5 + 20 * 5 + 2, indexer.encode([100, 20, 2]));
    }
}
