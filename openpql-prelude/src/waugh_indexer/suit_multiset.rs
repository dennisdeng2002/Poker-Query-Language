use colex_multi_decode as decode;
use colex_multi_encode as encode;

use super::{MixedRadix, Suit, WaughIndex, colex_multi_decode, colex_multi_encode};

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub enum SuitMultiset {
    MMMM(MixedRadix<1>),
    MMMS(MixedRadix<2>),
    SMMM(MixedRadix<2>),
    MMSS(MixedRadix<3>),
    SMMS(MixedRadix<3>),
    SSMM(MixedRadix<3>),
    SSSS(MixedRadix<4>),
    MMNN(MixedRadix<2>),
}

impl SuitMultiset {
    pub const fn size(&self) -> WaughIndex {
        match self {
            Self::MMMM(r) => r.size(),
            Self::MMMS(r) | Self::SMMM(r) | Self::MMNN(r) => r.size(),
            Self::MMSS(r) | Self::SMMS(r) | Self::SSMM(r) => r.size(),
            Self::SSSS(r) => r.size(),
        }
    }

    pub(crate) const fn index(
        &self,
        (s, h, d, c): (WaughIndex, WaughIndex, WaughIndex, WaughIndex),
    ) -> WaughIndex {
        match self {
            Self::MMMM(radix) => radix.encode([encode([s, h, d, c])]),
            Self::MMMS(radix) => radix.encode([encode([s, h, d]), c]),
            Self::SMMM(radix) => radix.encode([s, encode([h, d, c])]),
            Self::MMSS(radix) => radix.encode([encode([s, h]), d, c]),
            Self::SMMS(radix) => radix.encode([s, encode([h, d]), c]),
            Self::SSMM(radix) => radix.encode([s, h, encode([d, c])]),
            Self::SSSS(radix) => radix.encode([s, h, d, c]),
            Self::MMNN(radix) => radix.encode([encode([s, h]), encode([d, c])]),
        }
    }

    pub(crate) const fn unindex(&self, index: WaughIndex) -> [WaughIndex; Suit::N_SUITS as usize] {
        match self {
            Self::MMMM(radix) => decode(radix.decode(index)[0], radix.block_sizes[0], 4),
            Self::MMMS(radix) => {
                let [multi, c] = radix.decode(index);
                let [s, h, d] = decode(multi, radix.block_sizes[0], 3);
                [s, h, d, c]
            }
            Self::SMMM(radix) => {
                let [s, multi] = radix.decode(index);
                let [h, d, c] = decode(multi, radix.block_sizes[1], 3);
                [s, h, d, c]
            }
            Self::MMSS(radix) => {
                let [multi, d, c] = radix.decode(index);
                let [s, h] = decode(multi, radix.block_sizes[0], 2);
                [s, h, d, c]
            }
            Self::SMMS(radix) => {
                let [s, multi, c] = radix.decode(index);
                let [h, d] = decode(multi, radix.block_sizes[1], 2);
                [s, h, d, c]
            }
            Self::SSMM(radix) => {
                let [s, h, multi] = radix.decode(index);
                let [d, c] = decode(multi, radix.block_sizes[2], 2);
                [s, h, d, c]
            }
            Self::SSSS(radix) => radix.decode(index),
            Self::MMNN(radix) => {
                let [multi0, multi1] = radix.decode(index);
                let [s, h] = decode(multi0, radix.block_sizes[0], 2);
                let [d, c] = decode(multi1, radix.block_sizes[1], 2);
                [s, h, d, c]
            }
        }
    }
}
