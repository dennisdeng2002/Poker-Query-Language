mod common;

use common::run_ok;

/// Regression: a pocket pair whose rank matches a card already fixed on the
/// board (e.g. hero holds a 9 while the board pins 9h/9s) is legal but rare
/// — only combos avoiding the board's own copies of that rank are valid, so
/// the success rate can fall well under 50%.
///
/// `run_trials`'s old bail-out fired as soon as failed samples equaled the
/// trial target, which is reached *before* enough successes accumulate
/// whenever the true success probability is under ~50%. That misfired here
/// even though the query is perfectly satisfiable (e.g. hero holding 9c9d),
/// turning a rare-but-valid case into a hard `SamplingFailed`.
#[test]
fn pair_colliding_with_paired_board_still_samples() {
    let out = run_ok(
        "select count(pocketpair(hero)) \
         from game='holdem', hero='99', villain='AcKc', board='9h9s4d'",
    );

    assert!(out.contains("trials"), "unexpected output:\n{out}");
}

#[test]
fn pair_colliding_with_single_board_card_still_samples() {
    let out = run_ok(
        "select count(pocketpair(hero)) \
         from game='holdem', hero='77', villain='AsKs', board='9h4s2d7c'",
    );

    assert!(out.contains("trials"), "unexpected output:\n{out}");
}
