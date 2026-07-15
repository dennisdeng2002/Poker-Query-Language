use crate::{
    Card, FlushingSuit, Game, IsomorphicCard, IsomorphicHandN,
    isomorphism::{
        flush_texture_omaha::FlushTextureOmaha, flush_texture_omaha5::FlushTextureOmaha5,
    },
};

const N_HOLDEM: usize = Game::HOLDEM_CARDS as usize;
const N_OMAHA: usize = Game::OMAHA_CARDS as usize;
const N_OMAHA5: usize = Game::OMAHA5_CARDS as usize;

pub const fn isomorphic_preflop_holdem(cards: &[Card]) -> IsomorphicHandN<N_HOLDEM> {
    let Card { rank: r0, suit: s0 } = cards[0];
    let Card { rank: r1, suit: s1 } = cards[1];

    let suit = if s0.const_eq(s1) {
        FlushingSuit::X
    } else {
        FlushingSuit::N
    };

    let (r0, r1) = if r0.const_lt(r1) { (r0, r1) } else { (r1, r0) };

    IsomorphicHandN([IsomorphicCard::new(r0, suit), IsomorphicCard::new(r1, suit)])
}

pub const fn isomorphic_preflop_omaha(cards: &[Card]) -> IsomorphicHandN<N_OMAHA> {
    IsomorphicHandN(FlushTextureOmaha::preflop_iso([
        cards[0], cards[1], cards[2], cards[3],
    ]))
}

pub const fn isomorphic_preflop_omaha5(cards: &[Card]) -> IsomorphicHandN<N_OMAHA5> {
    IsomorphicHandN(FlushTextureOmaha5::preflop_iso(
        cards[0], cards[1], cards[2], cards[3], cards[4],
    ))
}
