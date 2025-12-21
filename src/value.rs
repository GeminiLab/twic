//! The Twic [`Value`] enum, representing all possible Twic values.

use std::collections::HashMap;

mod convert;
mod number;

#[doc(inline)]
pub use number::Number;

/// Represents a Twic value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Represents a Twic null value.
    Null,
    /// Represents a Twic boolean value.
    Boolean(bool),
    /// Represents a Twic number value.
    Number(Number),
    /// Represents a Twic string value.
    String(String),
    /// Represents a Twic vector value.
    Vector(Vec<Value>),
    /// Represents a Twic map value.
    Map(HashMap<String, Value>),
}

impl Value {
    /// Checks if the value is null.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::Null;
    /// assert!(v.is_null());
    /// ```
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Checks if the value is a boolean.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::Boolean(true);
    /// assert!(v.is_boolean());
    /// ```
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Checks if the value is a number.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::Number(3.14.into());
    /// assert!(v.is_number());
    /// ```
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Checks if the value is a string.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::String("hello".to_owned());
    /// assert!(v.is_string());
    /// ```
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Checks if the value is a vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::Vector(vec![1.0.into(), 2.0.into()]);
    /// assert!(v.is_vector());
    /// ```
    pub fn is_vector(&self) -> bool {
        matches!(self, Value::Vector(_))
    }

    /// Checks if the value is a map.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    /// use std::collections::HashMap;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("key".to_owned(), 42f64.into());
    /// let v = Value::Map(map);
    /// assert!(v.is_map());
    /// ```
    pub fn is_map(&self) -> bool {
        matches!(self, Value::Map(_))
    }
}
