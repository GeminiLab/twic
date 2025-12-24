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

/// Panics indicating that the given type is not indexable.
fn panic_not_indexable(type_name: &str) -> ! {
    panic!("twic value of type {} is not indexable", type_name)
}

/// Panics indicating that the given index type is incompatible with the value
/// type.
fn panic_incompatible_index_type(type_name: &str) -> ! {
    panic!(
        "incompatible index type for twic value of type {}",
        type_name
    )
}

impl<T: IndexInto> core::ops::Index<T> for Value {
    type Output = Value;

    /// Indexes into the [`Value`] using the given index type. If the index does
    /// not exist, this method returns a [`Value::Null`].
    ///
    /// # Panics
    ///
    /// Panics if the value is not indexable, the index type is incompatible.
    ///
    /// # Examples
    ///
    /// ```
    /// use twic::value::{Value, Map};
    ///
    /// let mut v = Value::vector_from([1, 2]);
    /// assert_eq!(v[0], Value::Number(1.into()));
    ///
    /// let mut m = Value::map_empty();
    /// m["key"] = Value::string("value");
    /// assert_eq!(m["key"], Value::string("value"));
    /// ```
    ///
    /// ```
    /// use twic::value::{Value, Map};
    ///
    /// let mut m = Value::map_empty();
    /// assert_eq!(m["missing_key"], Value::Null);
    /// ```
    ///
    /// ```should_panic
    /// use twic::value::Value;
    ///
    /// let v = Value::number(42);
    /// let _ = v[0]; // Panics: Value of type Number is not indexable
    /// ```
    ///
    /// ```should_panic
    /// use twic::value::{Value, Map};
    ///
    /// let mut m = Value::map_empty();
    /// let _ = m[0]; // Panics: Incompatible index type for Value of type Map
    /// ```
    fn index(&self, index: T) -> &Self::Output {
        let type_name = self.type_name();
        match index.index_into(self) {
            Ok(value) => value,
            Err(ValueIndexError::NotIndexable) => panic_not_indexable(type_name),
            Err(ValueIndexError::IncompatibleIndexType) => panic_incompatible_index_type(type_name),
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
    /// When indexing into a vector with an out-of-bounds index, the vector is
    /// automatically extended with `Value::Null` values up to the specified
    /// index.
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
    /// let mut v = Value::vector_from([1, 2]);
    /// v[1] = Value::number(3);
    /// assert_eq!(v[1], Value::number(3));
    ///
    /// v[4] = Value::number(5); // Extends the vector with Nulls
    /// assert_eq!(v[2], Value::Null);
    /// assert_eq!(v[3], Value::Null);
    /// assert_eq!(v[4], Value::number(5));
    ///
    /// let mut m = Value::map_empty();
    /// m["key"] = Value::string("value");
    /// assert_eq!(m["key"], Value::string("value"));
    /// ```
    ///
    /// ```should_panic
    /// use twic::value::Value;
    ///
    /// let mut v = Value::number(42);
    /// v[0] = Value::number(1); // Panics: Value of type Number is not indexable
    /// ```
    ///
    /// ```should_panic
    /// use twic::value::{Value, Map};
    ///
    /// let mut m = Value::map_empty();
    /// m[0] = Value::number(1); // Panics: Incompatible index type for Value of type Map
    /// ```
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let type_name = self.type_name();
        match index.index_into_or_insert(self) {
            Ok(value) => value,
            Err(ValueIndexError::NotIndexable) => panic_not_indexable(type_name),
            Err(ValueIndexError::IncompatibleIndexType) => panic_incompatible_index_type(type_name),
            Err(ValueIndexError::KeyNotFound) => {
                unreachable!("KeyNotFound should not occur in index_into_or_insert")
            }
        }
    }
}
