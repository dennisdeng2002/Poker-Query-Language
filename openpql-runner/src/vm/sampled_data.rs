use super::*;

/// Sampled Data
/// all rng related values
#[derive(Clone, Debug, Default, derive_more::From)]
pub struct VmSampledData {
    pub(crate) cards: Vec<PQLCard>,
    card_gen: CardGen,
    pub(crate) n_players: PQLCardCount,
    pub(crate) n_holecards: PQLCardCount,
}

fn gen_card(
    rng: &mut impl rand::Rng,
    card_gen: &mut CardGen,
    hand: &mut [PQLCard],
    idx: usize,
    predicate: &FnCheckRange,
) -> Option<()> {
    let mut failed = PQLCardSet::default();

    loop {
        hand[idx] = card_gen.deal(rng)?;

        if predicate(&hand[..=idx]) {
            card_gen.unset(failed);
            return Some(());
        }

        failed.set(hand[idx]);
    }
}

fn gen_cards(
    rng: &mut impl rand::Rng,
    card_gen: &mut CardGen,
    hand: &mut [PQLCard],
    predicate: &FnCheckRange,
) -> Option<()> {
    for i in 0..hand.len() {
        gen_card(rng, card_gen, hand, i, predicate)?;
    }

    Some(())
}

/// Deals a fully-specified hand directly, without rejection sampling: each
/// card is either free (and gets taken) or already dealt elsewhere in this
/// trial (a conflict, in which case the trial fails, matching what the
/// rejection-sampling path would eventually conclude on its own).
fn deal_literal(card_gen: &mut CardGen, hand: &mut [PQLCard], literal: &[PQLCard]) -> Option<()> {
    for (slot, &card) in hand.iter_mut().zip(literal) {
        *slot = card_gen.take(card)?;
    }

    Some(())
}

fn deal_hand(
    rng: &mut impl rand::Rng,
    card_gen: &mut CardGen,
    hand: &mut [PQLCard],
    literal: Option<&[PQLCard]>,
    predicate: &FnCheckRange,
) -> Option<()> {
    match literal {
        Some(cs) => deal_literal(card_gen, hand, cs),
        None => gen_cards(rng, card_gen, hand, predicate),
    }
}

impl VmSampledData {
    pub fn new(game: PQLGame, n_players: PQLPlayerCount, dead_cards: PQLCardSet) -> Self {
        let card_gen = if game.is_shortdeck() {
            CardGen::new::<true>(dead_cards)
        } else {
            CardGen::new::<false>(dead_cards)
        };
        let n_holecards = game.player_cards_len();
        let n_total = PQLFnContext::n_total_cards(n_players, n_holecards);

        Self {
            cards: vec![PQLCard::default(); n_total],
            card_gen,
            n_players,
            n_holecards,
        }
    }

    pub fn sample(
        &mut self,
        rng: &mut impl rand::Rng,
        player_ranges: &[PQLRange],
        board_range: &PQLBoardRange,
    ) -> Option<()> {
        self.card_gen.reset();

        self.sample_player_cards(rng, player_ranges)?;
        self.sample_board_cards(rng, board_range)?;

        Some(())
    }

    fn sample_player_cards(
        &mut self,
        rng: &mut impl rand::Rng,
        player_ranges: &[PQLRange],
    ) -> Option<()> {
        let n = self.n_holecards as usize;

        for (i, range) in player_ranges.iter().enumerate() {
            let hand = &mut self.cards[i * n..(i + 1) * n];

            deal_hand(rng, &mut self.card_gen, hand, range.literal(), &range.0)?;
        }

        Some(())
    }

    fn sample_board_cards(
        &mut self,
        rng: &mut impl rand::Rng,
        board_range: &PQLBoardRange,
    ) -> Option<()> {
        let i = PQLFnContext::idx_board_start(self.n_players, self.n_holecards);
        let board = &mut self.cards[i..i + PQLBoard::N_RIVER as usize];

        deal_hand(
            rng,
            &mut self.card_gen,
            board,
            board_range.literal(),
            &board_range.0,
        )?;

        Some(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::*;

    fn mk_sample_fn(
        game: PQLGame,
        player: &[&str],
        board: &str,
    ) -> impl FnMut() -> Option<Vec<PQLCard>> {
        let (ps, b) = mk_ranges(game, player, board);

        move || {
            let mut sampler = VmSampledData::new(
                game,
                PQLCardCount::try_from(player.len()).unwrap(),
                PQLCardSet::default(),
            );

            if sampler.sample(&mut rand::rng(), &ps, &b).is_some() {
                Some(sampler.cards)
            } else {
                None
            }
        }
    }

    fn assert_sample(game: PQLGame, player: &[&str], board: &str) {
        let (ps, b) = mk_ranges(game, player, board);
        let res = mk_sample_fn(game, player, board)().unwrap();

        let n = game.player_cards_len() as usize;
        for (i, range) in ps.iter().enumerate() {
            assert!(range.is_satisfied(&res[i * n..(i + 1) * n]));
        }

        let i = ps.len() * n;
        assert!(b.is_satisfied(&res[i..i + 5]));
    }

    #[test]
    fn test_holdem() {
        assert_sample(PQLGame::Holdem, &["AA", "KK"], "AK*JT");
    }

    #[test]
    fn test_omaha() {
        assert_sample(PQLGame::Omaha, &["AAAA", "KKKK"], "***JT");
    }

    #[test]
    fn test_shortdeck() {
        assert_sample(PQLGame::ShortDeck, &["*", "*"], "***JT");
    }

    fn assert_none(game: PQLGame, player: &[&str], board: &str) {
        assert!(mk_sample_fn(game, player, board)().is_none());
    }

    #[test]
    fn test_none() {
        let game = PQLGame::default();

        assert_none(game, &["AA"], "AAA");
        assert_none(game, &["AA", "AA", "AA"], "*");
    }

    #[test]
    fn test_literal_hands() {
        assert_sample(PQLGame::Holdem, &["AsKs", "7c6c"], "*");
        assert_sample(PQLGame::Omaha6, &["AsKsQsJsTs9s", "7c6c5c4c3c2c"], "*");
    }

    #[test]
    fn test_literal_hand_collision_fails() {
        // Both players want the same fixed cards: unsatisfiable every time,
        // same as the generic (non-literal) rejection-sampling path would
        // eventually conclude.
        assert_none(PQLGame::Holdem, &["AsKs", "AsKs"], "*");
    }

    #[test]
    fn test_literal_hand_vs_dead_card_fails() {
        let (ps, b) = mk_ranges(PQLGame::Holdem, &["AsKs"], "*");
        let mut sampler = VmSampledData::new(PQLGame::Holdem, 1, PQLCardSet::from(card!("As")));

        assert!(sampler.sample(&mut rand::rng(), &ps, &b).is_none());
    }

    #[test]
    fn test_literal_board_preserves_street_order() {
        let res = mk_sample_fn(PQLGame::Holdem, &["*", "*"], "AhKdQc2s3h")().unwrap();
        let board = &res[res.len() - 5..];

        assert_eq!(
            board,
            [
                card!("Ah"),
                card!("Kd"),
                card!("Qc"),
                card!("2s"),
                card!("3h"),
            ],
        );
    }
}
