use core::fmt;
use core::hash::Hash;

/// Represents a Twic number, which can be an integer, float (excluding NaN and
/// Infinity), `nan`, and `[+/-]inf`.
///
/// This enum is not expected to be constructed directly by specifying its
/// variants but rather through conversions from Rust numeric types.
#[derive(Clone, Copy)]
pub enum Number {
    /// Represents a positive integer, including zero, in the range of
    /// `[0, 2^64 - 1]`.
    PosInt(u64),
    /// Represents a negative integer in the range of `[-1, -2^64]`.
    ///
    /// The integer is stored with an offset of 1, so that `-1` is represented
    /// as `0`, `-2` as `1`, and so on.
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

impl Number {
    /// Checks if the Twic number is an integer (either positive or negative).
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

    /// Checks if the Twic number is a floating-point number, excluding NaN and
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

    /// Checks if the Twic number is NaN.
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

    /// Checks if the Twic number is infinite (either positive or negative).
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

    /// Checks if the Twic number is positive infinity.
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

    /// Checks if the Twic number is negative infinity.
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

    /// Checks if the Twic number is positive (greater than zero).
    ///
    /// This includes positive integers, positive floats (excluding +0.0), and
    /// positive infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let p_int = Number::PosInt(42);
    /// assert!(p_int.is_positive());
    /// let n_int = Number::NegInt(1);
    /// assert!(!n_int.is_positive());
    /// let p_float = Number::Float(3.14);
    /// assert!(p_float.is_positive());
    /// let zero_float = Number::Float(0.0);
    /// assert!(!zero_float.is_positive());
    /// let n_float = Number::Float(-2.71);
    /// assert!(!n_float.is_positive());
    /// let p_inf = Number::Inf { negative: false };
    /// assert!(p_inf.is_positive());
    /// let n_inf = Number::Inf { negative: true };
    /// assert!(!n_inf.is_positive());
    /// ```
    pub const fn is_positive(&self) -> bool {
        match self {
            Number::PosInt(_) => true,
            Number::NegInt(_) => false,
            Number::Float(n) => n.is_sign_positive() && *n != 0.0,
            Number::NaN => false,
            Number::Inf { negative } => !*negative,
        }
    }

    /// Checks if the Twic number is negative (less than zero).
    ///
    /// This includes negative integers, negative floats (excluding -0.0), and
    /// negative infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let p_int = Number::PosInt(42);
    /// assert!(!p_int.is_negative());
    /// let n_int = Number::NegInt(1);
    /// assert!(n_int.is_negative());
    /// let p_float = Number::Float(3.14);
    /// assert!(!p_float.is_negative());
    /// let zero_float = Number::Float(0.0);
    /// assert!(!zero_float.is_negative());
    /// let n_float = Number::Float(-2.71);
    /// assert!(n_float.is_negative());
    /// let p_inf = Number::Inf { negative: false };
    /// assert!(!p_inf.is_negative());
    /// let n_inf = Number::Inf { negative: true };
    /// assert!(n_inf.is_negative());
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

    /// Checks if the Twic number is zero (either +0.0 or 0).
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let int_zero = Number::PosInt(0);
    /// assert!(int_zero.is_zero());
    /// let float_zero = Number::Float(0.0);
    /// assert!(float_zero.is_zero());
    /// let neg_int = Number::NegInt(1);
    /// assert!(!neg_int.is_zero());
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

    /// Checks if the Twic number can be represented as an i8 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(100);
    /// assert!(n.fits_in_i8());
    /// let m = Number::PosInt(200);
    /// assert!(!m.fits_in_i8());
    /// ```
    pub const fn fits_in_i8(&self) -> bool {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n <= i8::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the Twic number can be represented as an i16 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(20000);
    /// assert!(n.fits_in_i16());
    /// let m = Number::PosInt(40000);
    /// assert!(!m.fits_in_i16());
    /// ```
    pub const fn fits_in_i16(&self) -> bool {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n <= i16::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the Twic number can be represented as an i32 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(2_000_000_000);
    /// assert!(n.fits_in_i32());
    /// let m = Number::PosInt(4_000_000_000);
    /// assert!(!m.fits_in_i32());
    /// ```
    pub const fn fits_in_i32(&self) -> bool {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n <= i32::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the Twic number can be represented as an i64 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(9_000_000_000_000_000_000);
    /// assert!(n.fits_in_i64());
    /// let m = Number::PosInt(10_000_000_000_000_000_000);
    /// assert!(!m.fits_in_i64());
    /// ```
    pub const fn fits_in_i64(&self) -> bool {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n <= i64::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the Twic number can be represented as an isize without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(1);
    /// assert!(n.fits_in_isize());
    /// let m = Number::PosInt(u64::MAX);
    /// assert!(!m.fits_in_isize());
    /// ```
    pub const fn fits_in_isize(&self) -> bool {
        match self {
            Number::PosInt(n) | Number::NegInt(n) => *n <= isize::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the Twic number can be represented as a u8 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(200);
    /// assert!(n.fits_in_u8());
    /// let m = Number::PosInt(300);
    /// assert!(!m.fits_in_u8());
    /// ```
    pub const fn fits_in_u8(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= u8::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the Twic number can be represented as a u16 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(60000);
    /// assert!(n.fits_in_u16());
    /// let m = Number::PosInt(70000);
    /// assert!(!m.fits_in_u16());
    /// ```
    pub const fn fits_in_u16(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= u16::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the Twic number can be represented as a u32 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(3_000_000_000);
    /// assert!(n.fits_in_u32());
    /// let m = Number::PosInt(5_000_000_000);
    /// assert!(!m.fits_in_u32());
    /// ```
    pub const fn fits_in_u32(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= u32::MAX as u64,
            _ => false,
        }
    }

    /// Checks if the Twic number can be represented as a u64 without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(18_000_000_000_000_000_000);
    /// assert!(n.fits_in_u64());
    /// let m = Number::NegInt(0);
    /// assert!(!m.fits_in_u64());
    /// ```
    pub const fn fits_in_u64(&self) -> bool {
        matches!(self, Number::PosInt(_))
    }

    /// Checks if the Twic number can be represented as a usize without overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n = Number::PosInt(1);
    /// assert!(n.fits_in_usize());
    /// let m = Number::NegInt(0);
    /// assert!(!m.fits_in_usize());
    /// ```
    pub const fn fits_in_usize(&self) -> bool {
        match self {
            Number::PosInt(n) => *n <= usize::MAX as u64,
            _ => false,
        }
    }

    /// Converts the Twic number to an i8 if it fits within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 100i8.into();
    /// assert_eq!(n.as_i8(), Some(100));
    /// let m: Number = (-100i8).into();
    /// assert_eq!(m.as_i8(), Some(-100));
    /// ```
    pub const fn as_i8(&self) -> Option<i8> {
        match self {
            Number::PosInt(n) if *n <= i8::MAX as u64 => Some(*n as i8),
            Number::NegInt(n) if *n <= i8::MAX as u64 => Some(-(*n as i8) - 1),
            _ => None,
        }
    }

    /// Converts the Twic number to an i16 if it fits within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 20000i16.into();
    /// assert_eq!(n.as_i16(), Some(20000));
    /// let m: Number = (-20000i16).into();
    /// assert_eq!(m.as_i16(), Some(-20000));
    /// ```
    pub const fn as_i16(&self) -> Option<i16> {
        match self {
            Number::PosInt(n) if *n <= i16::MAX as u64 => Some(*n as i16),
            Number::NegInt(n) if *n <= i16::MAX as u64 => Some(-(*n as i16) - 1),
            _ => None,
        }
    }

    /// Converts the Twic number to an i32 if it fits within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 2_000_000_000i32.into();
    /// assert_eq!(n.as_i32(), Some(2_000_000_000));
    /// let m: Number = (-2_000_000_000i32).into();
    /// assert_eq!(m.as_i32(), Some(-2_000_000_000));
    /// ```
    pub const fn as_i32(&self) -> Option<i32> {
        match self {
            Number::PosInt(n) if *n <= i32::MAX as u64 => Some(*n as i32),
            Number::NegInt(n) if *n <= i32::MAX as u64 => Some(-(*n as i32) - 1),
            _ => None,
        }
    }

    /// Converts the Twic number to an i64 if it fits within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 9_000_000_000_000_000_000i64.into();
    /// assert_eq!(n.as_i64(), Some(9_000_000_000_000_000_000));
    /// let m: Number = (-9_000_000_000_000_000_000i64).into();
    /// assert_eq!(m.as_i64(), Some(-9_000_000_000_000_000_000));
    /// ```
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Number::PosInt(n) if *n <= i64::MAX as u64 => Some(*n as i64),
            Number::NegInt(n) if *n <= i64::MAX as u64 => Some(-(*n as i64) - 1),
            _ => None,
        }
    }

    /// Converts the Twic number to an i128 if it's an integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 1u64.into();
    /// assert_eq!(n.as_i128(), Some(1));
    /// let m: Number = (-1i64).into();
    /// assert_eq!(m.as_i128(), Some(-1));
    /// ```
    pub const fn as_i128(&self) -> Option<i128> {
        match self {
            Number::PosInt(n) => Some(*n as i128),
            Number::NegInt(n) => Some(-(*n as i128) - 1),
            _ => None,
        }
    }

    /// Converts the Twic number to an isize if it fits within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 1isize.into();
    /// assert_eq!(n.as_isize(), Some(1));
    /// let m: Number = (-1isize).into();
    /// assert_eq!(m.as_isize(), Some(-1));
    /// ```
    pub const fn as_isize(&self) -> Option<isize> {
        match self {
            Number::PosInt(n) if *n <= isize::MAX as u64 => Some(*n as isize),
            Number::NegInt(n) if *n <= isize::MAX as u64 => Some(-(*n as isize) - 1),
            _ => None,
        }
    }

    /// Converts the Twic number to a u8 if it fits within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 200u8.into();
    /// assert_eq!(n.as_u8(), Some(200));
    /// ```
    pub const fn as_u8(&self) -> Option<u8> {
        match self {
            Number::PosInt(n) if *n <= u8::MAX as u64 => Some(*n as u8),
            _ => None,
        }
    }

    /// Converts the Twic number to a u16 if it fits within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 60000u16.into();
    /// assert_eq!(n.as_u16(), Some(60000));
    /// ```
    pub const fn as_u16(&self) -> Option<u16> {
        match self {
            Number::PosInt(n) if *n <= u16::MAX as u64 => Some(*n as u16),
            _ => None,
        }
    }

    /// Converts the Twic number to a u32 if it fits within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 3_000_000_000u32.into();
    /// assert_eq!(n.as_u32(), Some(3_000_000_000));
    /// ```
    pub const fn as_u32(&self) -> Option<u32> {
        match self {
            Number::PosInt(n) if *n <= u32::MAX as u64 => Some(*n as u32),
            _ => None,
        }
    }

    /// Converts the Twic number to a u64 if it is a positive integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 18_000_000_000_000_000_000u64.into();
    /// assert_eq!(n.as_u64(), Some(18_000_000_000_000_000_000));
    /// ```
    pub const fn as_u64(&self) -> Option<u64> {
        match self {
            Number::PosInt(n) => Some(*n),
            _ => None,
        }
    }

    /// Converts the Twic number to a u128 if it is a positive integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///     
    /// let n: Number = 1u64.into();
    /// assert_eq!(n.as_u128(), Some(1));
    /// ```
    pub const fn as_u128(&self) -> Option<u128> {
        match self {
            Number::PosInt(n) => Some(*n as u128),
            _ => None,
        }
    }

    /// Converts the Twic number to a usize if it fits within the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 1usize.into();
    /// assert_eq!(n.as_usize(), Some(1));
    /// ```
    pub const fn as_usize(&self) -> Option<usize> {
        match self {
            Number::PosInt(n) if *n <= usize::MAX as u64 => Some(*n as usize),
            _ => None,
        }
    }

    /// Converts the Twic number to an f32 if it is a float, NaN, or infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 3.14f32.into();
    /// assert_eq!(n.as_f32(), Some(3.14));
    /// let nan: Number = f32::NAN.into();
    /// assert!(nan.as_f32().unwrap().is_nan());
    /// let inf: Number = f32::INFINITY.into();
    /// assert_eq!(inf.as_f32(), Some(f32::INFINITY));
    /// ```
    pub const fn as_f32(&self) -> Option<f32> {
        match self {
            Number::Float(n) => Some(*n as f32),
            Number::NaN => Some(f32::NAN),
            Number::Inf { negative } => {
                if *negative {
                    Some(f32::NEG_INFINITY)
                } else {
                    Some(f32::INFINITY)
                }
            }
            _ => None,
        }
    }

    /// Converts the Twic number to an f64 if it is a float, NaN, or infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Number;
    ///
    /// let n: Number = 3.14f64.into();
    /// assert_eq!(n.as_f64(), Some(3.14));
    /// let nan: Number = f64::NAN.into();
    /// assert!(nan.as_f64().unwrap().is_nan());
    /// let inf: Number = f64::INFINITY.into();
    /// assert_eq!(inf.as_f64(), Some(f64::INFINITY));
    /// ```
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Number::Float(n) => Some(*n),
            Number::NaN => Some(f64::NAN),
            Number::Inf { negative } => {
                if *negative {
                    Some(f64::NEG_INFINITY)
                } else {
                    Some(f64::INFINITY)
                }
            }
            _ => None,
        }
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::PosInt(n) => write!(f, "Integer({})", n),
            Number::NegInt(n) if *n == u64::MAX => write!(f, "Integer(-18446744073709551616)"),
            Number::NegInt(n) => write!(f, "Integer(-{})", n + 1),
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
            Number::PosInt(n) => {
                n.hash(state);
            }
            Number::NegInt(n) => {
                (!n).hash(state);
            }
            Number::Float(n) => {
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
    /// let n: Number = 3.14f32.into();
    /// assert!(n.is_float());
    /// let nan: Number = f32::NAN.into();
    /// assert!(nan.is_nan());
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
    /// let n: Number = 3.14f64.into();
    /// assert!(n.is_float());
    /// let nan: Number = f64::NAN.into();
    /// assert!(nan.is_nan());
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
                        Number::NegInt((-(value + 1)) as u64)
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
