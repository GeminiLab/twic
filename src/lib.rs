#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

pub mod value;

#[doc(inline)]
pub use value::{Map, Number, Value};
