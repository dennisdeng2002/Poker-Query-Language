use super::{CardCount, RoundBitsInner, RoundIndex, WaughIndex};

#[inline]
pub const fn dot_product<const N: usize>(a: [WaughIndex; N], b: [WaughIndex; N]) -> WaughIndex {
    let mut res = 0;
    let mut i = 0;

    while i < N {
        res += a[i] * b[i];
        i += 1;
    }

    res
}

#[inline]
pub const fn to_wide<const N: usize>(input: [RoundIndex; N]) -> [WaughIndex; N] {
    let mut res = [0; N];
    let mut i = 0;

    while i < N {
        res[i] = input[i] as WaughIndex;
        i += 1;
    }

    res
}

#[inline]
pub const fn suffix_product(input: &mut [WaughIndex]) {
    let mut acc = 1;
    let mut i = input.len();
    while i > 0 {
        i -= 1;
        let cur = input[i];
        input[i] = acc;
        acc *= cur;
    }
}

#[inline]
pub const fn prefix_sum(input: &mut [WaughIndex]) {
    let mut acc = 0;
    let mut i = 0;
    while i < input.len() {
        let cur = input[i];
        input[i] = acc;
        acc += cur;
        i += 1;
    }
}

#[expect(clippy::cast_possible_truncation, reason = "max num of bits == 16")]
#[inline]
pub const fn colex_encode(mut bits: RoundBitsInner) -> RoundIndex {
    let mut index = 0;
    let mut i = 1;

    while bits != 0 {
        let pos = bits.trailing_zeros() as CardCount;
        index += binom_coeff_u16(pos, i);
        bits &= bits - 1;
        i += 1;
    }

    index
}

#[inline]
pub const fn colex_decode(mut index: RoundIndex, n: CardCount, r: CardCount) -> RoundBitsInner {
    let mut bits = 0;
    let mut high = n;
    let mut i = r;
    while i > 0 {
        let res = greatest_n_with_ncr_le(index, high, i);
        bits |= 1 << res;
        index -= binom_coeff_u16(res, i);
        high = res;
        i -= 1;
    }

    bits
}

#[inline]
const fn smaller_r(n: CardCount, r: CardCount) -> CardCount {
    if r < n - r { r } else { n - r }
}

#[inline]
const fn mid(a: CardCount, b: CardCount) -> CardCount {
    a + (b - a) / 2
}

#[inline]
const fn greatest_n_with_ncr_le(index: RoundIndex, mut high: CardCount, r: CardCount) -> CardCount {
    let mut low = r - 1;
    let mut res = low;
    while low < high {
        let n = mid(low, high);
        if binom_coeff_u16(n, r) <= index {
            res = n;
            low = n + 1;
        } else {
            high = n;
        }
    }
    res
}

#[inline]
pub const fn binom_coeff_u16(n: CardCount, r: CardCount) -> RoundIndex {
    if n < r {
        return 0;
    }
    let r = smaller_r(n, r);

    let mut sum = 1;
    let mut i = 1;
    while i <= r {
        sum = sum * (n - r + i) as RoundIndex / i as RoundIndex;
        i += 1;
    }
    sum
}

#[inline]
const fn binom_coeff_u64(n: WaughIndex, r: WaughIndex) -> WaughIndex {
    if n < r {
        return 0;
    }

    let mut sum = 1;
    let mut i = 1;
    while i <= r {
        sum = sum * (n - r + i) / i;
        i += 1;
    }
    sum
}

#[inline]
pub const fn multi(items: CardCount, groups: WaughIndex) -> WaughIndex {
    binom_coeff_u64(items as WaughIndex + groups - 1, items as WaughIndex)
}

#[inline]
pub const fn const_sort<const N: usize>(arr: &mut [WaughIndex; N]) {
    let mut i = 1;
    while i < N {
        let mut j = i;
        while j > 0 && arr[j] < arr[j - 1] {
            let tmp = arr[j - 1];
            arr[j - 1] = arr[j];
            arr[j] = tmp;
            j -= 1;
        }
        i += 1;
    }
}

#[inline]
pub const fn colex_multi_encode<const N: usize>(mut indices: [WaughIndex; N]) -> WaughIndex {
    const_sort(&mut indices);
    let mut res = 0;
    let mut i = 0;

    while i < N {
        res += binom_coeff_u64(indices[i] + i as WaughIndex, i as WaughIndex + 1);
        i += 1;
    }

    res
}

#[inline]
pub const fn colex_multi_decode<const N: usize>(
    mut index: WaughIndex,
    n: WaughIndex,
    r: CardCount,
) -> [WaughIndex; N] {
    let mut indices = [0; N];
    let mut high = n;
    let mut i = r;
    while i > 0 {
        let res = greatest_n_with_ncr_le_wide(index, high, i as WaughIndex);
        indices[i as usize - 1] = res - (i - 1) as WaughIndex;
        index -= binom_coeff_u64(res, i as WaughIndex);
        high = res;
        i -= 1;
    }

    indices
}

#[inline]
const fn mid_wide(a: WaughIndex, b: WaughIndex) -> WaughIndex {
    a + (b - a) / 2
}

#[inline]
const fn greatest_n_with_ncr_le_wide(
    index: WaughIndex,
    mut high: WaughIndex,
    r: WaughIndex,
) -> WaughIndex {
    let mut low = r - 1;
    let mut res = low;
    while low < high {
        let n = mid_wide(low, high);
        if binom_coeff_u64(n, r) <= index {
            res = n;
            low = n + 1;
        } else {
            high = n;
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck]
    fn test_suffix_product(numbers: Vec<WaughIndex>) {
        fn build_xs(numbers: &[WaughIndex]) -> Vec<WaughIndex> {
            let mut xs = Vec::new();
            let mut acc: WaughIndex = 1;
            for v in numbers {
                let v = v % WaughIndex::from(u16::MAX) + 1;
                match acc.checked_mul(v) {
                    Some(p) => acc = p,
                    None => break,
                }
                xs.push(v);
            }

            if xs.is_empty() { vec![100] } else { xs }
        }

        let mut xs = build_xs(&numbers);

        let product = xs
            .iter()
            .enumerate()
            .map(|(i, _)| xs[i + 1..].iter().product::<WaughIndex>())
            .collect::<Vec<_>>();
        suffix_product(&mut xs);
        assert_eq!(product, xs);
    }

    #[quickcheck]
    fn test_const_sort(mut numbers: [WaughIndex; 10]) {
        let mut expected = [0; 10];
        expected.copy_from_slice(&numbers);
        expected.sort_unstable();

        const_sort(&mut numbers);
        assert_eq!(expected, numbers);
    }
}
