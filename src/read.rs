use crate::error::Result;
use crate::value::Value;

pub enum Token {
    Colon,
    Comma,
    SemiColon,
    True,
    False,
    Number(String),
    String,
}

pub trait TokenRead: Iterator<Item = Token> + Sized {
    fn into_value(self) -> Result<Value> {
        todo!()
    }
}

pub struct StrReader<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Iterator for StrReader<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<'a> TokenRead for StrReader<'a> {}

#[cfg(feature = "std")]
mod with_std {
    use super::*;

    pub struct StdTokenReader<R: std::io::Read> {
        reader: R,
    }

    impl<R: std::io::Read> Iterator for StdTokenReader<R> {
        type Item = Token;

        fn next(&mut self) -> Option<Self::Item> {
            todo!()
        }
    }

    impl<R: std::io::Read> TokenRead for StdTokenReader<R> {}
}

#[cfg(feature = "std")]
pub use with_std::*;
