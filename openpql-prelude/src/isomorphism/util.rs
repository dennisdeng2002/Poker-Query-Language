use crate::{Card, CardCount, FlushingSuit, IsomorphicCard, IsomorphicHand, Suit, SuitMap};

/// Isomorphic form of a `board`/`player` pair carrying no flush-relevant suit.
#[inline]
pub const fn no_flush_pair(board: &[Card], player: &[Card]) -> (IsomorphicHand, IsomorphicHand) {
    (
        IsomorphicHand::no_flush(board),
        IsomorphicHand::no_flush(player),
    )
}

/// Isomorphic form of a `board`/`player` pair relabeled through `map`.
#[inline]
pub const fn mapped_pair(
    map: SuitMap,
    board: &[Card],
    player: &[Card],
) -> (IsomorphicHand, IsomorphicHand) {
    (map.iso_cards(board), map.iso_cards(player))
}

/// Relabels `cards` through `map`, then rank-sorts the resulting [`IsomorphicCard`]s.
#[inline]
pub const fn relabel_sorted<const N: usize>(map: SuitMap, cards: [Card; N]) -> [IsomorphicCard; N] {
    let mut arr = [IsomorphicCard::DEFAULT; N];
    let mut i = 0;

    while i < N {
        arr[i] = map.iso_card(cards[i]);
        i += 1;
    }

    IsomorphicCard::const_sort(&mut arr);
    arr
}

/// Lexicographic less-than over two equal-length isomorphic forms.
#[inline]
const fn iso_lt<const N: usize>(lhs: [IsomorphicCard; N], rhs: [IsomorphicCard; N]) -> bool {
    let mut i = 0;

    while i < N {
        if !lhs[i].const_eq(rhs[i]) {
            return lhs[i].const_lt(rhs[i]);
        }
        i += 1;
    }

    true
}

/// Picks whichever of `candidates` relabels `cards` into the lexicographically
/// smallest form, making the result invariant to the input order of the
/// flush-relevant suits.
#[inline]
const fn best_map<const K: usize, const N: usize>(
    candidates: [SuitMap; K],
    cards: [Card; N],
) -> SuitMap {
    let mut best = relabel_sorted(candidates[0], cards);
    let mut best_map = candidates[0];
    let mut i = 1;

    while i < K {
        let cur = relabel_sorted(candidates[i], cards);
        if iso_lt(cur, best) {
            best = cur;
            best_map = candidates[i];
        }
        i += 1;
    }

    best_map
}

/// Picks the [`SuitMap`] that relabels two flush suits into canonical order.
///
/// Both `map2` orderings are tried and the one yielding the lexicographically
/// smaller form wins, making the result invariant to which suit is labeled first.
#[inline]
pub const fn double_flush_map<const N: usize>(s0: Suit, s1: Suit, cards: [Card; N]) -> SuitMap {
    best_map([SuitMap::map2(s0, s1), SuitMap::map2(s1, s0)], cards)
}

/// Picks the [`SuitMap`] that relabels three flush suits into canonical order.
///
/// All `3! = 6` orderings of `map3` are tried and the one yielding the
/// lexicographically smallest form wins, making the result invariant to
/// which suit is labeled first, second, or third.
#[inline]
pub const fn triple_flush_map<const N: usize>(
    s0: Suit,
    s1: Suit,
    s2: Suit,
    cards: [Card; N],
) -> SuitMap {
    best_map(
        [
            SuitMap::map3(s0, s1, s2),
            SuitMap::map3(s0, s2, s1),
            SuitMap::map3(s1, s0, s2),
            SuitMap::map3(s1, s2, s0),
            SuitMap::map3(s2, s0, s1),
            SuitMap::map3(s2, s1, s0),
        ],
        cards,
    )
}

/// Number of `cards` whose suit is `suit`.
#[inline]
pub const fn count_suit(cards: &[Card], suit: Suit) -> CardCount {
    let mut n: CardCount = 0;
    let mut i = 0;

    while i < cards.len() {
        if cards[i].suit.const_eq(suit) {
            n += 1;
        }
        i += 1;
    }

    n
}

#[inline]
pub const fn n_flush_suits(cards: &[IsomorphicCard]) -> CardCount {
    let (mut x, mut y, mut z) = (false, false, false);
    let mut idx = 0;

    while idx < cards.len() {
        match cards[idx].suit {
            FlushingSuit::X => x = true,
            FlushingSuit::Y => y = true,
            FlushingSuit::Z => z = true,
            FlushingSuit::N => {}
        }
        idx += 1;
    }

    x as CardCount + y as CardCount + z as CardCount
}

#[inline]
pub const fn place_card(c: IsomorphicCard, next: CardCount) -> (Card, CardCount) {
    const fn take(suit: FlushingSuit, next: CardCount) -> (Suit, CardCount) {
        match suit {
            FlushingSuit::X => (Suit::S, next),
            FlushingSuit::Y => (Suit::H, next),
            FlushingSuit::Z => (Suit::D, next),
            FlushingSuit::N => (Suit::ARR_ALL[(next % Suit::N_SUITS) as usize], next + 1),
        }
    }

    let (s, next) = take(c.suit, next);

    (Card::new(c.rank, s), next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_z_suit() {
        let z = IsomorphicCard::new(Rank::RA, FlushingSuit::Z);

        assert_eq!(n_flush_suits(&[z]), 1);
        assert_eq!(place_card(z, 0), (Card::new(Rank::RA, Suit::D), 0));
    }

    #[test]
    fn test_triple_flush_map_is_suit_order_invariant() {
        let cards = cards!("2s3s4h5h6d7d");
        let arr: [Card; 6] = cards.try_into().unwrap();

        let base = relabel_sorted(triple_flush_map(Suit::S, Suit::H, Suit::D, arr), arr);

        for (s0, s1, s2) in [
            (Suit::S, Suit::D, Suit::H),
            (Suit::H, Suit::S, Suit::D),
            (Suit::H, Suit::D, Suit::S),
            (Suit::D, Suit::S, Suit::H),
            (Suit::D, Suit::H, Suit::S),
        ] {
            let map = triple_flush_map(s0, s1, s2, arr);
            assert_eq!(relabel_sorted(map, arr), base);
        }
    }
}
