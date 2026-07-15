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

/// Picks the [`SuitMap`] that relabels two flush suits into canonical order.
///
/// Both `map2` orderings are tried and the one yielding the lexicographically
/// smaller form wins, making the result invariant to which suit is labeled first.
#[inline]
pub const fn double_flush_map<const N: usize>(s0: Suit, s1: Suit, cards: [Card; N]) -> SuitMap {
    let (map0, map1) = (SuitMap::map2(s0, s1), SuitMap::map2(s1, s0));

    if iso_lt(relabel_sorted(map0, cards), relabel_sorted(map1, cards)) {
        map0
    } else {
        map1
    }
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
}
