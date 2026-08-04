use super::{
    Array, ConstrainSuit, From, Idx, RangeCard, Suit, Suit4, SuitVar, Term, TermElem, VarCondition,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct VarConditionSuit<const N: usize>(pub(crate) VarCondition<Suit4, Suit, N>)
where
    [Idx; N]: Array<Item = Idx>;

impl<const N: usize> VarConditionSuit<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    #[inline]
    #[allow(clippy::enum_glob_use)]
    fn from_term(term: &Term, var: SuitVar, self_idx: Idx) -> Self {
        use RangeCard::*;

        let mut inner = VarCondition::<Suit4, Suit, N>::default();
        let mut idx: Idx = 0;

        for e in &term.0 {
            if let TermElem::Card(c) = e
                && idx != self_idx
            {
                match c {
                    CC(_, s) | VC(_, s) | AC(s) => inner.banned |= *s,
                    CV(_, other) | VV(_, other) | AV(other) => {
                        inner.set_indices(*other == var, idx as usize);
                    }
                    _ => (),
                }
            }

            // A span's own suit-variable correlation is handled entirely
            // within `constrain::span_suit_constrain`; it doesn't otherwise
            // contribute to variables elsewhere in the term (a List can't
            // carry a suit variable/concrete suit constraint either) — a
            // span/list here just needs its flattened width counted so
            // later cards get the right index.
            idx += match e {
                TermElem::Span(s) => s.elems().len().to_le_bytes()[0],
                TermElem::Card(_) | TermElem::List(_) => 1,
            };
        }

        Self(inner)
    }
}

impl<const N: usize> From<(&Term, SuitVar, Idx)> for VarConditionSuit<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    fn from((t, v, i): (&Term, SuitVar, Idx)) -> Self {
        Self::from_term(t, v, i)
    }
}

impl<const N: usize> From<(&Term, SuitVar, Idx)> for ConstrainSuit<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    fn from((t, v, i): (&Term, SuitVar, Idx)) -> Self {
        Self::Var(VarConditionSuit::from_term(t, v, i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn assert_varcond(
        (term, var, self_idx): (&str, SuitVar, Idx),
        expected: (&[Idx], &[Idx], Suit4),
    ) {
        assert!(self_idx < 7);

        let cond = VarConditionSuit::<7>::from((&parse_term(term).unwrap(), var, self_idx));

        assert_eq!(cond.0.equal.as_slice(), expected.0);
        assert_eq!(cond.0.not_equal.as_slice(), expected.1);
        assert_eq!(cond.0.banned, expected.2);
    }

    #[test]
    fn test_var_info_suit() {
        use SuitVar::*;

        // Flattened slots: x@0, [c]@1, [AdKh-]@2-3 (span eats 2), y@4, s@5, x@6.
        let t = "x[c][AdKh-]ysx";
        assert_varcond((t, Y, 4), (&[], &[0, 6], s4!("s")));
        assert_varcond((t, X, 0), (&[6], &[4], s4!("s")));

        assert_varcond(("xAs", X, 0), (&[], &[], s4!("s")));
        assert_varcond(("xOs", X, 0), (&[], &[], s4!("s")));
        assert_varcond(("xs", X, 0), (&[], &[], s4!("s")));

        assert_varcond(("xAy", X, 0), (&[], &[1], Suit4::default()));
        assert_varcond(("xRy", X, 0), (&[], &[1], Suit4::default()));
        assert_varcond(("xy", X, 0), (&[], &[1], Suit4::default()));

        assert_varcond(("xAx", X, 0), (&[1], &[], Suit4::default()));
        assert_varcond(("xRx", X, 0), (&[1], &[], Suit4::default()));
        assert_varcond(("xx", X, 0), (&[1], &[], Suit4::default()));
    }
}
