use std::cmp;

use crate::{Board, Card64, FlopHandCategory, eval::flop::eval_flop_holdem};

/// Returns the flop-hand category of an Omaha hand against `board`.
///
/// Enumerates every 2-card combination of `player`'s hole cards (not just
/// the first 4), so this is correct for any Omaha variant regardless of how
/// many hole cards are dealt.
// TODO: refactor
pub fn eval_flop_omaha(player: Card64, board: Board) -> FlopHandCategory {
    let player_cards: Vec<_> = player.iter().collect();
    let n = player_cards.len();

    let mut max = FlopHandCategory::default();

    for i in 0..n {
        for j in (i + 1)..n {
            let hand: Card64 = [player_cards[i], player_cards[j]].into_iter().collect();
            let cur = eval_flop_holdem(hand, board);

            if cur.compare::<false>(max) == cmp::Ordering::Greater {
                max = cur;
            }
        }
    }

    max
}

#[cfg(test)]
mod tests {
    use FlopHandCategory::*;

    use super::*;
    use crate::*;

    fn assert_flop_cat(s: &str, expected: FlopHandCategory) {
        let mut s = s.split('|');
        let p = c64!(s.next().unwrap());
        let b = board!(s.next().unwrap());

        assert_eq!(eval_flop_omaha(p, b), expected);
    }

    #[test]
    fn test_flop_hand_category_omaha() {
        assert_flop_cat("3d 6c As Ks | Qs Js Ts", StraightFlush);
        assert_flop_cat("3d 6c As Ah | Ad Ac Ks", Quads);
        assert_flop_cat("3d 6c As Ah | Ad Kc Ks", FullHouse);
        assert_flop_cat("3d 6c As Ks | Qs Js 9s", Flush);
        assert_flop_cat("3d 6c As Kh | Qd Jc Ts", Straight);
        assert_flop_cat("3d 6c As Ah | Ad Kc Qs", Set);
        assert_flop_cat("3d 6c As 2h | Ad Ac Qs", Trips);
        assert_flop_cat("3d 6c Js Qh | Td Jc Qs", TopTwo);
        assert_flop_cat("3d 6c Ts Qh | Td Jc Qs", TopAndBottom);
        assert_flop_cat("3d 6c Js Th | Td Jc Qs", BottomTwo);
        assert_flop_cat("3d 6c As Ah | Kd Qc Js", Overpair);
        assert_flop_cat("3d 6c Ks 2h | Kd Qc Js", TopPair);
        assert_flop_cat("3d 6c Qs Qh | Kd Tc 7s", Pocket12);
        assert_flop_cat("3d 6c Ts 2h | Kd Tc 7s", SecondPair);
        assert_flop_cat("3d 6c 9s 9h | Kd Tc 7s", Pocket23);
        assert_flop_cat("3d 6c 7h 2h | Kd Tc 7s", ThirdPair);
        assert_flop_cat("3d 6c 2s 2h | Kd Tc 7s", UnderPair);
        assert_flop_cat("3d 6c As Kh | Qd Jc 9s", Nothing);
    }
}
