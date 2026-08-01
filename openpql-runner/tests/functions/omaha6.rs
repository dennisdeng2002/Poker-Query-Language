use crate::common::{assert_count_all, assert_count_none};

#[test]
fn omaha6_uses_exactly_two_hole_cards_for_straight_flush() {
    assert_count_all(
        "select count(handtype(hero, flop) = straightflush) \
         from game='omaha6', hero='AhKh2c3d9s4c', board='QhJhTh'",
    );
}

#[test]
fn omaha6_cannot_flush_with_suited_hole_cards_on_two_suit_board() {
    assert_count_none(
        "select count(handtype(hero, flop) = flush) \
         from game='omaha6', hero='AsKsQsJs2h4c', board='Ts9s4h'",
    );
}

#[test]
fn omaha6_sixth_hole_card_completes_full_house() {
    assert_count_all(
        "select count(handtype(hero, flop) = fullhouse) \
         from game='omaha6', hero='Ah2s3dQc9c5d', board='9h9d5c'",
    );
}

#[test]
fn omaha6_hero_range_with_six_cards() {
    assert_count_all(
        "select count(handtype(hero, flop) = pair) \
         from game='omaha6', hero='AAKKQQ', board='2c7d9s'",
    );
}

#[test]
fn omaha6_pocket_pair_outside_first_four_cards_makes_set() {
    // Rank-ascending hole-card order is 2s,3h,4d,5c,Kd,Kc; a flop-category
    // evaluator hardcoded to only check pairs among the first 4 cards can
    // never see the (Kd,Kc) pocket pair, and would report a lesser category
    // instead of the correct set.
    assert_count_all(
        "select count(flophandcategory(hero) = FLOPSET) \
         from game='omaha6', hero='2s3h4d5cKdKc', board='Kh9c2d'",
    );
}
