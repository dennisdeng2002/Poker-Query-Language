use super::{Array, Card, Constrain, Error, Idx, LocInfo, SmallVec, ast, range_cond_indices};

#[derive(PartialEq, Eq, Debug, Clone)]
pub(super) struct Leaf<const N: usize, const B: bool>
where
    [Idx; N]: Array<Item = Idx>,
{
    constrains: [Constrain<N>; N],
}

impl<const N: usize, const B: bool> Leaf<N, B>
where
    [Idx; N]: Array<Item = Idx>,
{
    pub const fn new(constrains: [Constrain<N>; N]) -> Self {
        Self { constrains }
    }

    #[inline]
    pub fn is_satisfied(&self, cs: &[Card]) -> bool {
        // Boards pin the turn/river to fixed slots (only the flop's 3
        // cards are freely permutable), so the general matching below —
        // which allows any card to fill any slot — only applies to
        // hole-card hands, never boards.
        if !B && self.constrains.iter().all(Constrain::is_context_free) {
            return Self::has_matching(&self.constrains, cs);
        }

        let n = self.constrains.len();
        let r = cs.len();

        !range_cond_indices(n, r, B)
            .iter()
            .all(|perm| Constrain::reject(&self.constrains, cs, perm))
    }

    /// Whether every card in `cs` can be assigned a distinct constrain that
    /// accepts it (maximum bipartite matching via augmenting paths, à la
    /// Kuhn's algorithm). Cards and constrains are both `<= 6` in practice
    /// (Omaha6 is the largest hand), so this is cheap even though it's
    /// `O(cards * constrains^2)` in the worst case — far cheaper than the
    /// `O(constrains!)` brute-force permutation search it replaces.
    ///
    /// Only sound when every constrain is context-free (see
    /// [`Constrain::is_context_free`]): a rank/suit variable's acceptance
    /// of a card depends on what other slots end up matched to, which this
    /// greedy matching doesn't reconsider once decided.
    fn has_matching(constrains: &[Constrain<N>], cs: &[Card]) -> bool {
        fn augment<const N: usize>(
            constrains: &[Constrain<N>],
            card: Card,
            visited: &mut [bool],
            matched_to: &mut [Option<Card>],
        ) -> bool
        where
            [Idx; N]: Array<Item = Idx>,
        {
            for (i, constrain) in constrains.iter().enumerate() {
                if visited[i] || constrain.accepts_standalone(card) != Some(true) {
                    continue;
                }

                visited[i] = true;

                let free =
                    matched_to[i].is_none_or(|other| augment(constrains, other, visited, matched_to));

                if free {
                    matched_to[i] = Some(card);
                    return true;
                }
            }

            false
        }

        let mut matched_to: SmallVec<[Option<Card>; 6]> = SmallVec::from_elem(None, constrains.len());

        cs.iter().all(|&card| {
            let mut visited: SmallVec<[bool; 6]> = SmallVec::from_elem(false, constrains.len());

            augment(constrains, card, &mut visited, &mut matched_to)
        })
    }

    /// Reference brute-force satisfiability (the exhaustive permutation
    /// search `is_satisfied` takes the `has_matching` fast path around for
    /// context-free leaves), kept for the differential test below.
    #[cfg(test)]
    fn brute_force(constrains: &[Constrain<N>; N], cs: &[Card]) -> bool {
        let n = constrains.len();
        let r = cs.len();

        !range_cond_indices(n, r, B)
            .iter()
            .all(|perm| Constrain::reject(constrains, cs, perm))
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Deps(pub LocInfo);

impl<const N: usize, const B: bool> TryFrom<(ast::Term, Deps)> for Leaf<N, B>
where
    [Idx; N]: Array<Item = Idx>,
{
    type Error = Error;

    fn try_from((term, deps): (ast::Term, Deps)) -> Result<Self, Self::Error> {
        let mut constrains = vec![];
        let mut i = 0;

        for el in &term.0 {
            match el {
                ast::TermElem::Card(c) => {
                    constrains.push(Constrain::from_card(&term, *c, i));
                    i += 1;
                }

                ast::TermElem::List(l) => {
                    constrains.push(Constrain::from_list(l));
                    i += 1;
                }

                ast::TermElem::Span(s) => {
                    let v = Constrain::from_span(s, i);

                    i += v.len().to_le_bytes()[0];

                    constrains.extend(v);
                }
            }
        }

        if i as usize > N {
            Err(Error::TooManyCardsInRange(deps.0))
        } else {
            for _ in i as usize..N {
                constrains.push(Constrain::default());
            }

            Ok(Self::new(constrains.try_into().unwrap()))
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use quickcheck::TestResult;

    use super::{super::{ConstrainRank, ConstrainSuit}, *};
    use crate::{CardN, Rank, Rank16, Suit, Suit4};

    /// A single context-free constrain (wildcard, rank-only, suit-only, or
    /// a literal card), for building random 6-slot leaves that exercise
    /// `has_matching`'s precondition.
    #[derive(Clone, Debug)]
    struct ArbConstrain(Constrain<6>);

    impl quickcheck::Arbitrary for ArbConstrain {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let rank = if bool::arbitrary(g) {
                ConstrainRank::Match(Rank16::from(Rank::arbitrary(g)))
            } else {
                ConstrainRank::Nil
            };
            let suit = if bool::arbitrary(g) {
                ConstrainSuit::Match(Suit4::from(Suit::arbitrary(g)))
            } else {
                ConstrainSuit::Nil
            };

            Self(Constrain {
                c64: None,
                rank,
                suit,
            })
        }
    }

    /// `has_matching` (used by `is_satisfied`'s fast path) must agree with
    /// the brute-force permutation search it replaces, for every prefix
    /// length `gen_card` would actually query (1..=6 cards).
    #[quickcheck]
    fn matching_agrees_with_brute_force(
        constrains: [ArbConstrain; 6],
        cards: CardN<6>,
    ) -> TestResult {
        let constrains: [Constrain<6>; 6] = constrains.map(|c| c.0);
        let leaf = Leaf::<6, false>::new(constrains.clone());

        for r in 1..=cards.len() {
            let cs = &cards.as_slice()[..r];
            let fast = leaf.is_satisfied(cs);
            let slow = Leaf::<6, false>::brute_force(&constrains, cs);

            if fast != slow {
                return TestResult::error(format!(
                    "mismatch at r={r}: fast={fast} slow={slow}"
                ));
            }
        }

        TestResult::passed()
    }
}
