use openpql_prelude::{HandN, HoleCardRule};

use crate::{PQLBoard, PQLCard, PQLCardCount, PQLCardSet, PQLGame};

/// Counts the unseen cards that would give the hand an unbeatable high hand.
// TODO: optimize
#[expect(clippy::cast_possible_truncation, reason = "num of cards < u8::MAX")]
pub fn nut_hi_outs(game: PQLGame, hand: &[PQLCard], board: PQLBoard) -> PQLCardCount {
    let p = PQLCardSet::from(hand);
    let b = PQLCardSet::from(board);
    let known = p | b;

    let all = if game.is_shortdeck() {
        PQLCardSet::all::<true>()
    } else {
        PQLCardSet::all::<false>()
    };

    (all & !known)
        .iter()
        .filter(|&c| {
            let c64 = PQLCardSet::from(c);
            is_nut_hi(game, p, b | c64, known | c64)
        })
        .count() as PQLCardCount
}

fn is_nut_hi(game: PQLGame, p: PQLCardSet, b: PQLCardSet, known: PQLCardSet) -> bool {
    let player_rating = game.eval_rating(p, b);

    match game.to_hole_card_rule() {
        // A hand's rating depends only on its best 2 hole cards, so the best
        // rating any opponent could possibly hold is exactly the rating of
        // the whole unseen-card pool evaluated as one hand — no need to
        // enumerate individual opponent hands at all.
        HoleCardRule::UseExactlyTwo => {
            let unseen = PQLCardSet::all::<false>() & !known;
            game.eval_rating(unseen, b) <= player_rating
        }
        HoleCardRule::UseAny => {
            let check = |opp: PQLCardSet| -> bool {
                if !(opp & known).is_empty() {
                    return true;
                }
                game.eval_rating(opp, b) <= player_rating
            };

            match game {
                PQLGame::Holdem => HandN::<2>::iter_all::<false>().all(|h| check(h.into())),
                PQLGame::ShortDeck => HandN::<2>::iter_all::<true>().all(|h| check(h.into())),
                PQLGame::Omaha | PQLGame::Omaha5 | PQLGame::Omaha6 => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use openpql_prelude::{board, cards};

    use super::*;

    #[test]
    fn test_nut_hi_outs_holdem() {
        // KsJs + QdTd4c: 3 As (not Ad) + 3 9s (not 9d) = 6
        // Ad gives broadway but board becomes 4-flush → opp with any diamond wins
        // 9d gives straight but board becomes 4-flush → opp with any diamond wins
        assert_eq!(
            nut_hi_outs(PQLGame::Holdem, &cards!("Ks Js"), board!("Qd Td 4c"),),
            6,
        );
    }

    /// `is_nut_hi`'s `UseExactlyTwo` branch (whole-unseen-pool trick) must
    /// agree with brute-force enumeration of every possible opponent hand,
    /// since an Omaha-family hand's rating depends only on its best 2-card
    /// subset. Checked directly against Omaha (N=4), where brute force is
    /// still cheap enough to run in a fast test; the same branch is shared
    /// verbatim by Omaha5/Omaha6.
    #[test]
    fn test_is_nut_hi_whole_pool_matches_brute_force() {
        fn brute_force(p: PQLCardSet, b: PQLCardSet) -> bool {
            let known = p | b;
            let player_rating = PQLGame::Omaha.eval_rating(p, b);
            HandN::<4>::iter_all::<false>().all(|h| {
                let opp: PQLCardSet = h.into();
                if !(opp & known).is_empty() {
                    return true;
                }
                PQLGame::Omaha.eval_rating(opp, b) <= player_rating
            })
        }

        let cases = [
            ("As Ks 2h 3d", "Qs Js Ts"),  // straight flush: unbeatable
            ("Ah Ad 7s 8h", "Ac As Kd"), // quads: unbeatable
            ("2s 3h 4c 5d", "9s Th Jc"), // high card: very beatable
            ("Ks Kh 2s 3h", "Kd 9c 4d"), // top set: usually good, not unbeatable
            ("7c 8c 9c Tc", "6c 5h 2d"), // straight flush draw already made
        ];

        for (hand, board) in cases {
            let p = PQLCardSet::from(cards!(hand).as_slice());
            let b = PQLCardSet::from(cards!(board).as_slice());

            assert_eq!(
                is_nut_hi(PQLGame::Omaha, p, b, p | b),
                brute_force(p, b),
                "mismatch for hand={hand} board={board}",
            );
        }
    }
}
