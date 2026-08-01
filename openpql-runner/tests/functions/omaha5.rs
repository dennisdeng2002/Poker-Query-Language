use crate::common::{assert_count_all, assert_count_none};

#[test]
fn omaha5_uses_exactly_two_hole_cards_for_straight_flush() {
    assert_count_all(
        "select count(handtype(hero, flop) = straightflush) \
         from game='omaha5', hero='AhKh2c3d9s', board='QhJhTh'",
    );
}

#[test]
fn omaha5_cannot_flush_with_four_suited_hole_cards_on_two_suit_board() {
    assert_count_none(
        "select count(handtype(hero, flop) = flush) \
         from game='omaha5', hero='AsKsQsJs2h', board='Ts9s4h'",
    );
}

#[test]
fn omaha5_fifth_hole_card_completes_full_house() {
    assert_count_all(
        "select count(handtype(hero, flop) = fullhouse) \
         from game='omaha5', hero='Ah2s3d9c5d', board='9h9d5c'",
    );
}

#[test]
fn omaha5_hero_range_with_five_cards() {
    assert_count_all(
        "select count(handtype(hero, flop) = pair) \
         from game='omaha5', hero='AAKKQ', board='2c7d9s'",
    );
}

#[test]
fn omaha5_pocket_pair_outside_first_four_cards_makes_set() {
    // Rank-ascending hole-card order is 2s,3h,4d,Kd,Kc; a flop-category
    // evaluator that only checks pairs among the first 4 cards can never
    // see the (Kd,Kc) pocket pair, and would report a lesser category
    // (e.g. no pair at all) instead of the correct set.
    assert_count_all(
        "select count(flophandcategory(hero) = FLOPSET) \
         from game='omaha5', hero='2s3h4dKdKc', board='Kh9c2d'",
    );
}
