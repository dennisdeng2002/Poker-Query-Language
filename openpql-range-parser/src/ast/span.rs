use super::{
    CardSuit, Error, LalrError, Loc, RangeCard, RankConst, RankInt, ResultE, SuitConst, SuitVar,
    TermElem, ToString,
};

/// One card in a span endpoint.
#[derive(Copy, Clone, PartialEq, Eq, Debug, derive_more::Display)]
pub enum SpanElem {
    /// Concrete rank with concrete suit.
    #[display("{_0}{_1}")]
    CC(RankConst, SuitConst),
    /// Concrete rank with suit variable.
    #[display("{_0}{_1}")]
    CV(RankConst, SuitVar),
    /// Concrete rank with any suit.
    #[display("{_0}")]
    CA(RankConst),
}

impl SpanElem {
    /// Returns the rank.
    pub const fn rank(self) -> RankConst {
        match self {
            Self::CC(r, _) | Self::CV(r, _) | Self::CA(r) => r,
        }
    }

    /// Returns the suit, or `None` when unspecified.
    pub const fn suit(self) -> Option<CardSuit> {
        match self {
            Self::CC(_, s) => Some(CardSuit::Const(s)),
            Self::CV(_, s) => Some(CardSuit::Var(s)),
            Self::CA(_) => None,
        }
    }
}

impl TryFrom<(Loc, Loc, RangeCard)> for SpanElem {
    type Error = LalrError<'static>;

    fn try_from((l, r, c): (Loc, Loc, RangeCard)) -> Result<Self, Self::Error> {
        match c {
            RangeCard::CC(r, s) => Ok(Self::CC(r, s)),
            RangeCard::CV(r, s) => Ok(Self::CV(r, s)),
            RangeCard::CA(r) => Ok(Self::CA(r)),
            _ => Err(Error::InvalidSpan((l, r)).into()),
        }
    }
}

impl TryFrom<(Loc, Loc, TermElem)> for SpanElem {
    type Error = LalrError<'static>;

    fn try_from((l, r, e): (Loc, Loc, TermElem)) -> Result<Self, Self::Error> {
        match e {
            TermElem::Card(c) => Self::try_from((l, r, c)),
            _ => Err(Error::InvalidSpan((l, r)).into()),
        }
    }
}

/// Rank span across one or two endpoints.
#[derive(Clone, PartialEq, Eq, Debug, derive_more::Display)]
pub enum Span {
    /// Open-ended downward span like `AK-`.
    #[display("{}-", to_str(_0))]
    Down(Vec<SpanElem>),
    /// Open-ended upward span like `AK+`.
    #[display("{}+", to_str(_0))]
    Up(Vec<SpanElem>),
    /// Bounded span like `AK-QJ` from top to bottom.
    #[display("{}-{}", to_str(_0), to_str(_1))]
    To(Vec<SpanElem>, Vec<SpanElem>),
}

fn to_str(elems: &[SpanElem]) -> String {
    elems.iter().map(ToString::to_string).collect::<String>()
}

impl Span {
    /// The span's own elements, regardless of which form it is.
    pub(crate) fn elems(&self) -> &[SpanElem] {
        match self {
            Self::Down(v) | Self::Up(v) | Self::To(v, _) => v,
        }
    }
}

#[inline]
fn to_span_elems<'i, T>(l: Loc, r: Loc, cs: Vec<T>) -> ResultE<'i, Vec<SpanElem>>
where
    SpanElem: TryFrom<(Loc, Loc, T), Error = LalrError<'i>>,
{
    cs.into_iter()
        .map(|c| SpanElem::try_from((l, r, c)))
        .collect()
}

/// Positions where `v1`/`v2` disagree on rank — the ones that actually vary
/// across the bound. Positions written with the same rank at both ends are
/// anchors: fixed, and independent of everything else in the span.
#[inline]
fn sliding_indices(v1: &[SpanElem], v2: &[SpanElem]) -> Vec<usize> {
    (0..v1.len()).filter(|&i| v1[i].rank() != v2[i].rank()).collect()
}

#[inline]
fn ensure_same_format<'i>(l: Loc, r: Loc, v1: &[SpanElem], v2: &[SpanElem]) -> ResultE<'i, ()> {
    let len = v1.len();

    if v2.len() != len {
        return Err(Error::NumberOfRanksMismatchInSpan((l, r)).into());
    }

    // Sliding positions must all move by the same rank distance between the
    // two bounds; anchored positions (same rank at both ends) are exempt.
    for w in sliding_indices(v1, v2).windows(2) {
        let (i, j) = (w[0], w[1]);
        if v1[j].rank() as RankInt - v1[i].rank() as RankInt
            != v2[j].rank() as RankInt - v2[i].rank() as RankInt
        {
            return Err(Error::RankDistanceMismatchInSpan((l, r)).into());
        }
    }

    for i in 0..len {
        if v1[i].suit() != v2[i].suit() {
            return Err(Error::SuitMismatchInSpan((l, r)).into());
        }
    }

    Ok(())
}

impl<'i> Span {
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn spandown<T>(l: Loc, cs: Vec<T>, r: Loc) -> ResultE<'i, Self>
    where
        SpanElem: TryFrom<(Loc, Loc, T), Error = LalrError<'i>>,
    {
        Ok(Self::Down(to_span_elems(l, r, cs)?))
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn spanup<T>(l: Loc, cs: Vec<T>, r: Loc) -> ResultE<'i, Self>
    where
        SpanElem: TryFrom<(Loc, Loc, T), Error = LalrError<'i>>,
    {
        Ok(Self::Up(to_span_elems(l, r, cs)?))
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn spanto<T>(l: Loc, top: Vec<T>, btm: Vec<T>, r: Loc) -> ResultE<'i, Self>
    where
        SpanElem: TryFrom<(Loc, Loc, T), Error = LalrError<'i>>,
    {
        let t = to_span_elems(l, r, top)?;
        let b = to_span_elems(l, r, btm)?;

        ensure_same_format(l, r, &t, &b).map(|()| {
            let i0 = sliding_indices(&t, &b).first().copied().unwrap_or(0);

            if t[i0].rank() > b[i0].rank() {
                Self::To(t, b)
            } else {
                Self::To(b, t)
            }
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::*;

    fn assert_span(src: &str, expected: &str) {
        let span = parse_span(src).unwrap();

        assert_eq!(span.to_string(), expected, "{span} != {expected}");
    }

    const fn valid_card(c: RangeCard) -> bool {
        matches!(c, RangeCard::CC(_, _) | RangeCard::CV(_, _) | RangeCard::CA(_))
    }

    #[quickcheck]
    fn test_spandown(c1: RangeCard, c2: RangeCard) -> TestResult {
        if !valid_card(c1) || !valid_card(c2) {
            return TestResult::discard();
        }

        let src = format!("{c1}{c2}-");
        assert_span(&src, &src);

        TestResult::passed()
    }

    #[quickcheck]
    fn test_spanup(c1: RangeCard, c2: RangeCard) -> TestResult {
        if !valid_card(c1) || !valid_card(c2) {
            return TestResult::discard();
        }

        let src = format!("{c1}{c2}+");
        assert_span(&src, &src);

        TestResult::passed()
    }

    #[quickcheck]
    fn test_spanto(
        a1: RankConst,
        a2: RankConst,
        b1: RankConst,
        b2: RankConst,
        s: SuitConst,
    ) -> TestResult {
        if (a1 as RankInt - a2 as RankInt) != (b1 as RankInt - b2 as RankInt) {
            return TestResult::discard();
        }

        let src = format!("{a1}{b1}{s}-{a2}{b2}{s}");

        let expected = if a1 > a2 {
            src.clone()
        } else {
            format!("{a2}{b2}{s}-{a1}{b1}{s}")
        };

        assert_span(&src, &expected);

        TestResult::passed()
    }

    #[test]
    fn test_spanto_fixed() {
        assert_span("AsK-JsT", "AsK-JsT");
        assert_span("T-A", "A-T");
        assert_span("A-T", "A-T");
        assert_span("TT-AA", "AA-TT");
        assert_span("AA-TT", "AA-TT");
    }

    #[test]
    fn test_span_suit_var() {
        assert_span("Aw-", "Aw-");
        assert_span("AwJw+", "AwJw+");
        assert_span("AwKw+", "AwKw+");
        assert_span("KwQw-JwTw", "KwQw-JwTw");
        assert_span("AxKy+", "AxKy+");
    }

    #[test]
    fn test_span_anchor_kicker() {
        // Anchor (A) fixed, kicker slides — no longer a rank-distance mismatch.
        assert_span("AwTw-Aw9w", "AwTw-Aw9w");
        assert_span("Aw9w-AwTw", "AwTw-Aw9w");
        assert_span("KwQw-KwTw", "KwQw-KwTw");
        // Fully anchored (identical hand on both ends) is a degenerate valid case.
        assert_span("AsKs-AsKs", "AsKs-AsKs");
    }

    #[test]
    fn test_span_error() {
        assert_err(
            parse_span("A-AK"),
            Error::NumberOfRanksMismatchInSpan((0, 4)),
        );

        assert_err(
            parse_span("AA-QT"),
            Error::RankDistanceMismatchInSpan((0, 5)),
        );

        assert_err(parse_span("As-K"), Error::SuitMismatchInSpan((0, 4)));
    }

    #[test]
    fn test_span_error_invalid() {
        assert_err(parse_span("B-    "), Error::InvalidSpan((0, 2)));
        assert_err(parse_span("B+    "), Error::InvalidSpan((0, 2)));
        assert_err(parse_span("B-A   "), Error::InvalidSpan((0, 3)));
        assert_err(parse_span("A-B   "), Error::InvalidSpan((0, 3)));
        assert_err(parse_span("Bs-   "), Error::InvalidSpan((0, 3)));
        assert_err(parse_span("*w-   "), Error::InvalidSpan((0, 3)));
        assert_err(parse_span("Bw-   "), Error::InvalidSpan((0, 3)));
        assert_err(parse_span("*-    "), Error::InvalidSpan((0, 2)));
    }
}
