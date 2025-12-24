use core::fmt;
use core::hash::Hash;

use super::Number;

impl fmt::Debug for Number {
    /// Formats the Twic number for debugging purposes.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(format!("{:?}", Number::PosInt(42)), "Integer(42)");
    /// assert_eq!(format!("{:?}", Number::NegInt(u64::MAX)), "Integer(-1)");
    /// assert_eq!(format!("{:?}", Number::NegInt(0)), "Integer(-18446744073709551616)");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::PosInt(n) => write!(f, "Integer({})", n),
            Number::NegInt(0) => write!(f, "Integer(-18446744073709551616)"),
            Number::NegInt(n) => write!(f, "Integer(-{})", n.wrapping_neg()),
            Number::Float(n) => write!(f, "Float({})", n),
            Number::NaN => write!(f, "NaN"),
            Number::Inf { negative } => {
                if *negative {
                    write!(f, "-Inf")
                } else {
                    write!(f, "+Inf")
                }
            }
        }
    }
}

impl PartialEq for Number {
    /// Compares two Twic numbers for equality.
    ///
    /// As a special case, NaN is considered equal to NaN in Twic.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Number::PosInt(a), Number::PosInt(b)) => a == b,
            (Number::NegInt(a), Number::NegInt(b)) => a == b,
            (Number::Float(a), Number::Float(b)) => a == b,
            (Number::NaN, Number::NaN) => true,
            (Number::Inf { negative: a }, Number::Inf { negative: b }) => a == b,
            _ => false,
        }
    }
}

impl Hash for Number {
    /// Hashes the Twic number.
    ///
    /// NaN is hashed to a constant value to ensure that all NaN values hash
    /// identically.
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => {
                n.hash(state);
            }
            Number::Float(n) => {
                // thanks to serde_json for this idea, we hash +0.0 and -0.0 to
                // the same value
                if *n == 0.0f64 {
                    0.0f64.to_bits().hash(state);
                } else {
                    n.to_bits().hash(state);
                }
            }
            Number::NaN => {
                f64::NAN.to_bits().hash(state);
            }
            Number::Inf { negative } => {
                (if *negative {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                })
                .to_bits()
                .hash(state);
            }
        }
    }
}

impl From<f32> for Number {
    /// Converts a f32 to a Twic number.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(3.14f32).is_float());
    /// assert!(Number::from(f32::NAN).is_nan());
    /// ```
    fn from(value: f32) -> Self {
        let value_f64 = value as f64;
        if value_f64.is_nan() {
            Number::NaN
        } else if value_f64.is_infinite() {
            Number::Inf {
                negative: value_f64.is_sign_negative(),
            }
        } else {
            Number::Float(value_f64)
        }
    }
}

impl From<f64> for Number {
    /// Converts a f64 to a Twic number.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(3.14f64).is_float());
    /// assert!(Number::from(f64::NAN).is_nan());
    /// ```
    fn from(value: f64) -> Self {
        if value.is_nan() {
            Number::NaN
        } else if value.is_infinite() {
            Number::Inf {
                negative: value.is_sign_negative(),
            }
        } else {
            Number::Float(value)
        }
    }
}

macro_rules! impl_from_i_number {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Number {
                /// Converts a signed integer to a Twic number.
                fn from(value: $t) -> Self {
                    if value >= 0 {
                        Number::PosInt(value as u64)
                    } else {
                        Number::NegInt(value as u64)
                    }
                }
            }
        )*
    }
}

macro_rules! impl_from_u_number {
    ($($t:ty),*) => {
        $(
            impl From<$t> for Number {
                /// Converts an unsigned integer to a Twic number.
                fn from(value: $t) -> Self {
                    Number::PosInt(value as u64)
                }
            }
        )*
    }
}

impl_from_i_number!(i8, i16, i32, i64, isize);
impl_from_u_number!(u8, u16, u32, u64, usize);

macro_rules! impl_partial_eq_number {
    ($($t:ty => $method:ident),* $(,)?) => {
        $(
            impl PartialEq<$t> for Number {
                /// Compares a Twic number with a primitive number for equality.
                fn eq(&self, other: &$t) -> bool {
                    self.$method() == Some(*other)
                }
            }

            impl PartialEq<Number> for $t {
                /// Compares a primitive number with a Twic number for equality.
                fn eq(&self, other: &Number) -> bool {
                    other.$method() == Some(*self)
                }
            }
        )*
    }
}

impl_partial_eq_number! {
    i8 => as_i8_exact,
    i16 => as_i16_exact,
    i32 => as_i32_exact,
    i64 => as_i64_exact,
    isize => as_isize_exact,
    i128 => as_i128_exact,
    u8 => as_u8_exact,
    u16 => as_u16_exact,
    u32 => as_u32_exact,
    u64 => as_u64_exact,
    usize => as_usize_exact,
    u128 => as_u128_exact,
    f32 => as_f32_exact,
    f64 => as_f64_exact,
}
