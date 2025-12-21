use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
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
    /// use twic::value::Number;
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
    /// let s = String::from("hello");
    /// let v: Value = s.into();
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
    /// let cow: Cow<str> = Cow::Borrowed("hello");
    /// let v: Value = cow.into();
    /// assert!(v.is_string());
    /// ```
    ///
    /// ```
    /// use twic::value::Value;
    /// use std::borrow::Cow;
    ///
    /// let cow: Cow<str> = Cow::Owned(String::from("hello"));
    /// let v: Value = cow.into();
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
    /// let vec = vec!["one", "two", "three"];
    /// let v: Value = vec.into();
    /// assert!(v.is_vector());
    /// ```
    fn from(value: Vec<T>) -> Self {
        Value::Vector(value.into_iter().map(Into::into).collect())
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
        Value::Vector(value.into_iter().map(Into::into).collect())
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
        Value::Vector(value.iter().cloned().map(Into::into).collect())
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
        Value::Vector(value.iter().cloned().map(Into::into).collect())
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
    /// let iter = std::iter::repeat("item").take(5);
    /// let v: Value = iter.collect();
    /// assert!(v.is_vector());
    /// ```
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let vec = vec!["one", "two", "three"];
    /// let v: Value = vec.into_iter().collect();
    /// assert!(v.is_vector());
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Value::Vector(iter.into_iter().map(Into::into).collect())
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
