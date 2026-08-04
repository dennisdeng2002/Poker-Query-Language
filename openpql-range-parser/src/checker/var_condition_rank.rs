use super::{
    Array, ConstrainRank, From, Idx, RangeCard, Rank, Rank16, RankVar, Term, TermElem, VarCondition,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct VarConditionRank<const N: usize>(pub(crate) VarCondition<Rank16, Rank, N>)
where
    [Idx; N]: Array<Item = Idx>;

impl<const N: usize> VarConditionRank<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    #[inline]
    #[allow(clippy::enum_glob_use)]
    fn from_term(term: &Term, var: RankVar, self_idx: Idx) -> Self {
        use RangeCard::*;

        let mut inner = VarCondition::<Rank16, Rank, N>::default();
        let mut idx: Idx = 0;

        for e in &term.0 {
            match e {
                TermElem::Card(c) if idx != self_idx => match c {
                    CC(r, _) | CV(r, _) | CA(r) => inner.banned |= *r,
                    VC(other, _) | VV(other, _) | VA(other) => {
                        inner.set_indices(*other == var, idx as usize);
                    }
                    _ => (),
                },
                // A span's rank at a "sliding" position varies dynamically
                // across the hands it generates, so (unlike suit) it can't
                // safely contribute to `banned` here without duplicating
                // `constrain`'s depth/mask math — left unhandled.
                _ => (), // self, a List, or a Span
            }

            idx += match e {
                TermElem::Span(s) => s.elems().len().to_le_bytes()[0],
                TermElem::Card(_) | TermElem::List(_) => 1,
            };
        }

        Self(inner)
    }
}

impl<const N: usize> From<(&Term, RankVar, Idx)> for VarConditionRank<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    fn from((t, v, i): (&Term, RankVar, Idx)) -> Self {
        Self::from_term(t, v, i)
    }
}

impl<const N: usize> From<(&Term, RankVar, Idx)> for ConstrainRank<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    fn from((t, v, i): (&Term, RankVar, Idx)) -> Self {
        Self::Var(VarConditionRank::from((t, v, i)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn assert_varcond(
        (term, var, self_idx): (&str, RankVar, Idx),
        expected: (&[Idx], &[Idx], Rank16),
    ) {
        assert!(self_idx < 7);

        let cond = VarConditionRank::<7>::from((&parse_term(term).unwrap(), var, self_idx));

        assert_eq!(cond.0.equal.as_slice(), expected.0);
        assert_eq!(cond.0.not_equal.as_slice(), expected.1);
        assert_eq!(cond.0.banned, expected.2);
    }

    #[test]
    fn test_var_info_rank() {
        use RankVar::*;

        // Flattened slots: R@0, [A,K]@1, [AK-]@2-3 (span eats 2), B@4, A@5, R@6.
        let t = "R[A,K][AK-]BAR";
        assert_varcond((t, RB, 4), (&[], &[0, 6], r16!("A")));
        assert_varcond((t, RR, 0), (&[6], &[4], r16!("A")));

        assert_varcond(("RAs", RR, 0), (&[], &[], r16!("A")));
        assert_varcond(("RAw", RR, 0), (&[], &[], r16!("A")));
        assert_varcond(("RA", RR, 0), (&[], &[], r16!("A")));

        assert_varcond(("ROs", RR, 0), (&[], &[1], Rank16::default()));
        assert_varcond(("ROw", RR, 0), (&[], &[1], Rank16::default()));
        assert_varcond(("RO", RR, 0), (&[], &[1], Rank16::default()));

        assert_varcond(("RRs", RR, 0), (&[1], &[], Rank16::default()));
        assert_varcond(("RRx", RR, 0), (&[1], &[], Rank16::default()));
        assert_varcond(("RR", RR, 0), (&[1], &[], Rank16::default()));
    }
}
