use std::sync::LazyLock;

#[cfg(feature = "rayon")]
use rayon::prelude::*;
use rustc_hash::FxHashSet;

use crate::{
    Card, Game, HandN, IsomorphicCard, IsomorphicHandN,
    isomorphism::{isomorphic_preflop_holdem, isomorphic_preflop_omaha, isomorphic_preflop_omaha5},
};

type StaticHands = LazyLock<Vec<Vec<Card>>>;
type StaticIsoHands = LazyLock<Vec<Vec<IsomorphicCard>>>;

fn collect_hands<const SD: bool, const N: usize>() -> Vec<Vec<Card>> {
    #[cfg(feature = "rayon")]
    {
        HandN::<N>::iter_all::<SD>()
            .into_par_iter()
            .map(|h| h.to_vec())
            .collect()
    }
    #[cfg(not(feature = "rayon"))]
    {
        HandN::<N>::iter_all::<SD>().map(|h| h.to_vec()).collect()
    }
}

pub static ALL_HANDS_SHORTDECK: StaticHands =
    LazyLock::new(collect_hands::<true, { Game::SHORTDECK_CARDS as usize }>);
pub static ALL_HANDS_HOLDEM: StaticHands =
    LazyLock::new(collect_hands::<false, { Game::HOLDEM_CARDS as usize }>);
pub static ALL_HANDS_OMAHA: StaticHands =
    LazyLock::new(collect_hands::<false, { Game::OMAHA_CARDS as usize }>);
pub static ALL_HANDS_OMAHA5: StaticHands =
    LazyLock::new(collect_hands::<false, { Game::OMAHA5_CARDS as usize }>);

fn iso_hands<const N: usize>(
    hands: &[Vec<Card>],
    to_iso: impl Fn(&[Card]) -> IsomorphicHandN<N> + Sync,
) -> Vec<Vec<IsomorphicCard>> {
    #[cfg(feature = "rayon")]
    let isos: Vec<IsomorphicHandN<N>> = hands.par_iter().map(|h| to_iso(h)).collect();
    #[cfg(not(feature = "rayon"))]
    let isos: Vec<IsomorphicHandN<N>> = hands.iter().map(|h| to_iso(h)).collect();

    let mut seen = FxHashSet::default();
    isos.into_iter()
        .filter(|iso| seen.insert(*iso))
        .map(|iso| iso.0.to_vec())
        .collect()
}

pub static ALL_HANDS_SHORTDECK_ISO: StaticIsoHands = LazyLock::new(|| {
    iso_hands::<{ Game::SHORTDECK_CARDS as usize }>(&ALL_HANDS_SHORTDECK, isomorphic_preflop_holdem)
});

pub static ALL_HANDS_HOLDEM_ISO: StaticIsoHands = LazyLock::new(|| {
    iso_hands::<{ Game::HOLDEM_CARDS as usize }>(&ALL_HANDS_HOLDEM, isomorphic_preflop_holdem)
});

pub static ALL_HANDS_OMAHA_ISO: StaticIsoHands = LazyLock::new(|| {
    iso_hands::<{ Game::OMAHA_CARDS as usize }>(&ALL_HANDS_OMAHA, isomorphic_preflop_omaha)
});

pub static ALL_HANDS_OMAHA5_ISO: StaticIsoHands = LazyLock::new(|| {
    iso_hands::<{ Game::OMAHA5_CARDS as usize }>(&ALL_HANDS_OMAHA5, isomorphic_preflop_omaha5)
});
