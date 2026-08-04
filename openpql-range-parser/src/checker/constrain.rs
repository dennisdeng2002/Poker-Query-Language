use super::{
    Array, Card, Card64, CardSuit, ConstrainRank, ConstrainSuit, From, Idx, List, ListElem,
    RangeCard, Rank16, RankDiff, Span, SpanElem, Term, VarCondition, VarConditionSuit,
};

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub(super) struct Constrain<const N: usize>
where
    [Idx; N]: Array<Item = Idx>,
{
    pub c64: Option<Card64>,
    pub rank: ConstrainRank<N>,
    pub suit: ConstrainSuit<N>,
}

impl<const N: usize> Constrain<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    #[inline]
    pub fn reject(constrains: &[Self], cs: &[Card], perm: &[Idx]) -> bool {
        for (i, &p) in perm.iter().enumerate() {
            let constrain = &constrains[p as usize];

            if let Some(c64) = constrain.c64
                && !c64.contains_card(cs[i])
            {
                return true;
            }

            if constrain.rank.reject(cs, perm, i) {
                return true;
            }

            if constrain.suit.reject(cs, perm, i) {
                return true;
            }
        }

        false
    }

    #[allow(clippy::enum_glob_use)]
    pub fn from_card(t: &Term, card: RangeCard, i: Idx) -> Self {
        use RangeCard::*;

        match card {
            AA => Self::default(),
            CC(r, s) => (ConstrainRank::from(r), ConstrainSuit::from(s)).into(),
            CA(r) => ConstrainRank::from(r).into(),
            AC(s) => ConstrainSuit::from(s).into(),
            CV(r, sv) => (ConstrainRank::from(r), ConstrainSuit::from((t, sv, i))).into(),

            VC(rv, s) => (ConstrainRank::from((t, rv, i)), ConstrainSuit::from(s)).into(),

            VV(rv, sv) => (
                ConstrainRank::from((t, rv, i)),
                ConstrainSuit::from((t, sv, i)),
            )
                .into(),
            VA(rv) => ConstrainRank::from((t, rv, i)).into(),
            AV(sv) => ConstrainSuit::from((t, sv, i)).into(),
        }
    }

    pub fn from_list(cs: &List) -> Self {
        let mut c64: Card64 = Card64::default();

        for c in &cs.0 {
            match c {
                ListElem::CC(r, s) => {
                    c64 |= Card64::from(Card::new(*r, *s));
                }
                ListElem::CA(r) => {
                    c64 |= Card64::from_ranks(Rank16::from(*r));
                }
                ListElem::AC(s) => {
                    c64 |= Card64::from_suit(*s);
                }
            }
        }

        c64.into()
    }

    fn from_span_head(span: &Span, start_idx: Idx) -> Self {
        let elems = span_elems(span);
        let head = head(span);

        Self::from((
            ConstrainRank::from(r16_from_depth(head.rank() as u8, span_depth(span))),
            span_suit_constrain(elems, 0, start_idx),
        ))
    }

    /// Open-ended spans (`+`/`-`): every position slides together, preserving
    /// the rank gaps between them (pairs-plus, connectors, etc).
    fn from_span_sliding(span: &Span, start_idx: Idx) -> Vec<Self> {
        let elems = span_elems(span);
        let len = elems.len();
        let mut res = Vec::with_capacity(len);
        let depth = span_depth(span);

        res.push(Self::from_span_head(span, start_idx));

        for (prev_idx, i) in (start_idx..).zip(1..len) {
            res.push(
                (
                    ConstrainRank::Diff(
                        prev_idx,
                        elems[i - 1].rank() as RankDiff - elems[i].rank() as RankDiff,
                        r16_from_depth(elems[i].rank() as u8, depth),
                    ),
                    span_suit_constrain(elems, i, start_idx),
                )
                    .into(),
            );
        }

        res
    }

    /// Bounded spans (`a-b`): positions written with the same rank at both
    /// ends are fixed anchors; the rest ("sliding") must move together
    /// preserving their own pairwise rank gaps, independent of the anchors.
    fn from_span_to(t: &[SpanElem], b: &[SpanElem], start_idx: Idx) -> Vec<Self> {
        let len = t.len();
        let sliding: Vec<usize> = (0..len).filter(|&i| t[i].rank() != b[i].rank()).collect();
        let depth = sliding
            .first()
            .map_or(0, |&i0| b[i0].rank() as RankDiff - t[i0].rank() as RankDiff);

        let mut res = Vec::with_capacity(len);
        let mut prev_sliding: Option<(Idx, usize)> = None;

        for (flat_idx, i) in (start_idx..).zip(0..len) {
            let is_sliding = sliding.contains(&i);

            let rank = if !is_sliding {
                ConstrainRank::from(t[i].rank())
            } else if let Some((prev_idx, prev_pos)) = prev_sliding {
                ConstrainRank::Diff(
                    prev_idx,
                    t[prev_pos].rank() as RankDiff - t[i].rank() as RankDiff,
                    r16_from_depth(t[i].rank() as u8, depth),
                )
            } else {
                ConstrainRank::from(r16_from_depth(t[i].rank() as u8, depth))
            };

            if is_sliding {
                prev_sliding = Some((flat_idx, i));
            }

            res.push((rank, span_suit_constrain(t, i, start_idx)).into());
        }

        res
    }

    pub fn from_span(span: &Span, start_idx: Idx) -> Vec<Self> {
        match span {
            Span::Up(_) | Span::Down(_) => Self::from_span_sliding(span, start_idx),
            Span::To(t, b) => Self::from_span_to(t, b, start_idx),
        }
    }
}

/// Suit constraint for the span element at `pos`, correlating suit variables
/// with sibling positions within the same span (`start_idx..start_idx+elems.len()`).
fn span_suit_constrain<const N: usize>(
    elems: &[SpanElem],
    pos: usize,
    start_idx: Idx,
) -> ConstrainSuit<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    match elems[pos].suit() {
        None => ConstrainSuit::Nil,
        Some(CardSuit::Const(s)) => ConstrainSuit::from(s),
        Some(CardSuit::Var(v)) => {
            let mut cond = VarCondition::default();

            for (j, e) in elems.iter().enumerate() {
                if j == pos {
                    continue;
                }

                match e.suit() {
                    Some(CardSuit::Const(s)) => cond.banned |= s,
                    Some(CardSuit::Var(other)) => {
                        cond.set_indices(other == v, start_idx as usize + j);
                    }
                    None => (),
                }
            }

            ConstrainSuit::Var(VarConditionSuit(cond))
        }
    }
}

#[inline]
fn head(span: &Span) -> SpanElem {
    match span {
        Span::Down(v) | Span::Up(v) | Span::To(v, _) => v[0],
    }
}

#[inline]
fn span_elems(span: &Span) -> &[SpanElem] {
    match span {
        Span::Down(v) | Span::Up(v) | Span::To(v, _) => v,
    }
}

#[inline]
fn span_depth(s: &Span) -> RankDiff {
    const RANK_I8_A: RankDiff = 12;

    match s {
        Span::Down(t) => -t.iter().map(|e| e.rank() as RankDiff).min().unwrap(),
        Span::Up(b) => RANK_I8_A - b.iter().map(|e| e.rank() as RankDiff).max().unwrap(),
        Span::To(t, b) => b[0].rank() as RankDiff - t[0].rank() as RankDiff,
    }
}

#[inline]
fn r16_from_depth(rank_u8: u8, d: RankDiff) -> Rank16 {
    let ones = 2u16.pow(u32::from(d.unsigned_abs() + 1)) - 1;

    if d > 0 {
        Rank16::from(ones << rank_u8)
    } else {
        Rank16::from(ones << rank_u8.saturating_add_signed(d))
    }
}

impl<const N: usize> From<ConstrainRank<N>> for Constrain<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    fn from(r: ConstrainRank<N>) -> Self {
        Self::from((r, ConstrainSuit::default()))
    }
}

impl<const N: usize> From<ConstrainSuit<N>> for Constrain<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    fn from(s: ConstrainSuit<N>) -> Self {
        Self::from((ConstrainRank::default(), s))
    }
}

impl<const N: usize> From<(ConstrainRank<N>, ConstrainSuit<N>)> for Constrain<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    fn from((rank, suit): (ConstrainRank<N>, ConstrainSuit<N>)) -> Self {
        Self {
            c64: None,
            rank,
            suit,
        }
    }
}

impl<const N: usize> From<Card64> for Constrain<N>
where
    [Idx; N]: Array<Item = Idx>,
{
    fn from(c: Card64) -> Self {
        Self {
            c64: Some(c),
            rank: ConstrainRank::Nil,
            suit: ConstrainSuit::Nil,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn assert_span_depth(s: &str, expected: RankDiff) {
        assert_eq!(span_depth(&parse_span(s).unwrap()), expected);
    }

    #[test]
    fn test_span_depth() {
        assert_span_depth("2-", 0);
        assert_span_depth("A+", 0);

        assert_span_depth("TQ8s-", -6);
        assert_span_depth("JTs-T9s", -1);
        assert_span_depth("K2s+", 1);
    }

    #[test]
    fn test_rank16_from_depth() {
        assert_eq!(r16_from_depth(0, 0), r16!("2"));
        assert_eq!(r16_from_depth(12, 0), r16!("A"));
        assert_eq!(r16_from_depth(4, 2), r16!("678"));
        assert_eq!(r16_from_depth(12, -5), r16!("9TJQKA"));
    }
}
