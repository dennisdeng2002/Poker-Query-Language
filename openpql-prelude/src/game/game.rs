use std::str::FromStr;

use super::HoleCardRule;
use crate::{CardCount, ParseError};

/// Poker variant.
#[cfg_attr(feature = "speedy", derive(speedy::Readable, speedy::Writable))] // LCOV_EXCL_LINE
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Game {
    /// Texas Hold'em.
    #[default]
    Holdem,
    /// Pot-Limit Omaha.
    Omaha,
    /// 5-Card Omaha (Big O / PLO5).
    Omaha5,
    /// Short Deck (6+) Hold'em.
    ShortDeck,
}

impl Game {
    /// Number of hole cards in Texas Hold'em.
    pub const HOLDEM_CARDS: CardCount = 2;
    /// Number of hole cards in Short Deck (6+) Hold'em.
    pub const SHORTDECK_CARDS: CardCount = 2;
    /// Number of hole cards in Pot-Limit Omaha.
    pub const OMAHA_CARDS: CardCount = 4;
    /// Number of hole cards in 5-Card Omaha.
    pub const OMAHA5_CARDS: CardCount = 5;

    /// Returns the number of hole cards dealt to each player.
    #[must_use]
    pub const fn player_cards_len(self) -> CardCount {
        match self {
            Self::Holdem => Self::HOLDEM_CARDS,
            Self::ShortDeck => Self::SHORTDECK_CARDS,
            Self::Omaha => Self::OMAHA_CARDS,
            Self::Omaha5 => Self::OMAHA5_CARDS,
        }
    }

    /// Returns `true` for Short Deck.
    #[must_use]
    pub const fn is_shortdeck(self) -> bool {
        matches!(self, Self::ShortDeck)
    }

    pub const fn to_hole_card_rule(self) -> HoleCardRule {
        match self {
            Self::Holdem | Self::ShortDeck => HoleCardRule::UseAny,
            Self::Omaha | Self::Omaha5 => HoleCardRule::UseExactlyTwo,
        }
    }
}

impl FromStr for Game {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().trim() {
            "holdem" => Ok(Self::Holdem),
            "omaha" => Ok(Self::Omaha),
            "omaha5" => Ok(Self::Omaha5),
            "shortdeck" => Ok(Self::ShortDeck),
            _ => Err(ParseError::InvalidGame(s.into())),
        }
    }
}

#[cfg(any(test, feature = "quickcheck"))]
impl quickcheck::Arbitrary for Game {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        #[allow(unused)]
        const fn completeness_check(e: Game) {
            match e {
                Game::Holdem | Game::Omaha | Game::Omaha5 | Game::ShortDeck => (),
            }
        }

        *g.choose(&[Self::Holdem, Self::Omaha, Self::Omaha5, Self::ShortDeck])
            .unwrap()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_player_cards_len() {
        assert_eq!(2, Game::Holdem.player_cards_len());
        assert_eq!(4, Game::Omaha.player_cards_len());
        assert_eq!(5, Game::Omaha5.player_cards_len());
        assert_eq!(2, Game::ShortDeck.player_cards_len());
    }

    #[test]
    fn test_is_shortdeck() {
        assert!(!Game::Holdem.is_shortdeck());
        assert!(!Game::Omaha.is_shortdeck());
        assert!(!Game::Omaha5.is_shortdeck());
        assert!(Game::ShortDeck.is_shortdeck());
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Ok(Game::Holdem), " HoldEM ".parse());

        assert_eq!(Ok(Game::Omaha), "omaha".parse());
        assert_eq!(Ok(Game::Omaha5), "omaha5".parse());
        assert_eq!(Ok(Game::ShortDeck), "shortdeck".parse());

        assert_eq!(
            Err(ParseError::InvalidGame("unknown".into())),
            "unknown".parse::<Game>()
        );
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests_serde {
    use super::*;
    use crate::*;

    #[quickcheck]
    fn test_game_ser_de() {
        assert_tokens(
            &Game::ShortDeck,
            &[Token::UnitVariant {
                name: "Game",
                variant: "ShortDeck",
            }],
        );

        assert_tokens(
            &Game::Holdem,
            &[Token::UnitVariant {
                name: "Game",
                variant: "Holdem",
            }],
        );

        assert_tokens(
            &Game::Omaha,
            &[Token::UnitVariant {
                name: "Game",
                variant: "Omaha",
            }],
        );

        assert_tokens(
            &Game::Omaha5,
            &[Token::UnitVariant {
                name: "Game",
                variant: "Omaha5",
            }],
        );
    }
}
