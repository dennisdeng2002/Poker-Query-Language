use super::{CardCount, MixedRadix, RankCount, Suit, SuitMultiset, fmt, util};

const N_SUITS: usize = Suit::N_SUITS as usize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SuitConfig<const ROUNDS: usize>(pub [RankCount<ROUNDS>; N_SUITS]);

impl<const ROUNDS: usize> SuitConfig<ROUNDS> {
    pub const EMPTY: Self = Self([RankCount::EMPTY; N_SUITS]);

    pub fn enumerate(num_cards: &[CardCount; ROUNDS]) -> Vec<Self> {
        let options = RankCount::enumerate(num_cards);
        let mut configs = Vec::new();
        let mut chosen = [RankCount::EMPTY; N_SUITS];
        Self::build(
            &options,
            0,
            options.len(),
            *num_cards,
            &mut chosen,
            &mut configs,
        );
        configs.sort_unstable();
        configs.reverse();
        configs
    }

    fn build(
        options: &[RankCount<ROUNDS>],
        suit: usize,
        max_idx: usize,
        remaining: [CardCount; ROUNDS],
        chosen: &mut [RankCount<ROUNDS>; N_SUITS],
        out: &mut Vec<Self>,
    ) {
        if suit == N_SUITS {
            if remaining.iter().all(|&c| c == 0) {
                out.push(Self(*chosen));
            }
            return;
        }

        for idx in 0..max_idx {
            let opt = options[idx];
            if opt.0.iter().zip(&remaining).any(|(&o, &r)| o > r) {
                continue;
            }
            let mut next = remaining;
            for r in 0..ROUNDS {
                next[r] -= opt.0[r];
            }
            chosen[suit] = opt;
            Self::build(options, suit + 1, idx + 1, next, chosen, out);
        }
    }

    pub const fn to_multiset(&self) -> SuitMultiset {
        use MixedRadix as R;
        use SuitMultiset::{MMMM, MMMS, MMNN, MMSS, SMMM, SMMS, SSMM, SSSS};
        use util::multi;
        const T: bool = true;
        const F: bool = false;
        let (s, h, d, c) = (self.0[0], self.0[1], self.0[2], self.0[3]);

        match (s.const_eq(h), h.const_eq(d), d.const_eq(c)) {
            (T, T, T) => MMMM(R::new([multi(4, s.size())])),
            (T, F, T) => MMNN(R::new([multi(2, s.size()), multi(2, d.size())])),
            (T, T, F) => MMMS(R::new([multi(3, s.size()), c.size()])),
            (F, T, T) => SMMM(R::new([s.size(), multi(3, h.size())])),
            (T, F, F) => MMSS(R::new([multi(2, s.size()), d.size(), c.size()])),
            (F, T, F) => SMMS(R::new([s.size(), multi(2, h.size()), c.size()])),
            (F, F, T) => SSMM(R::new([s.size(), h.size(), multi(2, d.size())])),
            (F, F, F) => SSSS(R::new([s.size(), h.size(), d.size(), c.size()])),
        }
    }

    #[inline]
    pub(crate) fn const_binary_search(&self, configs: &[Self]) -> usize {
        let mut lo = 0;
        let mut hi = configs.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if self.const_eq(&configs[mid]) {
                return mid;
            }
            if self.const_lt(&configs[mid]) {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        unreachable!() // LCOV_EXCL_LINE
    }

    #[inline]
    const fn const_eq(&self, other: &Self) -> bool {
        let mut i = 0;
        while i < N_SUITS {
            if !self.0[i].const_eq(other.0[i]) {
                return false;
            }
            i += 1;
        }
        true
    }

    #[inline]
    const fn const_lt(&self, other: &Self) -> bool {
        let mut i = 0;
        while i < N_SUITS {
            if self.0[i].const_lt(other.0[i]) {
                return true;
            }
            if other.0[i].const_lt(self.0[i]) {
                return false;
            }
            i += 1;
        }
        false
    }
}

impl<const ROUNDS: usize> fmt::Debug for SuitConfig<ROUNDS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        for (i, rc) in self.0.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{rc:?}")?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<const ROUNDS: usize> quickcheck::Arbitrary for SuitConfig<ROUNDS> {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self([
                RankCount::arbitrary(g),
                RankCount::arbitrary(g),
                RankCount::arbitrary(g),
                RankCount::arbitrary(g),
            ])
        }
    }

    fn brute_force_enumerate<const ROUNDS: usize>(
        num_cards: &[CardCount; ROUNDS],
    ) -> Vec<SuitConfig<ROUNDS>> {
        fn sum_to_num_card<const N: usize>(
            num_cards: &[CardCount; N],
            suits: &[RankCount<N>],
        ) -> bool {
            let mut res = [0; N];
            for rc in suits {
                for (r, &c) in rc.0.iter().enumerate() {
                    res[r] += c;
                }
            }
            &res == num_cards
        }

        let options = RankCount::enumerate(num_cards);
        let mut configs = (0..N_SUITS)
            .fold(vec![Vec::new()], |acc, _| {
                acc.iter()
                    .flat_map(|prefix| {
                        options.iter().map(move |opt| {
                            let mut next = prefix.clone();
                            next.push(*opt);
                            next
                        })
                    })
                    .collect()
            })
            .into_iter()
            .filter(|suits| sum_to_num_card(num_cards, suits))
            .map(|mut v| {
                v.sort_unstable();
                v.reverse();
                SuitConfig(v.try_into().unwrap())
            })
            .collect::<Vec<_>>();
        configs.sort_unstable();
        configs.dedup();
        configs.reverse();
        configs
    }

    #[test]
    fn test_enumerate_matches_brute_force() {
        assert_eq!(SuitConfig::enumerate(&[5]), brute_force_enumerate(&[5]));
        assert_eq!(
            SuitConfig::enumerate(&[3, 2]),
            brute_force_enumerate(&[3, 2])
        );
        assert_eq!(
            SuitConfig::enumerate(&[3, 1, 1, 1]),
            brute_force_enumerate(&[3, 1, 1, 1])
        );
    }

    #[test]
    fn test_const_binary_search() {
        let configs = SuitConfig::enumerate(&[3, 2]);
        for (i, config) in configs.iter().enumerate() {
            assert_eq!(config.const_binary_search(&configs), i);
        }
    }

    const R: usize = 4;
    #[quickcheck]
    fn test_const_lt(a: SuitConfig<R>, b: SuitConfig<R>) {
        assert_eq!(a < b, a.const_lt(&b));
    }

    #[test]
    fn test_const_lt_eq() {
        let a = SuitConfig::<2>::EMPTY;
        assert!(!a.const_lt(&a));
    }

    #[test]
    fn test_debug() {
        let obj = SuitConfig([
            RankCount([3, 1]),
            RankCount([1, 2]),
            RankCount([0, 0]),
            RankCount([0, 0]),
        ]);
        assert_eq!("{(3,1), (1,2), (0,0), (0,0)}", format!("{obj:?}"));
    }
}
