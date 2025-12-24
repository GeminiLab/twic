mod impls;
mod utils;

use utils::{
    consts::*, f64_to_f32_lossless, f64_to_u64_no_sig_lossless, f64_to_u128_no_sig_lossless,
    from_inf, neg_i65_to_i128, u64_to_f32_lossless, u64_to_f64_lossless,
};

/// Represents a Twic number, which can be an integer, float (excluding NaN and
/// Infinity), `nan`, and `[+/-]inf`.
///
/// This enum is not expected to be constructed directly by specifying its
/// variants but rather through conversions from Rust numeric types.
///
/// # Representation and Ranges
///
/// `Number` can represent:
/// - Integers from `-2^64` to `2^64 - 1`, i.e., the range of `i65` (if such a
///   type exists). Therefore all Rust integer types (except `i128` and `u128`)
///   can be safely converted to `Number` without loss of information.
/// - Floating-point numbers representable by `f64`, including special values
///   `NaN` and positive/negative infinity (though these are represented by
///   separate enum variants for clarity and convenience).
///
/// # Range Checking and Conversions
///
/// A number of methods and trait implementations are provided to check whether
/// a `Number` can fit into specific Rust numeric types, and to convert it
/// from/to those types.
///
/// - `fits_in_<type>` methods check if the `Number` can be represented as the
///   specified Rust numeric type without overflow. Integer `Number`s are
///   considered not to fit into floating-point types regardless of their value,
///   and vice versa.
/// - `From::from` allows conversion from Rust numeric types to `Number`s. These
///   conversions are always safe and lossless.
/// - `get_<type>` methods attempt to get the value of the `Number` as the
///   specified Rust numeric type, if the value and target type are both
///   integers or both floats, and the value fits within the target type's
///   range. Otherwise, `None` is returned.
/// - `as_<type>_exact` methods convert the `Number` to the specified Rust
///   numeric type, if the value can be represented losslessly in the target
///   type. These methods allow conversion between integer and float types when
///   possible. If the conversion is not possible without loss of information,
///   `None` is returned.
/// - `as_<type>` methods convert the `Number` to the specified Rust numeric
///   type, following Rust's [standard numeric casting `as` rules](
///   https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#r-expr.as.numeric)
///   and may result in loss of information.
#[derive(Clone, Copy)]
pub enum Number {
    /// Represents a positive integer, including zero, in the range of
    /// `[0, 2^64 - 1]`.
    PosInt(u64),
    /// Represents a negative integer in the range of `[-1, -2^64]`.
    ///
    /// The integer is stored with an offset of `-2^64`, i.e., `NegInt(0)`
    /// represents `-2^64`, `NegInt(u64::MAX)` represents `-1`.
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::NegInt(u64::MAX);
    /// assert_eq!(n.get_i64(), Some(-1));
    /// ```
    NegInt(u64),
    /// Represents a floating-point number (excluding NaN and Infinity).
    ///
    /// Constructing this variant directly with NaN or Infinity is not allowed,
    /// and can result in unexpected behaviors.
    Float(f64),
    /// Represents a Not-a-Number (`nan`) value.
    ///
    /// As a special case, `nan` is considered equal to `nan` in Twic.
    NaN,
    /// Represents positive or negative infinity.
    Inf {
        /// Indicates if the infinity is negative.
        negative: bool,
    },
}

/// Basic checks.
impl Number {
    /// Checks if the `Number` is an integer (either positive or negative).
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(42);
    /// assert!(n.is_integer());
    ///
    /// let m = Number::Float(3.14);
    /// assert!(!m.is_integer());
    /// ```
    pub const fn is_integer(&self) -> bool {
        matches!(self, Number::PosInt(_) | Number::NegInt(_))
    }

    /// Checks if the `Number` is a floating-point number, excluding NaN and
    /// Infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::Float(3.14);
    /// assert!(n.is_float());
    ///
    /// let m = Number::PosInt(42);
    /// assert!(!m.is_float());
    /// ```
    pub const fn is_float(&self) -> bool {
        matches!(self, Number::Float(_))
    }

    /// Checks if the `Number` is NaN.
    ///
    /// # Examples
    ////
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::NaN;
    /// assert!(n.is_nan());
    /// ```
    pub const fn is_nan(&self) -> bool {
        matches!(self, Number::NaN)
    }

    /// Checks if the `Number` is infinite (either positive or negative).
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let p = Number::Inf { negative: false };
    /// assert!(p.is_infinite());
    /// let n = Number::Inf { negative: true };
    /// assert!(n.is_infinite());
    /// ```
    pub const fn is_infinite(&self) -> bool {
        matches!(self, Number::Inf { .. })
    }

    /// Checks if the `Number` is positive infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let p = Number::Inf { negative: false };
    /// assert!(p.is_positive_infinite());
    /// let n = Number::Inf { negative: true };
    /// assert!(!n.is_positive_infinite());
    /// ```
    pub const fn is_positive_infinite(&self) -> bool {
        matches!(self, Number::Inf { negative: false })
    }

    /// Checks if the `Number` is negative infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let p = Number::Inf { negative: false };
    /// assert!(!p.is_negative_infinite());
    /// let n = Number::Inf { negative: true };
    /// assert!(n.is_negative_infinite());
    /// ```
    pub const fn is_negative_infinite(&self) -> bool {
        matches!(self, Number::Inf { negative: true })
    }

    /// Checks if the `Number` is positive (greater than zero).
    ///
    /// This includes positive integers, positive floats (excluding +0.0), and
    /// positive infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::PosInt(42).is_positive());
    /// assert!(!Number::PosInt(0).is_positive());
    /// assert!(!Number::NegInt(1).is_positive());
    /// assert!(Number::Float(3.14).is_positive());
    /// assert!(!Number::Float(0.0).is_positive());
    /// assert!(!Number::Float(-2.71).is_positive());
    /// assert!(Number::Inf { negative: false }.is_positive());
    /// assert!(!Number::Inf { negative: true }.is_positive());
    /// ```
    pub const fn is_positive(&self) -> bool {
        match self {
            Number::PosInt(n) => *n > 0,
            Number::NegInt(_) => false,
            Number::Float(n) => n.is_sign_positive() && *n != 0.0,
            Number::NaN => false,
            Number::Inf { negative } => !*negative,
        }
    }

    /// Checks if the `Number` is negative (less than zero).
    ///
    /// This includes negative integers, negative floats (excluding -0.0), and
    /// negative infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(!Number::PosInt(42).is_negative());
    /// assert!(!Number::PosInt(0).is_negative());
    /// assert!(Number::NegInt(1).is_negative());
    /// assert!(!Number::Float(3.14).is_negative());
    /// assert!(!Number::Float(0.0).is_negative());
    /// assert!(Number::Float(-2.71).is_negative());
    /// assert!(!Number::Inf { negative: false }.is_negative());
    /// assert!(Number::Inf { negative: true }.is_negative());
    /// ```
    pub const fn is_negative(&self) -> bool {
        match self {
            Number::PosInt(_) => false,
            Number::NegInt(_) => true,
            Number::Float(n) => n.is_sign_negative() && *n != 0.0,
            Number::NaN => false,
            Number::Inf { negative } => *negative,
        }
    }

    /// Checks if the `Number` is zero (either +0.0, -0.0, or 0).
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::PosInt(0).is_zero());
    /// assert!(Number::Float(0.0).is_zero());
    /// assert!(!Number::NegInt(1).is_zero());
    /// ```
    pub const fn is_zero(&self) -> bool {
        match self {
            Number::PosInt(n) => *n == 0,
            Number::NegInt(_) => false,
            Number::Float(n) => *n == 0.0,
            Number::NaN => false,
            Number::Inf { .. } => false,
        }
    }
}

/// Range checkers.
impl Number {
    /// Checks if the `Number` is an integer and can be represented as an i8
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(127).fits_in_i8());
    /// assert!(!Number::from(128).fits_in_i8());
    /// assert!(Number::from(-128).fits_in_i8());
    /// assert!(!Number::from(-129).fits_in_i8());
    /// ```
    pub const fn fits_in_i8(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= i8::MAX as u64,
            Number::NegInt(n) => *n >= i8::MIN as u64,
            _ => false,
        }
    }

    /// Checks if the `Number` is an integer and can be represented as an i16
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(32767).fits_in_i16());
    /// assert!(!Number::from(32768).fits_in_i16());
    /// assert!(Number::from(-32768).fits_in_i16());
    /// assert!(!Number::from(-32769).fits_in_i16());
    /// ```
    pub const fn fits_in_i16(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= i16::MAX as u64,
            Number::NegInt(n) => *n >= i16::MIN as u64,
            _ => false,
        }
    }

    /// Checks if the `Number` is an integer and can be represented as an i32
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(i32::MAX as i64).fits_in_i32());
    /// assert!(!Number::from(i32::MAX as i64 + 1).fits_in_i32());
    /// assert!(Number::from(i32::MIN as i64).fits_in_i32());
    /// assert!(!Number::from(i32::MIN as i64 - 1).fits_in_i32());
    /// ```
    pub const fn fits_in_i32(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= i32::MAX as u64,
            Number::NegInt(n) => *n >= i32::MIN as u64,
            _ => false,
        }
    }

    /// Checks if the `Number` is an integer and can be represented as an i64
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::PosInt(i64::MAX as u64).fits_in_i64());
    /// assert!(!Number::PosInt(i64::MAX as u64 + 1).fits_in_i64());
    /// assert!(Number::from(i64::MIN).fits_in_i64());
    /// assert!(!Number::NegInt(i64::MIN as u64 - 1).fits_in_i64());
    /// ```
    pub const fn fits_in_i64(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= i64::MAX as u64,
            Number::NegInt(n) => *n >= i64::MIN as u64,
            _ => false,
        }
    }

    /// Checks if the `Number` is an integer and can be represented as an isize
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(isize::MAX).fits_in_isize());
    /// assert!(!Number::PosInt(isize::MAX as u64 + 1).fits_in_isize());
    /// assert!(Number::from(isize::MIN).fits_in_isize());
    /// assert!(!Number::NegInt(isize::MIN as u64 - 1).fits_in_isize());
    /// ```
    pub const fn fits_in_isize(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= isize::MAX as u64,
            Number::NegInt(n) => *n >= isize::MIN as u64,
            _ => false,
        }
    }

    /// Checks if the `Number` is an integer and can be represented as an i128
    /// without overflow. This is always true for integer `Number`s.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(1u64).fits_in_i128());
    /// assert!(Number::from(-1i64).fits_in_i128());
    /// assert!(Number::from(u64::MAX).fits_in_i128());
    /// assert!(Number::NegInt(0).fits_in_i128());
    /// assert!(Number::NegInt(u64::MAX).fits_in_i128());
    /// ```
    pub const fn fits_in_i128(&self) -> bool {
        matches!(self, Number::PosInt(_) | Number::NegInt(_))
    }

    /// Checks if the `Number` is an integer and can be represented as a u8
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(255).fits_in_u8());
    /// assert!(!Number::from(256).fits_in_u8());
    /// assert!(Number::from(0).fits_in_u8());
    /// assert!(!Number::from(-1).fits_in_u8());
    /// ```
    pub const fn fits_in_u8(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= u8::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the `Number` is an integer and can be represented as a u16
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(65535).fits_in_u16());
    /// assert!(!Number::from(65536).fits_in_u16());
    /// assert!(Number::from(0).fits_in_u16());
    /// assert!(!Number::from(-1).fits_in_u16());
    /// ```
    pub const fn fits_in_u16(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= u16::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the `Number` is an integer and can be represented as a u32
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(4294967295u64).fits_in_u32());
    /// assert!(!Number::from(4294967296u64).fits_in_u32());
    /// assert!(Number::from(0).fits_in_u32());
    /// assert!(!Number::from(-1).fits_in_u32());
    /// ```
    pub const fn fits_in_u32(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= u32::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the `Number` is an integer and can be represented as a u64
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(u64::MAX).fits_in_u64());
    /// assert!(Number::from(0u64).fits_in_u64());
    /// assert!(!Number::from(-1i64).fits_in_u64());
    /// ```
    pub const fn fits_in_u64(&self) -> bool {
        matches!(self, Number::PosInt(_))
    }

    /// Checks if the `Number` is an integer and can be represented as a usize
    /// without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(usize::MAX).fits_in_usize());
    /// assert!(Number::from(0).fits_in_usize());
    /// assert!(!Number::from(-1).fits_in_usize());
    /// ```
    pub const fn fits_in_usize(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= usize::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the `Number` is an integer and can be represented as a u128
    /// without overflow. This is always true for positive integer `Number`s.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(u64::MAX).fits_in_u128());
    /// assert!(Number::from(0u64).fits_in_u128());
    /// assert!(!Number::from(-1i64).fits_in_u128());
    /// ```
    pub const fn fits_in_u128(&self) -> bool {
        matches!(self, Number::PosInt(_))
    }

    /// Checks if the `Number` is a float and can be represented as an f32
    /// without loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(1f64).fits_in_f32());
    /// assert!(!Number::from(1e100f64).fits_in_f32());
    /// ```
    pub const fn fits_in_f32(&self) -> bool {
        match self {
            Number::Float(n) => f64_to_f32_lossless(*n).is_some(),
            Number::NaN | Number::Inf { .. } => true,
            _ => false,
        }
    }

    /// Checks if the `Number` is a float and can be represented as an f64
    /// without loss of information. This is always true for floats, NaNs, and
    /// infinities.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert!(Number::from(1f64).fits_in_f64());
    /// assert!(Number::from(f64::NAN).fits_in_f64());
    /// assert!(Number::from(f64::INFINITY).fits_in_f64());
    /// assert!(!Number::from(1u64).fits_in_f64());
    /// ```
    pub const fn fits_in_f64(&self) -> bool {
        matches!(self, Number::Float(_) | Number::NaN | Number::Inf { .. })
    }
}

/// `get_<type>` methods.
impl Number {
    /// Gets the `Number` as an i8 if it is an integer and can be represented
    /// as an i8 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(127).get_i8(), Some(127));
    /// assert_eq!(Number::from(128).get_i8(), None);
    /// assert_eq!(Number::from(-128).get_i8(), Some(-128));
    /// assert_eq!(Number::from(-129).get_i8(), None);
    /// ```
    pub const fn get_i8(&self) -> Option<i8> {
        match self {
            Number::PosInt(n) if *n <= i8::MAX as u64 => Some(*n as i8),
            Number::NegInt(n) if *n >= i8::MIN as u64 => Some(*n as i8),
            _ => None,
        }
    }

    /// Gets the `Number` as an i16 if it is an integer and can be represented
    /// as an i16 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(32767).get_i16(), Some(32767));
    /// assert_eq!(Number::from(32768).get_i16(), None);
    /// assert_eq!(Number::from(-32768).get_i16(), Some(-32768));
    /// assert_eq!(Number::from(-32769).get_i16(), None);
    /// ```
    pub const fn get_i16(&self) -> Option<i16> {
        match self {
            Number::PosInt(n) if *n <= i16::MAX as u64 => Some(*n as i16),
            Number::NegInt(n) if *n >= i16::MIN as u64 => Some(*n as i16),
            _ => None,
        }
    }

    /// Gets the `Number` as an i32 if it is an integer and can be represented
    /// as an i32 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(i32::MAX as i64).get_i32(), Some(i32::MAX));
    /// assert_eq!(Number::from(i32::MAX as i64 + 1).get_i32(), None);
    /// assert_eq!(Number::from(i32::MIN as i64).get_i32(), Some(i32::MIN));
    /// assert_eq!(Number::from(i32::MIN as i64 - 1).get_i32(), None);
    /// ```
    pub const fn get_i32(&self) -> Option<i32> {
        match self {
            Number::PosInt(n) if *n <= i32::MAX as u64 => Some(*n as i32),
            Number::NegInt(n) if *n >= i32::MIN as u64 => Some(*n as i32),
            _ => None,
        }
    }

    /// Gets the `Number` as an i64 if it is an integer and can be represented
    /// as an i64 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(i64::MAX as u64).get_i64(), Some(i64::MAX));
    /// assert_eq!(Number::from(i64::MAX as u64 + 1).get_i64(), None);
    /// assert_eq!(Number::from(i64::MIN).get_i64(), Some(i64::MIN));
    /// assert_eq!(Number::NegInt(i64::MIN as u64 - 1).get_i64(), None);
    /// ```
    pub const fn get_i64(&self) -> Option<i64> {
        match self {
            Number::PosInt(n) if *n <= i64::MAX as u64 => Some(*n as i64),
            Number::NegInt(n) if *n >= i64::MIN as u64 => Some(*n as i64),
            _ => None,
        }
    }

    /// Gets the `Number` as an isize if it is an integer and can be represented
    /// as an isize without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(isize::MAX).get_isize(), Some(isize::MAX));
    /// assert_eq!(Number::from(isize::MAX as u64 + 1).get_isize(), None);
    /// assert_eq!(Number::from(isize::MIN).get_isize(), Some(isize::MIN));
    /// assert_eq!(Number::NegInt(isize::MIN as u64 - 1).get_isize(), None);
    /// ```
    pub const fn get_isize(&self) -> Option<isize> {
        match self {
            Number::PosInt(n) if *n <= isize::MAX as u64 => Some(*n as isize),
            Number::NegInt(n) if *n >= isize::MIN as u64 => Some(*n as isize),
            _ => None,
        }
    }

    /// Gets the `Number` as an i128 if it is an integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(1u64).get_i128(), Some(1));
    /// assert_eq!(Number::from(-1i64).get_i128(), Some(-1));
    /// assert_eq!(Number::from(u64::MAX).get_i128(), Some(u64::MAX as i128));
    /// assert_eq!(Number::NegInt(u64::MAX).get_i128(), Some(-1));
    /// assert_eq!(Number::NegInt(0).get_i128(), Some(-(u64::MAX as i128 + 1)));
    /// ```
    pub const fn get_i128(&self) -> Option<i128> {
        match self {
            Number::PosInt(n) => Some(*n as i128),
            Number::NegInt(n) => Some(neg_i65_to_i128(*n)),
            _ => None,
        }
    }

    /// Gets the `Number` as a u8 if it is an integer and can be represented as
    /// a u8 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(255).get_u8(), Some(255));
    /// assert_eq!(Number::from(256).get_u8(), None);
    /// assert_eq!(Number::from(0).get_u8(), Some(0));
    /// assert_eq!(Number::from(-1).get_u8(), None);
    /// ```
    pub const fn get_u8(&self) -> Option<u8> {
        match self {
            Number::PosInt(n) if *n <= u8::MAX as u64 => Some(*n as u8),
            _ => None,
        }
    }

    /// Gets the `Number` as a u16 if it is an integer and can be represented as
    /// a u16 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(65535).get_u16(), Some(65535));
    /// assert_eq!(Number::from(65536).get_u16(), None);
    /// assert_eq!(Number::from(0).get_u16(), Some(0));
    /// assert_eq!(Number::from(-1).get_u16(), None);
    /// ```
    pub const fn get_u16(&self) -> Option<u16> {
        match self {
            Number::PosInt(n) if *n <= u16::MAX as u64 => Some(*n as u16),
            _ => None,
        }
    }

    /// Gets the `Number` as a u32 if it is an integer and can be represented as
    /// a u32 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(4294967295u64).get_u32(), Some(4294967295));
    /// assert_eq!(Number::from(4294967296u64).get_u32(), None);
    /// assert_eq!(Number::from(0).get_u32(), Some(0));
    /// assert_eq!(Number::from(-1).get_u32(), None);
    /// ```
    pub const fn get_u32(&self) -> Option<u32> {
        match self {
            Number::PosInt(n) if *n <= u32::MAX as u64 => Some(*n as u32),
            _ => None,
        }
    }

    /// Gets the `Number` as a u64 if it is an integer and can be represented as
    /// a u64 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(u64::MAX).get_u64(), Some(u64::MAX));
    /// assert_eq!(Number::from(0u64).get_u64(), Some(0));
    /// assert_eq!(Number::from(-1i64).get_u64(), None);
    /// ```
    pub const fn get_u64(&self) -> Option<u64> {
        match self {
            Number::PosInt(n) => Some(*n),
            _ => None,
        }
    }

    /// Gets the `Number` as a usize if it is an integer and can be represented
    /// as a usize without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(1usize).get_usize(), Some(1));
    /// ```
    pub const fn get_usize(&self) -> Option<usize> {
        match self {
            Number::PosInt(n) if *n <= usize::MAX as u64 => Some(*n as usize),
            _ => None,
        }
    }

    /// Gets the `Number` as a u128 if it is a positive integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(u64::MAX).get_u128(), Some(u64::MAX as u128));
    /// assert_eq!(Number::from(1u64).get_u128(), Some(1));
    /// assert_eq!(Number::from(-1i64).get_u128(), None);
    /// ```
    pub const fn get_u128(&self) -> Option<u128> {
        match self {
            Number::PosInt(n) => Some(*n as u128),
            _ => None,
        }
    }

    /// Gets the `Number` as an f32 if it is a float, NaN, or infinity, and can
    /// be represented as an f32 without loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(3.14f32).get_f32(), Some(3.14));
    /// assert!(Number::from(f32::NAN).get_f32().unwrap().is_nan());
    /// assert_eq!(Number::from(f32::INFINITY).get_f32(), Some(f32::INFINITY));
    /// ```
    pub const fn get_f32(&self) -> Option<f32> {
        match self {
            Number::Float(n) => {
                if self.fits_in_f32() {
                    Some(*n as f32)
                } else {
                    None
                }
            }
            Number::NaN => Some(f32::NAN),
            Number::Inf { negative } => Some(from_inf(*negative)),
            _ => None,
        }
    }

    /// Gets the `Number` as an f64 if it is a float, NaN, or infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(3.14f64).get_f64(), Some(3.14));
    /// assert!(Number::from(f64::NAN).get_f64().unwrap().is_nan());
    /// assert_eq!(Number::from(f64::INFINITY).get_f64(), Some(f64::INFINITY));
    /// ```
    pub const fn get_f64(&self) -> Option<f64> {
        match self {
            Number::Float(n) => Some(*n),
            Number::NaN => Some(f64::NAN),
            Number::Inf { negative } => Some(from_inf(*negative)),
            _ => None,
        }
    }
}

/// `as_<type>_exact` methods.
impl Number {
    /// Converts the `Number` to an i8 if it is an integer or a float that
    /// can be represented as an i8 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(127).as_i8_exact(), Some(127));
    /// assert_eq!(Number::from(128).as_i8_exact(), None);
    /// assert_eq!(Number::from(-128).as_i8_exact(), Some(-128));
    /// assert_eq!(Number::from(-129).as_i8_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_i8_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_i8_exact(), None);
    /// assert_eq!(Number::from(127.0f64).as_i8_exact(), Some(127));
    /// assert_eq!(Number::from(128.0f64).as_i8_exact(), None);
    /// assert_eq!(Number::from(-127.0f64).as_i8_exact(), Some(-127));
    /// assert_eq!(Number::from(-128.0f64).as_i8_exact(), Some(-128));
    /// assert_eq!(Number::from(-129.0f64).as_i8_exact(), None);
    /// ```
    pub const fn as_i8_exact(&self) -> Option<i8> {
        match self {
            Number::PosInt(n) if *n <= i8::MAX as u64 => Some(*n as i8),
            Number::NegInt(n) if *n >= i8::MIN as u64 => Some(*n as i8),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() {
                    if int <= i8::MAX as u64 {
                        Some(int as i8)
                    } else {
                        None
                    }
                } else if int <= -(i8::MIN as i64) as u64 {
                    Some((int as i8).wrapping_neg())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to an i16 if it is an integer or a float that
    /// can be represented as an i16 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(32767).as_i16_exact(), Some(32767));
    /// assert_eq!(Number::from(32768).as_i16_exact(), None);
    /// assert_eq!(Number::from(-32768).as_i16_exact(), Some(-32768));
    /// assert_eq!(Number::from(-32769).as_i16_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_i16_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_i16_exact(), None);
    /// assert_eq!(Number::from(32767.0f64).as_i16_exact(), Some(32767));
    /// assert_eq!(Number::from(32768.0f64).as_i16_exact(), None);
    /// assert_eq!(Number::from(-32767.0f64).as_i16_exact(), Some(-32767));
    /// assert_eq!(Number::from(-32768.0f64).as_i16_exact(), Some(-32768));
    /// assert_eq!(Number::from(-32769.0f64).as_i16_exact(), None);
    /// ```
    pub const fn as_i16_exact(&self) -> Option<i16> {
        match self {
            Number::PosInt(n) if *n <= i16::MAX as u64 => Some(*n as i16),
            Number::NegInt(n) if *n >= i16::MIN as u64 => Some(*n as i16),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() {
                    if int <= i16::MAX as u64 {
                        Some(int as i16)
                    } else {
                        None
                    }
                } else if int <= -(i16::MIN as i64) as u64 {
                    Some((int as i16).wrapping_neg())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to an i32 if it is an integer or a float that
    /// can be represented as an i32 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(i32::MAX as i64).as_i32_exact(), Some(i32::MAX));
    /// assert_eq!(Number::from(i32::MAX as i64 + 1).as_i32_exact(), None);
    /// assert_eq!(Number::from(i32::MIN as i64).as_i32_exact(), Some(i32::MIN));
    /// assert_eq!(Number::from(i32::MIN as i64 - 1).as_i32_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_i32_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_i32_exact(), None);
    /// assert_eq!(Number::from(i32::MAX as f64).as_i32_exact(), Some(i32::MAX));
    /// assert_eq!(Number::from((i32::MAX as f64) + 1.0).as_i32_exact(), None);
    /// assert_eq!(Number::from(i32::MIN as f64).as_i32_exact(), Some(i32::MIN));
    /// assert_eq!(Number::from((i32::MIN as f64) - 1.0).as_i32_exact(), None);
    /// ```
    pub const fn as_i32_exact(&self) -> Option<i32> {
        match self {
            Number::PosInt(n) if *n <= i32::MAX as u64 => Some(*n as i32),
            Number::NegInt(n) if *n >= i32::MIN as u64 => Some(*n as i32),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() {
                    if int <= i32::MAX as u64 {
                        Some(int as i32)
                    } else {
                        None
                    }
                } else if int <= -(i32::MIN as i64) as u64 {
                    Some((int as i32).wrapping_neg())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to an i64 if it is an integer or a float that
    /// can be represented as an i64 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(i64::MAX as u64).as_i64_exact(), Some(i64::MAX));
    /// assert_eq!(Number::from(i64::MAX as u64 + 1).as_i64_exact(), None);
    /// assert_eq!(Number::from(i64::MIN).as_i64_exact(), Some(i64::MIN));
    /// assert_eq!(Number::NegInt(i64::MIN as u64 - 1).as_i64_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_i64_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_i64_exact(), None);
    ///
    /// let max_i64_in_f64 = (i64::MAX) & !( (1 << 11) - 1); // Clear least significant 11 bits
    /// assert_eq!(Number::from(max_i64_in_f64 as f64).as_i64_exact(), Some(i64::MAX - 2047));
    /// assert_eq!(Number::from(i64::MAX as f64).as_i64_exact(), None);
    /// assert_eq!(Number::from(i64::MIN as f64).as_i64_exact(), Some(i64::MIN));
    /// ```
    pub const fn as_i64_exact(&self) -> Option<i64> {
        match self {
            Number::PosInt(n) if *n <= i64::MAX as u64 => Some(*n as i64),
            Number::NegInt(n) if *n >= i64::MIN as u64 => Some(*n as i64),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() {
                    if int <= i64::MAX as u64 {
                        Some(int as i64)
                    } else {
                        None
                    }
                } else if int <= i64::MIN as u64 {
                    Some((int as i64).wrapping_neg())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to an isize if it is an integer or a float that
    /// can be represented as an isize without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(isize::MAX as u64).as_isize_exact(), Some(isize::MAX));
    /// assert_eq!(Number::from(isize::MAX as u64 + 1).as_isize_exact(), None);
    /// assert_eq!(Number::from(isize::MIN).as_isize_exact(), Some(isize::MIN));
    /// assert_eq!(Number::NegInt(isize::MIN as u64 - 1).as_isize_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_isize_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_isize_exact(), None);
    /// ```
    pub const fn as_isize_exact(&self) -> Option<isize> {
        match self {
            Number::PosInt(n) if *n <= isize::MAX as u64 => Some(*n as isize),
            Number::NegInt(n) if *n >= isize::MIN as u64 => Some(*n as isize),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() {
                    if int <= isize::MAX as u64 {
                        Some(int as isize)
                    } else {
                        None
                    }
                } else if int <= isize::MIN as u64 {
                    Some((int as isize).wrapping_neg())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to an i128 if it is an integer or a float that
    /// can be represented as an i128 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(1u64).as_i128_exact(), Some(1));
    /// assert_eq!(Number::from(-1i64).as_i128_exact(), Some(-1));
    /// assert_eq!(Number::from(u64::MAX).as_i128_exact(), Some(u64::MAX as i128));
    /// assert_eq!(Number::NegInt(u64::MAX).as_i128_exact(), Some(-1));
    /// assert_eq!(Number::NegInt(0).as_i128_exact(), Some(-(u64::MAX as i128 + 1)));
    /// assert_eq!(Number::from(3.0f64).as_i128_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_i128_exact(), None);
    ///
    /// let max_i64_in_f64 = (i64::MAX) & !( (1 << 11) - 1); // Clear least significant 11 bits
    /// assert_eq!(Number::from((max_i64_in_f64) as f64).as_i128_exact(), Some(max_i64_in_f64 as i128));
    ///
    /// let max_i128_in_f64 = (i128::MAX) & !( (1 << 75) - 1); // Clear least significant 75 bits
    /// assert_eq!(Number::from((max_i128_in_f64) as f64).as_i128_exact(), Some(max_i128_in_f64));
    /// assert_eq!(Number::from(i128::MAX as f64).as_i128_exact(), None);
    /// assert_eq!(Number::from(i128::MIN as f64).as_i128_exact(), Some(i128::MIN));
    /// ```
    pub const fn as_i128_exact(&self) -> Option<i128> {
        match self {
            Number::PosInt(n) => Some(*n as i128),
            Number::NegInt(n) => Some(neg_i65_to_i128(*n)),
            Number::Float(n) => {
                let int = match f64_to_u128_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() && int <= i128::MAX as u128 {
                    Some(int as i128)
                } else if n.is_sign_negative() && int <= i128::MIN as u128 {
                    Some((int as i128).wrapping_neg())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to a u8 if it is a integer or a float that can be
    /// represented as a u8 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(255).as_u8_exact(), Some(255));
    /// assert_eq!(Number::from(256).as_u8_exact(), None);
    /// assert_eq!(Number::from(0).as_u8_exact(), Some(0));
    /// assert_eq!(Number::from(-1).as_u8_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_u8_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_u8_exact(), None);
    /// assert_eq!(Number::from(255.0f64).as_u8_exact(), Some(255));
    /// assert_eq!(Number::from(256.0f64).as_u8_exact(), None);
    /// assert_eq!(Number::from(0.0f64).as_u8_exact(), Some(0));
    /// assert_eq!(Number::from(-0.0f64).as_u8_exact(), Some(0));
    /// assert_eq!(Number::from(-1.0f64).as_u8_exact(), None);
    /// ```
    pub const fn as_u8_exact(&self) -> Option<u8> {
        match self {
            Number::PosInt(n) if *n <= u8::MAX as u64 => Some(*n as u8),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() && int <= u8::MAX as u64 {
                    Some(int as u8)
                } else if int == 0 {
                    // -0.0 case
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to a u16 if it is a integer or a float that can be
    /// represented as a u16 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(65535).as_u16_exact(), Some(65535));
    /// assert_eq!(Number::from(65536).as_u16_exact(), None);
    /// assert_eq!(Number::from(0).as_u16_exact(), Some(0));
    /// assert_eq!(Number::from(-1).as_u16_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_u16_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_u16_exact(), None);
    /// assert_eq!(Number::from(65535.0f64).as_u16_exact(), Some(65535));
    /// assert_eq!(Number::from(65536.0f64).as_u16_exact(), None);
    /// assert_eq!(Number::from(0.0f64).as_u16_exact(), Some(0));
    /// assert_eq!(Number::from(-0.0f64).as_u16_exact(), Some(0));
    /// assert_eq!(Number::from(-1.0f64).as_u16_exact(), None);
    /// ```
    pub const fn as_u16_exact(&self) -> Option<u16> {
        match self {
            Number::PosInt(n) if *n <= u16::MAX as u64 => Some(*n as u16),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() && int <= u16::MAX as u64 {
                    Some(int as u16)
                } else if int == 0 {
                    // -0.0 case
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to a u32 if it is a integer or a float that can be
    /// represented as a u32 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(4294967295u64).as_u32_exact(), Some(4294967295));
    /// assert_eq!(Number::from(4294967296u64).as_u32_exact(), None);
    /// assert_eq!(Number::from(0).as_u32_exact(), Some(0));
    /// assert_eq!(Number::from(-1).as_u32_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_u32_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_u32_exact(), None);
    /// assert_eq!(Number::from(4294967295.0f64).as_u32_exact(), Some(4294967295));
    /// assert_eq!(Number::from(4294967296.0f64).as_u32_exact(), None);
    /// assert_eq!(Number::from(0.0f64).as_u32_exact(), Some(0));
    /// assert_eq!(Number::from(-0.0f64).as_u32_exact(), Some(0));
    /// assert_eq!(Number::from(-1.0f64).as_u32_exact(), None);
    /// ```
    pub const fn as_u32_exact(&self) -> Option<u32> {
        match self {
            Number::PosInt(n) if *n <= u32::MAX as u64 => Some(*n as u32),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() && int <= u32::MAX as u64 {
                    Some(int as u32)
                } else if int == 0 {
                    // -0.0 case
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to a u64 if it is a integer or a float that can be
    /// represented as a u64 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(u64::MAX).as_u64_exact(), Some(u64::MAX));
    /// assert_eq!(Number::from(0u64).as_u64_exact(), Some(0));
    /// assert_eq!(Number::from(-1i64).as_u64_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_u64_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_u64_exact(), None);
    ///
    /// let max_u64_in_f64 = u64::MAX & !( (1 << 11) - 1); // Clear least significant 11 bits
    /// assert_eq!(Number::from(max_u64_in_f64 as f64).as_u64_exact(), Some(max_u64_in_f64));
    /// assert_eq!(Number::from(u64::MAX as f64).as_u64_exact(), None);
    /// ```
    pub const fn as_u64_exact(&self) -> Option<u64> {
        match self {
            Number::PosInt(n) => Some(*n),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() {
                    Some(int)
                } else if int == 0 {
                    // -0.0 case
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to a usize if it is a integer or a float that can
    /// be represented as a usize without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(1usize).as_usize_exact(), Some(1));
    /// assert_eq!(Number::from(3.0f64).as_usize_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_usize_exact(), None);
    /// assert_eq!(Number::from(0.0f64).as_usize_exact(), Some(0));
    /// assert_eq!(Number::from(-0.0f64).as_usize_exact(), Some(0));
    /// ```
    pub const fn as_usize_exact(&self) -> Option<usize> {
        match self {
            Number::PosInt(n) if *n <= usize::MAX as u64 => Some(*n as usize),
            Number::Float(n) => {
                let int = match f64_to_u64_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() && int <= usize::MAX as u64 {
                    Some(int as usize)
                } else if int == 0 {
                    // -0.0 case
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to a u128 if it is a integer or a float that can
    /// be represented as a u128 without overflow or loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(u64::MAX).as_u128_exact(), Some(u64::MAX as u128));
    /// assert_eq!(Number::from(1u64).as_u128_exact(), Some(1));
    /// assert_eq!(Number::from(-1i64).as_u128_exact(), None);
    /// assert_eq!(Number::from(3.0f64).as_u128_exact(), Some(3));
    /// assert_eq!(Number::from(3.14f64).as_u128_exact(), None);
    ///
    /// let max_u64_in_f64 = u64::MAX & !( (1 << 11) - 1); // Clear least significant 11 bits
    /// assert_eq!(Number::from(max_u64_in_f64 as f64).as_u128_exact(), Some(max_u64_in_f64 as u128));
    /// assert_eq!(Number::from(u64::MAX as f64).as_u128_exact(), Some(u64::MAX as u128 + 1));
    ///
    /// let max_u128_in_f64 = (u128::MAX) & !( (1 << 75) - 1); // Clear least significant 75 bits
    /// assert_eq!(Number::from(max_u128_in_f64 as f64).as_u128_exact(), Some(max_u128_in_f64));
    /// assert_eq!(Number::from(u128::MAX as f64).as_u128_exact(), None);
    /// ```
    pub const fn as_u128_exact(&self) -> Option<u128> {
        match self {
            Number::PosInt(n) => Some(*n as u128),
            Number::Float(n) => {
                let int = match f64_to_u128_no_sig_lossless(*n) {
                    Some(v) => v,
                    None => return None,
                };

                if n.is_sign_positive() {
                    Some(int)
                } else if int == 0 {
                    // -0.0 case
                    Some(0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Converts the `Number` to an f32 if it can be represented as an f32
    /// without loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(16777216u64).as_f32_exact(), Some(16777216.0));
    /// assert_eq!(Number::from(16777217u64).as_f32_exact(), None);
    /// assert_eq!(Number::from(-16777216i64).as_f32_exact(), Some(-16777216.0));
    /// assert_eq!(Number::from(-16777217i64).as_f32_exact(), None);
    /// assert_eq!(Number::from(3.14f32 as f64).as_f32_exact(), Some(3.14f32));
    /// assert_eq!(Number::from(3.14159265f64).as_f32_exact(), None);
    /// assert!(Number::from(f64::NAN).as_f32_exact().unwrap().is_nan());
    /// assert_eq!(Number::from(f64::INFINITY).as_f32_exact(), Some(f32::INFINITY));
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_f32_exact(), Some(f32::NEG_INFINITY));
    /// ```
    pub const fn as_f32_exact(&self) -> Option<f32> {
        match self {
            Number::PosInt(n) => u64_to_f32_lossless(*n),
            Number::NegInt(0) => Some(-TWO_POW_64_F32),
            Number::NegInt(n) => match u64_to_f32_lossless(u64::MAX - *n + 1) {
                Some(v) => Some(-v),
                None => None,
            },
            Number::Float(n) => f64_to_f32_lossless(*n),
            Number::NaN => Some(f32::NAN),
            Number::Inf { negative } => Some(from_inf(*negative)),
        }
    }

    /// Converts the `Number` to an f64 if it can be represented as an f64
    /// without loss of information.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(u64::MAX).as_f64_exact(), Some(u64::MAX as f64));
    /// assert_eq!(Number::from(-1i64).as_f64_exact(), Some(-1.0));
    /// assert_eq!(Number::from(3.14f64).as_f64_exact(), Some(3.14));
    /// assert!(Number::from(f64::NAN).as_f64_exact().unwrap().is_nan());
    /// assert_eq!(Number::from(f64::INFINITY).as_f64_exact(), Some(f64::INFINITY));
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_f64_exact(), Some(f64::NEG_INFINITY));
    /// ```
    pub const fn as_f64_exact(&self) -> Option<f64> {
        match self {
            Number::PosInt(n) => u64_to_f64_lossless(*n),
            Number::NegInt(0) => Some(-TWO_POW_64_F64),
            Number::NegInt(n) => match u64_to_f64_lossless(u64::MAX - *n + 1) {
                Some(v) => Some(-v),
                None => None,
            },
            Number::Float(n) => Some(*n),
            Number::NaN => Some(f64::NAN),
            Number::Inf { negative } => Some(from_inf(*negative)),
        }
    }
}

/// `as_<type>` methods.
impl Number {
    /// Converts the `Number` to an i8 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(300u16).as_i8(), 44);
    /// assert_eq!(Number::from(-1i8).as_i8(), -1);
    /// assert_eq!(Number::from(128u8).as_i8(), -128);
    /// assert_eq!(Number::from(3.14f64).as_i8(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_i8(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_i8(), i8::MAX);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_i8(), i8::MIN);
    /// ```
    pub const fn as_i8(&self) -> i8 {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n as i8,
            Number::Float(n) => *n as i8,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to an i16 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(70000u32).as_i16(), 4464);
    /// assert_eq!(Number::from(-1i16).as_i16(), -1);
    /// assert_eq!(Number::from(32768u32).as_i16(), -32768);
    /// assert_eq!(Number::from(3.14f64).as_i16(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_i16(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_i16(), i16::MAX);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_i16(), i16::MIN);
    /// ```
    pub const fn as_i16(&self) -> i16 {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n as i16,
            Number::Float(n) => *n as i16,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to an i32 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(5_000_000_000u64).as_i32(), 705032704);
    /// assert_eq!(Number::from(-1i32).as_i32(), -1);
    /// assert_eq!(Number::from(2147483648u64).as_i32(), -2147483648);
    /// assert_eq!(Number::from(3.14f64).as_i32(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_i32(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_i32(), i32::MAX);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_i32(), i32::MIN);
    /// ```
    pub const fn as_i32(&self) -> i32 {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n as i32,
            Number::Float(n) => *n as i32,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to an i64 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(-1i64).as_i64(), -1);
    /// assert_eq!(Number::from(u64::MAX).as_i64(), -1);
    /// assert_eq!(Number::from(3.14f64).as_i64(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_i64(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_i64(), i64::MAX);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_i64(), i64::MIN);
    /// ```
    pub const fn as_i64(&self) -> i64 {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n as i64,
            Number::Float(n) => *n as i64,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to an isize following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(-1isize).as_isize(), -1);
    /// assert_eq!(Number::from(u64::MAX).as_isize(), -1);
    /// assert_eq!(Number::from(3.14f64).as_isize(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_isize(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_isize(), isize::MAX);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_isize(), isize::MIN);
    /// ```
    pub const fn as_isize(&self) -> isize {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n as isize,
            Number::Float(n) => *n as isize,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to an i128 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(-1i64).as_i128(), -1);
    /// assert_eq!(Number::from(u64::MAX).as_i128(), u64::MAX as i128);
    /// assert_eq!(Number::from(3.14f64).as_i128(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_i128(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_i128(), i128::MAX);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_i128(), i128::MIN);
    /// assert_eq!(Number::NegInt(0).as_i128(), -(u64::MAX as i128 + 1));
    /// ```
    pub const fn as_i128(&self) -> i128 {
        match self {
            Number::PosInt(n) => *n as i128,
            Number::NegInt(n) => neg_i65_to_i128(*n),
            Number::Float(n) => *n as i128,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to a u8 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(300u16).as_u8(), 44);
    /// assert_eq!(Number::from(-1i8).as_u8(), 255);
    /// assert_eq!(Number::from(3.14f64).as_u8(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_u8(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_u8(), 255);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_u8(), 0);
    /// ```
    pub const fn as_u8(&self) -> u8 {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n as u8,
            Number::Float(n) => *n as u8,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to a u16 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(70000u32).as_u16(), 4464);
    /// assert_eq!(Number::from(-1i16).as_u16(), 65535);
    /// assert_eq!(Number::from(3.14f64).as_u16(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_u16(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_u16(), 65535);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_u16(), 0);
    /// ```
    pub const fn as_u16(&self) -> u16 {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n as u16,
            Number::Float(n) => *n as u16,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to a u32 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(5_000_000_000u64).as_u32(), 705032704);
    /// assert_eq!(Number::from(-1i32).as_u32(), 4294967295);
    /// assert_eq!(Number::from(3.14f64).as_u32(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_u32(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_u32(), 4294967295);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_u32(), 0);
    /// ```
    pub const fn as_u32(&self) -> u32 {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n as u32,
            Number::Float(n) => *n as u32,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to a u64 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(-1i64).as_u64(), 18446744073709551615);
    /// assert_eq!(Number::from(3.14f64).as_u64(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_u64(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_u64(), 18446744073709551615);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_u64(), 0);
    /// ```
    pub const fn as_u64(&self) -> u64 {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n,
            Number::Float(n) => *n as u64,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to a usize following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(-1isize).as_usize(), usize::MAX);
    /// assert_eq!(Number::from(3.14f64).as_usize(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_usize(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_usize(), usize::MAX);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_usize(), usize::MIN);
    /// ```
    pub const fn as_usize(&self) -> usize {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n as usize,
            Number::Float(n) => *n as usize,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to a u128 following Rust's `as` casting rules.
    ///
    /// # Examples
    ////
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(-1i64).as_u128(), u128::MAX);
    /// assert_eq!(Number::from(u64::MAX).as_u128(), u64::MAX as u128);
    /// assert_eq!(Number::from(3.14f64).as_u128(), 3);
    /// assert_eq!(Number::from(f64::NAN).as_u128(), 0);
    /// assert_eq!(Number::from(f64::INFINITY).as_u128(), u128::MAX);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_u128(), u128::MIN);
    /// assert_eq!(Number::NegInt(0).as_u128(), 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000u128);
    /// ```
    pub const fn as_u128(&self) -> u128 {
        match self {
            Number::PosInt(n) => *n as u128,
            Number::NegInt(n) => neg_i65_to_i128(*n) as u128,
            Number::Float(n) => *n as u128,
            Number::NaN => 0,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to an f32 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(42u64).as_f32(), 42.0);
    /// assert_eq!(Number::from(-1i64).as_f32(), -1.0);
    /// assert_eq!(Number::from(3.14f32).as_f32(), 3.14f32);
    /// assert!(Number::from(f64::NAN).as_f32().is_nan());
    /// assert_eq!(Number::from(f64::INFINITY).as_f32(), f32::INFINITY);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_f32(), f32::NEG_INFINITY);
    /// ```
    pub const fn as_f32(&self) -> f32 {
        match self {
            Number::PosInt(n) => *n as f32,
            Number::NegInt(n) => neg_i65_to_i128(*n) as f32,
            Number::Float(n) => *n as f32,
            Number::NaN => f32::NAN,
            Number::Inf { negative } => from_inf(*negative),
        }
    }

    /// Converts the `Number` to an f64 following Rust's `as` casting rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// assert_eq!(Number::from(42u64).as_f64(), 42.0);
    /// assert_eq!(Number::from(-1i64).as_f64(), -1.0);
    /// assert_eq!(Number::from(3.14f64).as_f64(), 3.14f64);
    /// assert!(Number::from(f64::NAN).as_f64().is_nan());
    /// assert_eq!(Number::from(f64::INFINITY).as_f64(), f64::INFINITY);
    /// assert_eq!(Number::from(f64::NEG_INFINITY).as_f64(), f64::NEG_INFINITY);
    /// ```
    pub const fn as_f64(&self) -> f64 {
        match self {
            Number::PosInt(n) => *n as f64,
            Number::NegInt(n) => neg_i65_to_i128(*n) as f64,
            Number::Float(n) => *n,
            Number::NaN => f64::NAN,
            Number::Inf { negative } => from_inf(*negative),
        }
    }
}
