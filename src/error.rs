use alloc::string::String;
use core::fmt;

/// Errors that can occur during tokenization or parsing of Twic input.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// A quoted string was not closed before end of input.
    UnfinishedString,
    /// An invalid escape sequence was encountered in a quoted string.
    InvalidEscape,
    /// A number could not be parsed from the given text.
    InvalidNumber(String),
    /// An unexpected token was encountered during parsing.
    UnexpectedToken {
        expected: &'static str,
        found: &'static str,
    },
    /// Unexpected end of input.
    UnexpectedEof {
        expected: &'static str,
    },
    /// A trailing comma was found without a following element.
    TrailingComma,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnfinishedString => write!(f, "unfinished string"),
            Error::InvalidEscape => write!(f, "invalid escape sequence"),
            Error::InvalidNumber(s) => write!(f, "invalid number: {}", s),
            Error::UnexpectedToken { expected, found } => {
                write!(f, "expected {}, found {}", expected, found)
            }
            Error::UnexpectedEof { expected } => {
                write!(f, "unexpected end of input, expected {}", expected)
            }
            Error::TrailingComma => write!(f, "trailing comma"),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
