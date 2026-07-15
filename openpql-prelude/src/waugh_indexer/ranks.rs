use super::{Card64, PerSuitRanks, Rank16, Suit, SuitConfig, WaughIndex, mem};

const N_SUITS: usize = Suit::N_SUITS as usize;

pub struct Ranks<const ROUNDS: usize>([PerSuitRanks<ROUNDS>; N_SUITS]);

impl<const ROUNDS: usize> Ranks<ROUNDS> {
    pub const fn from_hands(hands: &[Card64; ROUNDS]) -> Self {
        let mut res = [PerSuitRanks::EMPTY; N_SUITS];
        let mut round = 0;

        while round < ROUNDS {
            let mut suit = 0;
            while suit < N_SUITS {
                res[suit].0[round] = hands[round].ranks_by_suit(Suit::ARR_ALL[suit]);
                suit += 1;
            }
            round += 1;
        }

        Self::const_sort(&mut res);
        Self(res)
    }

    #[inline]
    const fn const_sort(suits: &mut [PerSuitRanks<ROUNDS>; N_SUITS]) {
        let mut i = 1;
        while i < N_SUITS {
            let mut j = i;
            while j > 0 && suits[j - 1].canonical_lt(&suits[j]) {
                let tmp = suits[j - 1];
                suits[j - 1] = suits[j];
                suits[j] = tmp;
                j -= 1;
            }
            i += 1;
        }
    }

    pub(crate) const fn to_suit_config(&self) -> SuitConfig<ROUNDS> {
        let mut res = SuitConfig::EMPTY;
        let mut suit = 0;

        while suit < N_SUITS {
            res.0[suit] = self.0[suit].to_rank_count();
            suit += 1;
        }

        res
    }

    pub(crate) const fn per_suit_index(
        &self,
    ) -> (
        SuitConfig<ROUNDS>,
        (WaughIndex, WaughIndex, WaughIndex, WaughIndex),
    ) {
        let conf = self.to_suit_config();
        let idx_s = self.0[0].index();
        let idx_h = self.0[1].index();
        let idx_d = self.0[2].index();
        let idx_c = self.0[3].index();

        (conf, (idx_s, idx_h, idx_d, idx_c))
    }

    pub(crate) const fn unindex(
        indices: &[WaughIndex; N_SUITS],
        suit_config: &SuitConfig<ROUNDS>,
    ) -> Self {
        Self([
            PerSuitRanks::unindex(suit_config.0[0], indices[0]),
            PerSuitRanks::unindex(suit_config.0[1], indices[1]),
            PerSuitRanks::unindex(suit_config.0[2], indices[2]),
            PerSuitRanks::unindex(suit_config.0[3], indices[3]),
        ])
    }

    pub(crate) const fn to_hand(&self) -> [Card64; ROUNDS] {
        let mut res = [Card64::EMPTY; ROUNDS];
        let mut round = 0;

        while round < ROUNDS {
            res[round] = make_c64([
                self.0[3].0[round],
                self.0[2].0[round],
                self.0[1].0[round],
                self.0[0].0[round],
            ]);
            round += 1;
        }

        res
    }
}

const fn make_c64(ranks: [Rank16; N_SUITS]) -> Card64 {
    unsafe { mem::transmute([ranks[0].0, ranks[1].0, ranks[2].0, ranks[3].0]) }
}
