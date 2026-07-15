macro_rules! const_cmp_opt {
    ($lhs:expr, $rhs:expr) => {{
        match ($lhs, $rhs) {
            (None, Some(_)) => return true,
            (Some(_), None) => return false,
            (Some(a), Some(b)) => {
                if a.const_lt(b) {
                    return true;
                }
                if b.const_lt(a) {
                    return false;
                }
            }
            (None, None) => {}
        }
    }};
}

pub(crate) use const_cmp_opt;
