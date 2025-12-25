use super::Value;

use alloc::string::String;

macro_rules! impl_eq_for {
    ($(
        $t:ty => $method:ident
    ),* $(,)?) => {
        $(
            impl PartialEq<$t> for Value {
                fn eq(&self, other: &$t) -> bool {
                    match self.$method() {
                        Some(v) => v == *other,
                        None => false,
                    }
                }
            }

            impl PartialEq<Value> for $t {
                fn eq(&self, other: &Value) -> bool {
                    match other.$method() {
                        Some(v) => *self == v,
                        None => false,
                    }
                }
            }
        )*
    };
}

impl_eq_for! {
    () => as_null,
    bool => as_boolean,
    i8 => as_number,
    i16 => as_number,
    i32 => as_number,
    i64 => as_number,
    i128 => as_number,
    isize => as_number,
    u8 => as_number,
    u16 => as_number,
    u32 => as_number,
    u64 => as_number,
    u128 => as_number,
    usize => as_number,
    f32 => as_number,
    f64 => as_number,
    &str => as_str,
}

impl PartialEq<str> for Value {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == Some(other)
    }
}

impl PartialEq<Value> for str {
    fn eq(&self, other: &Value) -> bool {
        other.as_str() == Some(self)
    }
}

impl PartialEq<String> for Value {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == Some(other.as_str())
    }
}

impl PartialEq<Value> for String {
    fn eq(&self, other: &Value) -> bool {
        other.as_str() == Some(self.as_str())
    }
}
