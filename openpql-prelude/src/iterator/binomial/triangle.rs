//! Flat row-major storage of Pascal's triangle.

/// Precomputed Pascal's (Yang Hui's) triangle for repeated binomial-coefficient lookups.
///
/// Stored row-major: row `n` (cells `C(n, 0..=n)`) begins at the `n`-th triangular number
/// `n*(n+1)/2`.
#[derive(Clone)]
pub(in crate::iterator) struct Triangle {
    cells: Vec<usize>,
    max_n: usize,
}

impl Triangle {
    /// Builds the triangle for every row in `0..=max_n`.
    pub fn new(max_n: usize) -> Self {
        let mut triangle = Self::zeroed(max_n);

        for n in 0..=max_n {
            triangle.set(n, 0, 1);
            for k in 1..=n {
                let value = if k == n {
                    1
                } else {
                    triangle.get(n - 1, k - 1) + triangle.get(n - 1, k)
                };
                triangle.set(n, k, value);
            }
        }

        triangle
    }

    /// Allocates zeroed storage for every row in `0..=max_n`.
    fn zeroed(max_n: usize) -> Self {
        Self {
            cells: vec![0; Self::row_offset(max_n + 1)],
            max_n,
        }
    }

    /// Offset of row `n`: the `n`-th triangular number.
    const fn row_offset(n: usize) -> usize {
        n * (n + 1) / 2
    }

    fn set(&mut self, n: usize, k: usize, value: usize) {
        self.cells[Self::row_offset(n) + k] = value;
    }

    /// `C(n, k)`, or 0 when out of range.
    #[inline]
    pub fn get(&self, n: usize, k: usize) -> usize {
        if k > n || n > self.max_n {
            return 0;
        }
        self.cells[Self::row_offset(n) + k]
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn render_pascal(max_n: usize) -> String {
        let t = Triangle::new(max_n);

        let mut out = String::new();
        for n in 0..=t.max_n {
            for _ in n..t.max_n {
                out.push(' ');
            }
            for k in 0..=n {
                if k > 0 {
                    out.push(' ');
                }
                out.push_str(&t.get(n, k).to_string());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn test_row_offset() {
        assert_eq!(Triangle::row_offset(0), 0);
        assert_eq!(Triangle::row_offset(1), 1);
        assert_eq!(Triangle::row_offset(4), 10);
    }

    #[test]
    fn test_set_get_visualized() {
        assert_eq!(
            render_pascal(4),
            concat!(
                "    1\n",
                "   1 1\n",
                "  1 2 1\n",
                " 1 3 3 1\n",
                "1 4 6 4 1\n",
            )
        );
    }

    #[test]
    fn test_get_out_of_range() {
        let t = Triangle::new(3);
        assert_eq!(t.get(2, 3), 0);
        assert_eq!(t.get(4, 0), 0);
    }
}
