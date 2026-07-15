use crate::Game;

/// How many hole cards a hand must use to form its rating.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HoleCardRule {
    /// Use any number of hole cards (Hold'em).
    #[default]
    UseAny,
    /// Use exactly two hole cards (Omaha).
    UseExactlyTwo,
}

impl HoleCardRule {
    /// Infers the rule from the number of hole cards a player holds
    #[must_use]
    pub const fn from_cards<T>(cards: &[T]) -> Self {
        Self::from_hole_count(cards.len())
    }

    /// Infers the rule from the number of hole cards a player holds
    #[must_use]
    pub const fn from_hole_count(count: usize) -> Self {
        if count == Game::HOLDEM_CARDS as usize {
            Self::UseAny
        } else {
            Self::UseExactlyTwo
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    #[test]
    fn test_hole_card_rule() {
        use Game::*;
        use HoleCardRule::*;

        assert_eq!(Holdem.to_hole_card_rule(), UseAny);
        assert_eq!(ShortDeck.to_hole_card_rule(), UseAny);
        assert_eq!(Omaha.to_hole_card_rule(), UseExactlyTwo);
        assert_eq!(Omaha5.to_hole_card_rule(), UseExactlyTwo);
    }

    #[test]
    fn test_from_hole_count() {
        use HoleCardRule::*;

        assert_eq!(HoleCardRule::from_hole_count(2), UseAny);
        assert_eq!(HoleCardRule::from_hole_count(4), UseExactlyTwo);
        assert_eq!(HoleCardRule::from_hole_count(5), UseExactlyTwo);
    }
}
