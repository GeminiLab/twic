use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
    vec::Vec,
};

use super::{Number, Value};

impl From<()> for Value {
    /// Converts a unit type to a Twic null value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v: Value = ().into();
    /// assert!(v.is_null());
    /// ```
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl From<bool> for Value {
    /// Converts a boolean to a Twic boolean value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v: Value = true.into();
    /// assert!(v.is_boolean());
    /// ```
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl<T: Into<Number>> From<T> for Value {
    /// Converts a convertible item to a Twic number value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let n: Value = 3.14f64.into();
    /// assert!(n.is_number());
    /// ```
    fn from(value: T) -> Self {
        Value::Number(value.into())
    }
}

impl From<String> for Value {
    /// Converts a String to a Twic string value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v: Value = String::from("hello").into();
    /// assert!(v.is_string());
    /// ```
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    /// Converts a string slice to a Twic string value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v: Value = "hello".into();
    /// assert!(v.is_string());
    /// ```
    fn from(value: &str) -> Self {
        Value::String(value.to_owned())
    }
}

impl From<Cow<'_, str>> for Value {
    /// Converts a Cow string to a Twic string value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    /// use std::borrow::Cow;
    ///
    /// let v: Value = Cow::Borrowed("hello").into();
    /// assert!(v.is_string());
    ///
    /// let v: Value = Cow::<str>::Owned(String::from("hello")).into();
    /// assert!(v.is_string());
    /// ```
    fn from(value: Cow<'_, str>) -> Self {
        Value::String(value.into_owned())
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    /// Converts a vector of convertible items to a Twic vector value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v: Value = vec!["one", "two", "three"].into();
    /// assert!(v.is_vector());
    /// ```
    fn from(value: Vec<T>) -> Self {
        Value::vector_from(value)
    }
}

impl<T: Into<Value>, const N: usize> From<[T; N]> for Value {
    /// Converts an array of convertible items to a Twic vector value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let arr = ["one", "two", "three"];
    /// let v: Value = arr.into();
    /// assert!(v.is_vector());
    /// ```
    fn from(value: [T; N]) -> Self {
        Value::vector_from(value)
    }
}

impl<T: Clone + Into<Value>> From<&[T]> for Value {
    /// Converts a reference to a slice of convertible items to a Twic vector
    /// value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let arr: &[&str] = &["one", "two", "three"];
    /// let v: Value = arr.into();
    /// assert!(v.is_vector());
    /// ```
    fn from(value: &[T]) -> Self {
        Value::vector_clone_from(value)
    }
}

impl<T: Clone + Into<Value>, const N: usize> From<&[T; N]> for Value {
    /// Converts a reference to an array of convertible items to a Twic vector
    /// value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let arr: &[&str; 3] = &["one", "two", "three"];
    /// let v: Value = arr.into();
    /// assert!(v.is_vector());
    /// ```
    fn from(value: &[T; N]) -> Self {
        Value::vector_clone_from(value)
    }
}

impl<T: Into<Value>> FromIterator<T> for Value {
    /// Creates a Twic vector value by collecting an iterator of convertible
    /// items.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v: Value = std::iter::repeat("item").take(5).collect();
    /// assert!(v.is_vector());
    ///
    /// let v: Value = vec!["one", "two", "three"].into_iter().collect();
    /// assert!(v.is_vector());
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Value::vector_from(iter)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    /// Converts an Option of a convertible item to a Twic value.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let some_value: Option<&str> = Some("hello");
    /// let v: Value = some_value.into();
    /// assert!(v.is_string());
    ///
    /// let none_value: Option<&str> = None;
    /// let v: Value = none_value.into();
    /// assert!(v.is_null());
    /// ```
    fn from(option: Option<T>) -> Self {
        match option {
            Some(value) => value.into(),
            None => Value::Null,
        }
    }
}
