use crate::Card;

/// Binomial coefficient `C(n, r)`
#[inline]
pub(in crate::iterator) const fn binom(n: usize, r: usize) -> usize {
    debug_assert!(n <= Card::N_CARDS as usize); // LCOV_EXCL_LINE
    if r > n {
        return 0;
    }
    let k = if r < n - r { r } else { n - r };
    let mut r = 1;
    let mut i = 1;
    while i <= k {
        r = r * (n - k + i) / i;
        i += 1;
    }
    r
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::*;

    #[quickcheck]
    fn test_pascals_identity(n: usize, r: usize) -> TestResult {
        if n == 0 || n > 52 || r == 0 || r > n {
            return TestResult::discard();
        }
        let lhs = binom(n, r);
        let rhs1 = binom(n - 1, r - 1);
        let rhs2 = binom(n - 1, r);

        TestResult::from_bool(lhs == rhs1 + rhs2)
    }

    #[test]
    fn test_ncr() {
        assert_eq!(binom(52, 4), 270_725); // Omaha
        assert_eq!(binom(52, 5), 2_598_960);
    }
}
