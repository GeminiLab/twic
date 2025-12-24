/// Convert f64 to u64 losslessly. This function ignores the sign bit.
pub const fn f64_to_u64_no_sig_lossless(n: f64) -> Option<u64> {
    if n.is_infinite() || n.is_nan() {
        return None;
    }

    let n = n.abs();

    // u64::MAX as f64 rounds up to 2^64
    if n >= (u64::MAX as f64) {
        return None;
    }

    let converted = n as u64;
    let back_converted = converted as f64;

    if back_converted == n {
        Some(converted)
    } else {
        None
    }
}

/// Convert f64 to u128 losslessly. This function ignores the sign bit.
pub const fn f64_to_u128_no_sig_lossless(n: f64) -> Option<u128> {
    if n.is_infinite() || n.is_nan() {
        return None;
    }

    let n = n.abs();

    // u128::MAX as f64 rounds up to 2^128
    if n >= (u128::MAX as f64) {
        return None;
    }

    let converted = n as u128;
    let back_converted = converted as f64;

    if back_converted == n {
        Some(converted)
    } else {
        None
    }
}

/// Convert f64 to f32 losslessly.
pub const fn f64_to_f32_lossless(n: f64) -> Option<f32> {
    let converted = n as f32 as f64;

    if converted.to_bits() == n.to_bits() {
        Some(n as f32)
    } else {
        None
    }
}

/// Convert u64 to f64 losslessly.
pub const fn u64_to_f64_lossless(n: u64) -> Option<f64> {
    let converted = n as f64;
    let back_converted = converted as u64;

    if back_converted == n {
        Some(converted)
    } else {
        None
    }
}

/// Convert u64 to f32 losslessly.
pub const fn u64_to_f32_lossless(n: u64) -> Option<f32> {
    let converted = n as f32;
    let back_converted = converted as u64;

    if back_converted == n {
        Some(converted)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::f64_to_u64_no_sig_lossless;

    #[test]
    fn test_f64_to_u64_lossless() {
        assert_eq!(f64_to_u64_no_sig_lossless(0.0), Some(0));
        assert_eq!(f64_to_u64_no_sig_lossless(1.0), Some(1));
        assert_eq!(f64_to_u64_no_sig_lossless(42.0), Some(42));
        assert_eq!(f64_to_u64_no_sig_lossless(1.5), None);

        let two_pow_52_f = 2.0_f64.powi(52);
        let two_pow_52_i = 1u64 << 52;
        assert_eq!(f64_to_u64_no_sig_lossless(two_pow_52_f), Some(two_pow_52_i));
        assert_eq!(
            f64_to_u64_no_sig_lossless(two_pow_52_f + 1.0),
            Some(two_pow_52_i + 1)
        );
        assert_eq!(
            f64_to_u64_no_sig_lossless(two_pow_52_f + 2.0),
            Some(two_pow_52_i + 2)
        );

        let two_pow_53_f = 2.0_f64.powi(53);
        let two_pow_53_i = 1u64 << 53;
        assert_eq!(f64_to_u64_no_sig_lossless(two_pow_53_f), Some(two_pow_53_i));
        assert_eq!(
            f64_to_u64_no_sig_lossless(two_pow_53_f + 1.5),
            Some(two_pow_53_i + 2)
        ); // rounds up correctly?

        for i in 0..64 {
            let val = 1u64 << i;
            let val_f = val as f64;
            assert_eq!(f64_to_u64_no_sig_lossless(val_f), Some(val));
        }

        assert!(f64_to_u64_no_sig_lossless(f64::NAN).is_none());
        assert!(f64_to_u64_no_sig_lossless(f64::INFINITY).is_none());
        assert!(f64_to_u64_no_sig_lossless(f64::NEG_INFINITY).is_none());
        assert!(f64_to_u64_no_sig_lossless(2.0f64.powi(64)).is_none());
    }

    #[test]
    fn test_f64_to_u128_lossless() {
        use super::f64_to_u128_no_sig_lossless;

        assert_eq!(f64_to_u128_no_sig_lossless(0.0), Some(0));
        assert_eq!(f64_to_u128_no_sig_lossless(1.0), Some(1));
        assert_eq!(f64_to_u128_no_sig_lossless(42.0), Some(42));
        assert_eq!(f64_to_u128_no_sig_lossless(1.5), None);

        let two_pow_52_f = 2.0_f64.powi(52);
        let two_pow_52_i = 1u128 << 52;
        assert_eq!(
            f64_to_u128_no_sig_lossless(two_pow_52_f),
            Some(two_pow_52_i)
        );
        assert_eq!(
            f64_to_u128_no_sig_lossless(two_pow_52_f + 1.0),
            Some(two_pow_52_i + 1)
        );
        assert_eq!(
            f64_to_u128_no_sig_lossless(two_pow_52_f + 2.0),
            Some(two_pow_52_i + 2)
        );

        let two_pow_53_f = 2.0_f64.powi(53);
        let two_pow_53_i = 1u128 << 53;
        assert_eq!(
            f64_to_u128_no_sig_lossless(two_pow_53_f),
            Some(two_pow_53_i)
        );
        assert_eq!(
            f64_to_u128_no_sig_lossless(two_pow_53_f + 1.5),
            Some(two_pow_53_i + 2)
        ); // rounds up correctly?

        for i in 0..128 {
            let val = 1u128 << i;
            let val_f = val as f64;
            assert_eq!(f64_to_u128_no_sig_lossless(val_f), Some(val));
        }

        assert!(f64_to_u128_no_sig_lossless(f64::NAN).is_none());
        assert!(f64_to_u128_no_sig_lossless(f64::INFINITY).is_none());
        assert!(f64_to_u128_no_sig_lossless(f64::NEG_INFINITY).is_none());
        assert!(f64_to_u128_no_sig_lossless(2.0f64.powi(128)).is_none());
    }
}

/// Constants used in number operations and conversions.
pub mod consts {
    /// 2 to the power of 64 as f32.
    pub const TWO_POW_64_F32: f32 = 18446744073709551616.0;
    /// 2 to the power of 64 as f64.
    pub const TWO_POW_64_F64: f64 = 18446744073709551616.0;
}

/// Trait for number types that can be created from an infinite value.
pub trait FromInf {
    /// Constant representing negative infinity.
    const FROM_NEG_INF: Self;
    /// Constant representing positive infinity.
    const FROM_POS_INF: Self;
}

macro_rules! impl_from_inf {
    ($(($($t:ty),* $(,)?) = [$neg:ident, $pos:ident]),* $(,)?) => {
        $($(
            impl FromInf for $t {
                const FROM_NEG_INF: Self = Self::$neg;
                const FROM_POS_INF: Self = Self::$pos;
            }
        )*)*
    };
}

impl_from_inf! {
    (i8, i16, i32, i64, i128, isize) = [MIN, MAX],
    (u8, u16, u32, u64, u128, usize) = [MIN, MAX],
    (f32, f64) = [NEG_INFINITY, INFINITY],
}

/// Creates a value of type T from an infinite value.
pub const fn from_inf<T: FromInf>(negative: bool) -> T {
    if negative {
        T::FROM_NEG_INF
    } else {
        T::FROM_POS_INF
    }
}

/// Sign-extends a negative i65 stored as u64 to i128.
pub const fn neg_i65_to_i128(repr: u64) -> i128 {
    /// High 64 bits of i128.
    pub const I128_HIGH_64_BITS: i128 = 0xFFFFFFFF_FFFFFFFF_00000000_00000000u128 as i128;

    repr as i128 | I128_HIGH_64_BITS
}
