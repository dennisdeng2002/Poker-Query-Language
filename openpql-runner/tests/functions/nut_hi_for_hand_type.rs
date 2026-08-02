use crate::common::{assert_count_all, assert_count_none};

#[test]
fn top_pair_top_kicker_is_nut_within_pair() {
    assert_count_all(
        "select count(nuthiforhandtype(hero, flop)) \
         from game='holdem', hero='AsKh', villain='2c3d', board='AdTd2d'",
    );
}

#[test]
fn low_pair_is_not_nut_within_pair() {
    assert_count_none(
        "select count(nuthiforhandtype(hero, flop)) \
         from game='holdem', hero='3h3s', villain='4c5c', board='AdTd2d'",
    );
}

#[test]
fn pair_of_kings_not_nut_when_ace_on_board() {
    assert_count_none(
        "select count(nuthiforhandtype(hero, river)) \
         from game='holdem', hero='KhKs', villain='2c3c', board='Ad4h7s8d9h'",
    );
}

#[test]
fn royal_flush_is_nut_within_straight_flush() {
    assert_count_all(
        "select count(nuthiforhandtype(hero, river)) \
         from game='holdem', hero='2c3c', villain='4d5d', board='AsKsQsJsTs'",
    );
}

#[test]
fn quads_with_top_kicker_is_nut_within_quads() {
    assert_count_all(
        "select count(nuthiforhandtype(hero, flop)) \
         from game='holdem', hero='AsKh', villain='2c3d', board='AhAdAc'",
    );
}

#[test]
fn omaha6_guaranteed_quads_is_nut_within_quads() {
    assert_count_all(
        "select count(nuthiforhandtype(hero, flop)) \
         from game='omaha6', hero='AsAcKhQd3s4h', board='AhAd2c'",
    );
}

#[test]
fn omaha6_low_pocket_pair_is_not_nut_within_pair() {
    assert_count_none(
        "select count(nuthiforhandtype(hero, flop)) \
         from game='omaha6', hero='3s3h4d5c9c7d', board='8h6d2d'",
    );
}
