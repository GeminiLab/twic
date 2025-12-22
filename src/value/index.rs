use alloc::{borrow::ToOwned, string::String};

use super::Value;

/// Errors that can occur when indexing into a [`Value`].
pub enum ValueIndexError {
    /// The value is not indexable (not a map or vector).
    NotIndexable,
    /// The index type is incompatible with the value type.
    IncompatibleIndexType,
    /// The specified key was not found in the map, or the index is out of
    /// bounds for a vector.
    KeyNotFound,
}

/// Result type for indexing into a [`Value`].
pub type IndexResult<'a> = Result<&'a Value, ValueIndexError>;
/// Mutable result type for indexing into a [`Value`].
pub type IndexMutResult<'a> = Result<&'a mut Value, ValueIndexError>;

/// Trait for types that can index into a [`Value`].
pub trait IndexInto {
    /// Indexes into the given Value, returning a reference to the indexed Value
    /// or an error if the index is invalid or the Value is not indexable.
    fn index_into<'a>(&self, value: &'a Value) -> IndexResult<'a>;
    /// Indexes into the given mutable Value, returning a mutable reference to
    /// the indexed Value or an error if the index is invalid or the Value is
    /// not indexable.
    fn index_into_mut<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a>;
    /// Indexes into the given mutable Value, inserting a new Value if the index
    /// does not exist. Returns a mutable reference to the indexed Value, or an
    /// error if the index is invalid or the Value is not indexable.
    fn index_into_or_insert<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a>;
}

impl IndexInto for usize {
    fn index_into<'a>(&self, value: &'a Value) -> IndexResult<'a> {
        if let Value::Vector(vec) = value {
            vec.get(*self).ok_or(ValueIndexError::KeyNotFound)
        } else if value.is_map() {
            Err(ValueIndexError::IncompatibleIndexType)
        } else {
            Err(ValueIndexError::NotIndexable)
        }
    }

    fn index_into_mut<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a> {
        if let Value::Vector(vec) = value {
            vec.get_mut(*self).ok_or(ValueIndexError::KeyNotFound)
        } else if value.is_map() {
            Err(ValueIndexError::IncompatibleIndexType)
        } else {
            Err(ValueIndexError::NotIndexable)
        }
    }

    fn index_into_or_insert<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a> {
        if let Value::Vector(vec) = value {
            if *self >= vec.len() {
                vec.extend(core::iter::repeat_n(Value::Null, *self - vec.len() + 1));
            }

            Ok(&mut vec[*self])
        } else if value.is_map() {
            Err(ValueIndexError::IncompatibleIndexType)
        } else {
            Err(ValueIndexError::NotIndexable)
        }
    }
}

impl IndexInto for str {
    fn index_into<'a>(&self, value: &'a Value) -> IndexResult<'a> {
        if let Value::Map(map) = value {
            map.get(self).ok_or(ValueIndexError::KeyNotFound)
        } else if value.is_vector() {
            Err(ValueIndexError::IncompatibleIndexType)
        } else {
            Err(ValueIndexError::NotIndexable)
        }
    }

    fn index_into_mut<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a> {
        if let Value::Map(map) = value {
            map.get_mut(self).ok_or(ValueIndexError::KeyNotFound)
        } else if value.is_vector() {
            Err(ValueIndexError::IncompatibleIndexType)
        } else {
            Err(ValueIndexError::NotIndexable)
        }
    }

    fn index_into_or_insert<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a> {
        if let Value::Map(map) = value {
            Ok(map.entry(self.to_owned()).or_insert(Value::Null))
        } else if value.is_vector() {
            Err(ValueIndexError::IncompatibleIndexType)
        } else {
            Err(ValueIndexError::NotIndexable)
        }
    }
}

impl IndexInto for String {
    fn index_into<'a>(&self, value: &'a Value) -> IndexResult<'a> {
        self.as_str().index_into(value)
    }

    fn index_into_mut<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a> {
        self.as_str().index_into_mut(value)
    }

    fn index_into_or_insert<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a> {
        self.as_str().index_into_or_insert(value)
    }
}

impl<T: IndexInto + ?Sized> IndexInto for &T {
    fn index_into<'a>(&self, value: &'a Value) -> IndexResult<'a> {
        (**self).index_into(value)
    }

    fn index_into_mut<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a> {
        (**self).index_into_mut(value)
    }

    fn index_into_or_insert<'a>(&self, value: &'a mut Value) -> IndexMutResult<'a> {
        (**self).index_into_or_insert(value)
    }
}

impl<T: IndexInto> core::ops::Index<T> for Value {
    type Output = Value;

    /// Indexes into the [`Value`] using the given index type. If the index does
    /// not exist, this method returns a [`Value::Null`].
    ///
    /// # Panics
    ///
    /// Panics if the value is not indexable, the index type is incompatible,
    /// or the key/index is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::{Value, Map};
    ///
    /// let mut v = Value::Vector(vec![Value::Number(1.into()), Value::Number(2.into())]);
    /// assert_eq!(v[0], Value::Number(1.into()));
    ///
    /// let mut m = Value::Map(Map::new());
    /// m["key"] = Value::String("value".to_owned());
    /// assert_eq!(m["key"], Value::String("value".to_owned()));
    /// ```
    ///
    /// ```
    /// use twic::value::{Value, Map};
    ///
    /// let mut m = Value::Map(Map::new());
    /// assert_eq!(m["missing_key"], Value::Null);
    /// ```
    ///
    /// ```should_panic
    /// use twic::value::Value;
    ///
    /// let v = Value::Number(42.into());
    /// let _ = v[0]; // Panics: Value of type Number is not indexable
    /// ```
    ///
    /// ```should_panic
    /// use twic::value::{Value, Map};
    ///
    /// let mut m = Value::Map(Map::new());
    /// let _ = m[0]; // Panics: Incompatible index type for Value of type Map
    /// ```
    fn index(&self, index: T) -> &Self::Output {
        match index.index_into(self) {
            Ok(value) => value,
            Err(ValueIndexError::NotIndexable) => {
                panic!("Value of type {} is not indexable", self.type_name())
            }
            Err(ValueIndexError::IncompatibleIndexType) => {
                panic!(
                    "Incompatible index type for Value of type {}",
                    self.type_name()
                )
            }
            Err(ValueIndexError::KeyNotFound) => {
                static NULL_VALUE: Value = Value::Null;
                &NULL_VALUE
            }
        }
    }
}

impl<T: IndexInto> core::ops::IndexMut<T> for Value {
    /// Mutably indexes into the [`Value`] using the given index type. If the
    /// index does not exist, a new value is inserted.
    ///
    /// # Panics
    ///
    /// Panics if the value is not indexable or the index type is incompatible.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::{Value, Map};
    ///
    /// let mut v = Value::Vector(vec![Value::Number(1.into()), Value::Number(2.into())]);
    /// v[1] = Value::Number(3.into());
    /// assert_eq!(v[1], Value::Number(3.into()));
    ///
    /// let mut m = Value::Map(Map::new());
    /// m["key"] = Value::String("value".to_owned());
    /// assert_eq!(m["key"], Value::String("value".to_owned()));
    /// ```
    ///
    /// ```should_panic
    /// use twic::value::Value;
    ///
    /// let mut v = Value::Number(42.into());
    /// v[0] = Value::Number(1.into()); // Panics: Value of type Number is not indexable
    /// ```
    ///
    /// ```should_panic
    /// use twic::value::{Value, Map};
    ///
    /// let mut m = Value::Map(Map::new());
    /// m[0] = Value::Number(1.into()); // Panics: Incompatible index type for Value of type Map
    /// ```
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let type_name = self.type_name();
        match index.index_into_or_insert(self) {
            Ok(value) => value,
            Err(ValueIndexError::NotIndexable) => {
                panic!("Value of type {} is not indexable", type_name)
            }
            Err(ValueIndexError::IncompatibleIndexType) => {
                panic!("Incompatible index type for Value of type {}", type_name)
            }
            Err(ValueIndexError::KeyNotFound) => {
                unreachable!("KeyNotFound should not occur in index_into_or_insert")
            }
        }
    }
}
