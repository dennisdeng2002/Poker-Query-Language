//! Suit-isomorphic canonical form of a flop.

use crate::{Board, Card, Flop, IsomorphicCard, SuitMap, isomorphism::FlushTextureFlop};

/// Canonical suit-isomorphic representative of a flop.
#[derive(Clone, Copy, derive_more::Debug, PartialEq, Eq, Hash)]
#[debug("IsomorphicFlopGto({_0:?})")]
pub struct IsomorphicFlopGto(pub(crate) [IsomorphicCard; Board::N_FLOP as usize]);

impl IsomorphicFlopGto {
    /// Canonical representative of `flop` and the [`SuitMap`] that produced it.
    #[inline]
    pub(crate) const fn from_flop(flop: Flop) -> (Self, SuitMap) {
        let [f0, f1, f2] = flop.0;
        let map = FlushTextureFlop::classify(f0, f1, f2).to_suit_map();

        (Self::relabel(f0, f1, f2, map), map)
    }

    /// Relabels three flop cards through `map`, then rank-sorts them.
    #[inline]
    pub(crate) const fn relabel(f0: Card, f1: Card, f2: Card, map: SuitMap) -> Self {
        let mut res = [map.iso_card(f0), map.iso_card(f1), map.iso_card(f2)];

        IsomorphicCard::const_sort(&mut res);
        Self(res)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::*;

    fn iso_flop(s: &str) -> IsomorphicFlopGto {
        let cs = isocards!(s);

        IsomorphicFlopGto([cs[0], cs[1], cs[2]])
    }

    fn assert_iso_flop(s: &str) {
        let (lhs, rhs) = s.split_once("->").unwrap();
        let got = IsomorphicFlopGto::from_flop(flop!(lhs)).0;

        assert_eq!(
            got,
            iso_flop(rhs),
            "{:?}->{rhs}; but got {got:?}",
            flop!(lhs)
        );
    }

    fn gen_iso_flop() -> FxHashSet<IsomorphicFlopGto> {
        Flop::iter_all::<true>()
            .map(|flop| IsomorphicFlopGto::from_flop(flop).0)
            .collect()
    }

    #[test]
    fn test_iso_flop() {
        assert_iso_flop("6s6hAs -> 6x6yAx");
        assert_iso_flop("6s6hAh -> 6x6yAx");

        assert_eq!(gen_iso_flop().len(), 573);
    }

    #[test]
    fn test_trips_rainbow() {
        assert_iso_flop("AsAhAd -> AxAyAz");
        assert_iso_flop("AsAhAc -> AxAyAz");
        assert_iso_flop("AsAdAc -> AxAyAz");
        assert_iso_flop("AhAdAc -> AxAyAz");
    }

    #[test]
    fn test_paired_twotone() {
        assert_iso_flop("AsAhKh -> KxAxAy");
        assert_iso_flop("AhAsKs -> KxAxAy");
        assert_iso_flop("ThTdKd -> TxTyKx");
        assert_iso_flop("TcTsKc -> TxTyKx");

        assert_iso_flop("6s6hAs -> 6x6yAx");
        assert_iso_flop("6s6hAh -> 6x6yAx");
    }

    #[test]
    fn test_paired_rainbow() {
        assert_iso_flop("AsAhKd -> KxAyAz");
        assert_iso_flop("ThTcKd -> TxTyKz");
    }

    #[test]
    fn test_monotone() {
        assert_iso_flop("AsKsQs -> QxKxAx");
        assert_iso_flop("AhKhQh -> QxKxAx");
    }
}
