use alloc::{collections::BTreeMap, string::String};

/// A map value in Twic, mapping string keys to [`Value`](super::Value)s.
pub type Map = BTreeMap<String, super::Value>;
