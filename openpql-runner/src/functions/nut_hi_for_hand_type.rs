use openpql_prelude::{HandN, HoleCardRule};

use super::*;

#[pqlfn]
pub fn nut_hi_for_hand_type(
    ctx: &PQLFnContext,
    player: PQLPlayer,
    street: PQLStreet,
) -> PQLBoolean {
    let p64 = ctx.get_c64_player(player);
    let b64 = ctx.get_c64_board(street);
    let known_cards = p64 | b64;

    let player_rating = ctx.eval_current_rating(player, street);
    let player_ht = PQLHandType::from(player_rating);

    let beats = |other: PQLCardSet| -> bool {
        if !(other & known_cards).is_empty() {
            return false;
        }

        // TODO: cache
        let other_rating = ctx.game.eval_rating(other, b64);

        PQLHandType::from(other_rating) == player_ht && other_rating > player_rating
    };

    match ctx.game.to_hole_card_rule() {
        // A hand's rating depends only on its best 2 hole cards, so every
        // rating any opponent could hold is achieved by some 2-card subset
        // of the unseen pool; there's no need to enumerate full-size hands.
        HoleCardRule::UseExactlyTwo => !HandN::<2>::iter_all::<false>().any(|h| beats(h.into())),
        HoleCardRule::UseAny => !ctx.iter_c64_player().any(beats),
    }
}
