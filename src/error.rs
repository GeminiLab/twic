use alloc::string::String;
use core::error::Error;
use core::fmt;

/// Errors that can occur during tokenization or parsing of Twic input.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
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
    UnexpectedEof { expected: &'static str },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnfinishedString => write!(f, "unfinished string"),
            ParseError::InvalidEscape => write!(f, "invalid escape sequence"),
            ParseError::InvalidNumber(s) => write!(f, "invalid number: {}", s),
            ParseError::UnexpectedToken { expected, found } => {
                write!(f, "expected {}, found {}", expected, found)
            }
            ParseError::UnexpectedEof { expected } => {
                write!(f, "unexpected end of input, expected {}", expected)
            }
        }
    }
}

impl Error for ParseError {}
