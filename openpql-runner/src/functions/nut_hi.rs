use openpql_prelude::HoleCardRule;

use super::*;

#[pqlfn]
pub fn nut_hi(ctx: &PQLFnContext, player: PQLPlayer, street: PQLStreet) -> PQLBoolean {
    let p64 = ctx.get_c64_player(player);
    let b64 = ctx.get_c64_board(street);
    let known_cards = p64 | b64;

    let player_rating = ctx.eval_current_rating(player, street);

    match ctx.game.to_hole_card_rule() {
        // A hand's rating depends only on its best 2 hole cards, so the best
        // rating any opponent could possibly hold is exactly the rating of
        // the whole unseen-card pool evaluated as one hand — no need to
        // enumerate individual opponent hands at all.
        HoleCardRule::UseExactlyTwo => {
            let unseen = PQLCardSet::all::<false>() & !known_cards;
            ctx.game.eval_rating(unseen, b64) <= player_rating
        }
        HoleCardRule::UseAny => {
            for other in ctx.iter_c64_player() {
                if !(other & known_cards).is_empty() {
                    continue;
                }

                // TODO: cache
                let other_rating = ctx.game.eval_rating(other, b64);

                if other_rating > player_rating {
                    return false;
                }
            }

            true
        }
    }
}
