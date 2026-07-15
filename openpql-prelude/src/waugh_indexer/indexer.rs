use super::{Card64, CardCount, Ranks, SuitConfig, SuitMultiset, WaughIndex, util};

/// Maps a hand of cards to a dense, suit-isomorphic index using Kevin Waugh's algorithm.
#[derive(Clone, Debug)]
pub struct WaughIndexer<const ROUNDS: usize> {
    suit_configs: Vec<SuitConfig<ROUNDS>>,
    multisets: Vec<SuitMultiset>,
    offsets: Vec<WaughIndex>,
}

impl<const ROUNDS: usize> WaughIndexer<ROUNDS> {
    pub fn new(rounds: [CardCount; ROUNDS]) -> Self {
        let suit_configs = SuitConfig::enumerate(&rounds);
        let multisets: Vec<_> = suit_configs.iter().map(SuitConfig::to_multiset).collect();
        let mut offsets: Vec<_> = multisets.iter().map(SuitMultiset::size).collect();
        util::prefix_sum(&mut offsets);

        Self {
            suit_configs,
            multisets,
            offsets,
        }
    }

    pub fn size(&self) -> WaughIndex {
        let last = self.multisets.len() - 1;
        self.offsets[last] + self.multisets[last].size()
    }

    pub fn index(&self, hands: &[Card64; ROUNDS]) -> WaughIndex {
        let ranks = Ranks::from_hands(hands);
        let (suit_config, indices) = ranks.per_suit_index();
        let i = suit_config.const_binary_search(&self.suit_configs);
        let multiset = &self.multisets[i];

        let index = multiset.index(indices);

        index + self.offsets[i]
    }

    pub fn unindex(&self, index: WaughIndex) -> [Card64; ROUNDS] {
        let mut i = self.offsets.len(); // TODO: binary search
        while i > 0 {
            i -= 1;
            if self.offsets[i] < index {
                break;
            }
        }

        let indices = self.multisets[i].unindex(index - self.offsets[i]);

        Ranks::unindex(&indices, &self.suit_configs[i]).to_hand()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CardN, c64};

    fn to_canonical<const N: usize>(hand: [Card64; N]) -> [Card64; N] {
        Ranks::from_hands(&hand).to_hand()
    }

    const A: usize = 3;
    const B: usize = 5;
    #[quickcheck]
    fn test_roundtrip(cards: CardN<{ A + B }>) {
        let (cards_a, cards_b): (CardN<A>, CardN<B>) = cards.into();
        let (cards_a, cards_b): (Card64, Card64) = (cards_a.into(), cards_b.into());

        let indexer = WaughIndexer::new([
            CardCount::try_from(A).unwrap(),
            CardCount::try_from(B).unwrap(),
        ]);
        assert_eq!(
            indexer.unindex(indexer.index(&[cards_a, cards_b])),
            to_canonical([cards_a, cards_b])
        );
    }

    fn assert_roundtrip(cards: Card64, count: CardCount) {
        let indexer = WaughIndexer::new([count]);
        assert_eq!(
            indexer.unindex(indexer.index(&[cards])),
            to_canonical([cards])
        );
    }

    #[test]
    fn test_mmmm() {
        assert_roundtrip(c64!("AsKhQdTc"), 4);
    }

    #[test]
    fn test_mmnn() {
        assert_roundtrip(c64!("AsKh"), 2);
    }

    #[test]
    fn test_mmms() {
        assert_roundtrip(c64!("AsKhQdJcTc"), 5);
    }

    #[test]
    fn test_smmm() {
        assert_roundtrip(c64!("AsKhQd"), 3);
    }

    #[test]
    fn test_mmss() {
        assert_roundtrip(c64!("AsKsKh"), 3);
    }

    #[test]
    fn test_smms() {
        assert_roundtrip(c64!("AsKsKhQd"), 4);
    }

    #[test]
    fn test_ssmm() {
        assert_roundtrip(c64!("AsKsAhKhQd"), 5);
    }

    #[test]
    fn test_ssss() {
        assert_roundtrip(c64!("AsKsQsAhKhAd"), 6);
    }

    #[test]
    fn test_size() {
        assert_eq!(WaughIndexer::new([3]).size(), 1755);
    }
}
