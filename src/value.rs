//! The Twic [`Value`] enum, representing all possible Twic values.

use alloc::{string::String, vec::Vec};

mod convert;
mod index;
mod map;
mod number;

#[doc(inline)]
pub use index::{IndexInto, IndexMutResult, IndexResult, ValueIndexError};
#[doc(inline)]
pub use map::Map;
#[doc(inline)]
pub use number::Number;

/// Represents a Twic value.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Value {
    /// Represents a Twic null value.
    #[default]
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
    Map(Map),
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

    /// Returns the null value if the value is null, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::Null;
    /// assert_eq!(v.as_null(), Some(()));
    /// ```
    pub fn as_null(&self) -> Option<()> {
        if self.is_null() { Some(()) } else { None }
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

    /// Returns the boolean value if the value is a boolean, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::Boolean(true);
    /// assert_eq!(v.as_boolean(), Some(true));
    /// ```
    pub fn as_boolean(&self) -> Option<bool> {
        if let Value::Boolean(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the boolean value if the value is a
    /// boolean, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let mut v = Value::Boolean(true);
    /// if let Some(b) = v.as_boolean_mut() {
    ///     *b = false;
    /// }
    /// assert_eq!(v.as_boolean(), Some(false));
    /// ```
    pub fn as_boolean_mut(&mut self) -> Option<&mut bool> {
        if let Value::Boolean(b) = self {
            Some(b)
        } else {
            None
        }
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

    /// Returns the number value if the value is a number, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::{Value, Number};
    ///
    /// let v = Value::Number(3.14.into());
    /// assert_eq!(v.as_number(), Some(Number::from(3.14)));
    /// ```
    pub fn as_number(&self) -> Option<Number> {
        if let Value::Number(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the number value if the value is a
    /// number, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::{Value, Number};
    ///
    /// let mut v = Value::Number(3.14.into());
    /// if let Some(n) = v.as_number_mut() {
    ///     *n = Number::from(2.71);
    /// }
    /// assert_eq!(v.as_number(), Some(Number::from(2.71)));
    /// ```
    pub fn as_number_mut(&mut self) -> Option<&mut Number> {
        if let Value::Number(n) = self {
            Some(n)
        } else {
            None
        }
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

    /// Returns the string reference if the value is a string, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::String("hello".to_owned());
    /// assert_eq!(v.as_string(), Some(&"hello".to_owned()));
    /// ```
    pub fn as_string(&self) -> Option<&String> {
        if let Value::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the string if the value is a string,
    /// `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let mut v = Value::String("hello".to_owned());
    /// if let Some(s) = v.as_string_mut() {
    ///     s.push_str(" world");
    /// }
    /// assert_eq!(v.as_string(), Some(&"hello world".to_owned()));
    /// ```
    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        if let Value::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns the string slice if the value is a string, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::String("hello".to_owned());
    /// assert_eq!(v.as_str(), Some("hello"));
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(s) = self {
            Some(s)
        } else {
            None
        }
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

    /// Returns the vector reference if the value is a vector, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::Vector(vec![1.0.into(), 2.0.into()]);
    /// assert_eq!(v.as_vector(), Some(&vec![1.0.into(), 2.0.into()]));
    /// ```
    pub fn as_vector(&self) -> Option<&Vec<Value>> {
        if let Value::Vector(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the vector if the value is a vector,
    /// `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let mut v = Value::Vector(vec![1.0.into(), 2.0.into()]);
    /// if let Some(vec) = v.as_vector_mut() {
    ///     vec.push(3.0.into());
    /// }
    /// assert_eq!(v.as_vector(), Some(&vec![1.0.into(), 2.0.into(), 3.0.into()]));
    /// ```
    pub fn as_vector_mut(&mut self) -> Option<&mut Vec<Value>> {
        if let Value::Vector(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Checks if the value is a map.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    /// use twic::value::Map;
    ///
    /// let mut map = Map::new();
    /// map.insert("key".to_owned(), 42f64.into());
    /// let v = Value::Map(map);
    /// assert!(v.is_map());
    /// ```
    pub fn is_map(&self) -> bool {
        matches!(self, Value::Map(_))
    }

    /// Returns the map reference if the value is a map, `None` otherwise.
    pub fn as_map(&self) -> Option<&Map> {
        if let Value::Map(m) = self {
            Some(m)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the map if the value is a map,
    /// `None` otherwise.
    pub fn as_map_mut(&mut self) -> Option<&mut Map> {
        if let Value::Map(m) = self {
            Some(m)
        } else {
            None
        }
    }

    /// Indexes into the Value using the provided index. Returns `Some(&Value)`
    /// if the value is indexable and the index exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::{Value, Map};
    ///
    /// let mut map = Map::new();
    /// map.insert("key".to_owned(), 42f64.into());
    /// let v = Value::Map(map);
    /// assert_eq!(v.get("key"), Some(&42f64.into()));
    /// ```
    pub fn get<I: IndexInto>(&self, index: I) -> Option<&Value> {
        index.index_into(self).ok()
    }

    /// Indexes into the Value using the provided index mutably. Returns
    /// `Some(&mut Value)` if the value is indexable and the index exists,
    /// `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::{Value, Map};
    ///
    /// let mut map = Map::new();
    /// map.insert("key".to_owned(), 42f64.into());
    /// let mut v = Value::Map(map);
    /// if let Some(val) = v.get_mut("key") {
    ///     *val = 100f64.into();
    /// }
    /// assert_eq!(v.get("key"), Some(&100f64.into()));
    /// ```
    pub fn get_mut<I: IndexInto>(&mut self, index: I) -> Option<&mut Value> {
        index.index_into_mut(self).ok()
    }

    /// Indexes into the Value using the provided index mutably, inserting a new
    /// Value if the index does not exist. Returns `Some(&mut Value)` if the
    /// value is indexable, `None` otherwise.
    ///
    /// When a new Value is inserted, it is initialized to `Value::Null`. When
    /// indexing into a vector, if the index is out of bounds, the vector is
    /// extended with `Value::Null` values up to the specified index.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::{Value, Map};
    ///
    /// let mut v = Value::Map(Map::new());
    /// if let Some(val) = v.get_or_insert("new_key") {
    ///     *val = 55f64.into();
    /// }
    /// assert_eq!(v.get("new_key"), Some(&55f64.into()));
    /// ```
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let mut v = Value::Vector(vec![1f64.into(), 2f64.into()]);
    /// if let Some(val) = v.get_or_insert(4) {
    ///    *val = 5f64.into();
    /// }
    /// assert_eq!(v.as_vector(), Some(&vec![1f64.into(), 2f64.into(), Value::Null, Value::Null, 5f64.into()]));
    /// ```
    pub fn get_or_insert<I: IndexInto>(&mut self, index: I) -> Option<&mut Value> {
        index.index_into_or_insert(self).ok()
    }
}

impl Value {
    /// Returns the type name of the value as a string slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::Value;
    ///
    /// let v = Value::String("hello".to_owned());
    /// assert_eq!(v.type_name(), "string");
    /// ```
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Boolean(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Vector(_) => "vector",
            Value::Map(_) => "map",
        }
    }
}
