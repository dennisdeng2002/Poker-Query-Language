use super::tables::{
    ALL_HANDS_HOLDEM, ALL_HANDS_HOLDEM_ISO, ALL_HANDS_OMAHA, ALL_HANDS_OMAHA_ISO, ALL_HANDS_OMAHA5,
    ALL_HANDS_OMAHA5_ISO, ALL_HANDS_OMAHA6, ALL_HANDS_OMAHA6_ISO, ALL_HANDS_SHORTDECK,
    ALL_HANDS_SHORTDECK_ISO,
};
use crate::{Card, Game, IsomorphicCard};

impl Game {
    /// Returns every legal starting hand for this variant.
    #[must_use]
    pub fn starting_hands(self) -> &'static [Vec<Card>] {
        match self {
            Self::Holdem => &ALL_HANDS_HOLDEM,
            Self::Omaha => &ALL_HANDS_OMAHA,
            Self::Omaha5 => &ALL_HANDS_OMAHA5,
            Self::Omaha6 => &ALL_HANDS_OMAHA6,
            Self::ShortDeck => &ALL_HANDS_SHORTDECK,
        }
    }

    #[must_use]
    pub fn starting_iso_hands(self) -> &'static [Vec<IsomorphicCard>] {
        match self {
            Self::Holdem => &ALL_HANDS_HOLDEM_ISO,
            Self::Omaha => &ALL_HANDS_OMAHA_ISO,
            Self::Omaha5 => &ALL_HANDS_OMAHA5_ISO,
            Self::Omaha6 => &ALL_HANDS_OMAHA6_ISO,
            Self::ShortDeck => &ALL_HANDS_SHORTDECK_ISO,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Game;

    #[test]
    #[ignore = "slow"]
    fn test_starting_hands() {
        assert_eq!(Game::ShortDeck.starting_hands().len(), 630);
        assert_eq!(Game::Holdem.starting_hands().len(), 1326);
        assert_eq!(Game::Omaha.starting_hands().len(), 270_725);
        assert_eq!(Game::Omaha5.starting_hands().len(), 2_598_960);
        assert_eq!(Game::Omaha6.starting_hands().len(), 20_358_520);
    }

    #[test]
    #[ignore = "slow"]
    fn test_starting_iso_hands() {
        for game in [Game::ShortDeck, Game::Holdem, Game::Omaha] {
            let n = match game {
                Game::Omaha => Game::OMAHA_CARDS,
                _ => Game::HOLDEM_CARDS,
            };
            assert!(
                game.starting_iso_hands()
                    .iter()
                    .all(|h| h.len() == n as usize)
            );
        }

        assert_eq!(Game::ShortDeck.starting_iso_hands().len(), 81);
        assert_eq!(Game::Holdem.starting_iso_hands().len(), 169);
        assert_eq!(Game::Omaha.starting_iso_hands().len(), 16_432);
        assert_eq!(Game::Omaha5.starting_iso_hands().len(), 134_459);
        assert_eq!(Game::Omaha6.starting_iso_hands().len(), 962_988);
    }
}
