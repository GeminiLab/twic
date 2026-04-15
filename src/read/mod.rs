mod lexer;
mod span;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::iter::Peekable;
use core::str::CharIndices;

use crate::error::Error;
use crate::value::{Map, Number, Value};

pub use span::{Span, Spanned};

use lexer::CharReader;

/// The result of a single tokenization step: either a spanned token or a spanned error.
pub type TokenResult = core::result::Result<Spanned<Token>, Spanned<Error>>;

/// A token produced by the Twic tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Colon,
    Comma,
    SemiColon,
    Null,
    True,
    False,
    Number(String),
    String(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::SemiColon => write!(f, ";"),
            Token::Null => write!(f, "null"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Number(s) => write!(f, "Number({})", s),
            Token::String(s) => write!(f, "String({:?})", s),
        }
    }
}

/// A trait for token readers that can be converted into a [`Value`].
pub trait TokenRead: Iterator<Item = TokenResult> + Sized {
    /// Consumes the token stream and produces a [`Value`].
    fn into_value(self) -> core::result::Result<Value, Error>;
}

/// A tokenizer that reads from a `&str` input.
pub struct StrReader<'a> {
    chars: Peekable<CharIndices<'a>>,
    line: usize,
    column: usize,
}

impl<'a> StrReader<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.char_indices().peekable(),
            line: 1,
            column: 1,
        }
    }
}

impl<'a> CharReader for StrReader<'a> {
    fn next_char(&mut self) -> Option<char> {
        let (_, c) = self.chars.next()?;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, c)| c)
    }

    fn current_line(&self) -> usize {
        self.line
    }

    fn current_column(&self) -> usize {
        self.column
    }
}

impl<'a> Iterator for StrReader<'a> {
    type Item = TokenResult;

    fn next(&mut self) -> Option<Self::Item> {
        lexer::tokenize_next(self)
    }
}

impl<'a> TokenRead for StrReader<'a> {
    fn into_value(self) -> core::result::Result<Value, Error> {
        let mut parser = Parser {
            iter: self.peekable(),
        };
        let value = parser.parse_value()?;
        // Ensure no trailing tokens
        if parser.next_token()?.is_some() {
            return Err(Error::TrailingComma);
        }
        Ok(value)
    }
}

// ---------------------------------------------------------------------------
// StdTokenReader (std feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "std")]
mod with_std {
    use alloc::boxed::Box;
    use alloc::string::String;
    use core::iter::Peekable;

    use crate::error::Error;
    use crate::value::Value;

    use super::lexer::CharReader;
    use super::{TokenResult, TokenRead};

    /// A tokenizer that reads from a `std::io::Read` stream.
    pub struct StdTokenReader<R: std::io::Read> {
        chars: Peekable<std::str::Chars<'static>>,
        line: usize,
        column: usize,
        _reader: R,
        _buf: Box<String>,
    }

    impl<R: std::io::Read> StdTokenReader<R> {
        pub fn new(mut reader: R) -> std::io::Result<Self> {
            let mut buf = String::new();
            reader.read_to_string(&mut buf)?;
            let boxed: Box<String> = Box::new(buf);
            // SAFETY: We convert the Chars<'a> lifetime to 'static via raw pointer.
            // This is safe because `_buf` (stored below) keeps the String alive for
            // the entire lifetime of Self, and the chars iterator only borrows from it.
            let chars: Peekable<std::str::Chars<'static>> = unsafe {
                let ptr: *const String = &*boxed;
                (*ptr).chars().peekable()
            };
            Ok(Self {
                chars,
                line: 1,
                column: 1,
                _reader: reader,
                _buf: boxed,
            })
        }
    }

    impl<R: std::io::Read> CharReader for StdTokenReader<R> {
        fn next_char(&mut self) -> Option<char> {
            let c = self.chars.next()?;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(c)
        }

        fn peek_char(&mut self) -> Option<char> {
            self.chars.peek().copied()
        }

        fn current_line(&self) -> usize {
            self.line
        }

        fn current_column(&self) -> usize {
            self.column
        }
    }

    impl<R: std::io::Read> Iterator for StdTokenReader<R> {
        type Item = TokenResult;

        fn next(&mut self) -> Option<Self::Item> {
            super::lexer::tokenize_next(self)
        }
    }

    impl<R: std::io::Read> TokenRead for StdTokenReader<R> {
        fn into_value(self) -> core::result::Result<Value, Error> {
            let mut parser = super::Parser {
                iter: self.peekable(),
            };
            let value = parser.parse_value()?;
            if parser.next_token()?.is_some() {
                return Err(Error::TrailingComma);
            }
            Ok(value)
        }
    }
}

#[cfg(feature = "std")]
pub use with_std::StdTokenReader;

// ---------------------------------------------------------------------------
// Recursive descent parser (into_value)
// ---------------------------------------------------------------------------

struct Parser<I: Iterator<Item = TokenResult>> {
    iter: Peekable<I>,
}

impl<I: Iterator<Item = TokenResult>> Parser<I> {
    fn next_token(&mut self) -> core::result::Result<Option<Token>, Error> {
        match self.iter.next() {
            None => Ok(None),
            Some(Err(e)) => Err(e.value),
            Some(Ok(spanned)) => Ok(Some(spanned.value)),
        }
    }

    fn peek_token(&mut self) -> core::result::Result<Option<Token>, Error> {
        match self.iter.peek() {
            None => Ok(None),
            Some(Ok(spanned)) => Ok(Some(spanned.value.clone())),
            Some(Err(_)) => {
                let item = self.iter.next();
                match item {
                    Some(Err(e)) => Err(e.value),
                    _ => unreachable!(),
                }
            }
        }
    }

    fn expect_token(&mut self, expected: &'static str) -> core::result::Result<Token, Error> {
        match self.next_token()? {
            Some(token) => Ok(token),
            None => Err(Error::UnexpectedEof { expected }),
        }
    }

    fn expect_semicolon(&mut self) -> core::result::Result<(), Error> {
        match self.next_token()? {
            Some(Token::SemiColon) => Ok(()),
            Some(other) => Err(Error::UnexpectedToken {
                expected: ";",
                found: token_name(&other),
            }),
            None => Err(Error::UnexpectedEof { expected: ";" }),
        }
    }

    fn parse_value(&mut self) -> core::result::Result<Value, Error> {
        let token = self.expect_token("value")?;

        match token {
            Token::Null => Ok(Value::Null),
            Token::True => Ok(Value::Boolean(true)),
            Token::False => Ok(Value::Boolean(false)),
            Token::Number(s) => parse_number(&s),
            Token::Colon => self.parse_vector(),
            Token::SemiColon => Ok(Value::Map(Map::new())),
            Token::String(s) => {
                match self.peek_token()? {
                    Some(Token::Colon) => {
                        self.iter.next(); // consume the Colon
                        self.parse_map_with_first_key(s)
                    }
                    _ => Ok(Value::String(s)),
                }
            }

            Token::Comma => Err(Error::TrailingComma),
        }
    }

    fn parse_vector(&mut self) -> core::result::Result<Value, Error> {
        let mut vec = Vec::new();

        if !matches!(self.peek_token()?, Some(Token::SemiColon)) {
            vec.push(self.parse_value()?);
            while matches!(self.peek_token()?, Some(Token::Comma)) {
                self.iter.next(); // consume Comma
                if matches!(self.peek_token()?, Some(Token::SemiColon)) {
                    return Err(Error::TrailingComma);
                }
                vec.push(self.parse_value()?);
            }
        }

        self.expect_semicolon()?;
        Ok(Value::Vector(vec))
    }

    fn parse_map_with_first_key(
        &mut self,
        first_key: String,
    ) -> core::result::Result<Value, Error> {
        let mut map = Map::new();
        let value = self.parse_value()?;
        map.insert(first_key, value);

        while matches!(self.peek_token()?, Some(Token::Comma)) {
            self.iter.next(); // consume Comma

            let key_token = self.expect_token("string key")?;
            let key = match key_token {
                Token::String(s) => s,
                other => {
                    return Err(Error::UnexpectedToken {
                        expected: "string key",
                        found: token_name(&other),
                    })
                }
            };

            match self.next_token()? {
                Some(Token::Colon) => {}
                Some(other) => {
                    return Err(Error::UnexpectedToken {
                        expected: ":",
                        found: token_name(&other),
                    })
                }
                None => return Err(Error::UnexpectedEof { expected: ":" }),
            }
            let value = self.parse_value()?;
            map.insert(key, value);
        }

        self.expect_semicolon()?;
        Ok(Value::Map(map))
    }
}

fn token_name(token: &Token) -> &'static str {
    match token {
        Token::Colon => ":",
        Token::Comma => ",",
        Token::SemiColon => ";",
        Token::Null => "null",
        Token::True => "true",
        Token::False => "false",
        Token::Number(_) => "number",
        Token::String(_) => "string",
    }
}

fn parse_number(text: &str) -> core::result::Result<Value, Error> {
    // Special number keywords
    match text {
        "nan" => return Ok(Value::Number(Number::NaN)),
        "inf" | "+inf" => return Ok(Value::Number(Number::Inf { negative: false })),
        "-inf" => return Ok(Value::Number(Number::Inf { negative: true })),
        _ => {}
    }

    // Hexadecimal
    let text_no_sign = text.strip_prefix(['+', '-']).unwrap_or(text);
    let negative = text.starts_with('-');

    if let Some(hex_str) = text_no_sign.strip_prefix("0x") {
        let val = u64::from_str_radix(hex_str, 16)
            .map_err(|_| Error::InvalidNumber(text.to_owned()))?;
        if negative {
            Ok(Value::Number(Number::NegInt(val.wrapping_neg() as u64)))
        } else {
            Ok(Value::Number(Number::PosInt(val)))
        }
    } else if text.contains('.') || text.contains('e') || text.contains('E') {
        let val: f64 = text
            .parse()
            .map_err(|_| Error::InvalidNumber(text.to_owned()))?;
        Ok(Value::Number(Number::Float(val)))
    } else {
        let val: i64 = text
            .parse()
            .map_err(|_| Error::InvalidNumber(text.to_owned()))?;
        if val >= 0 {
            Ok(Value::Number(Number::PosInt(val as u64)))
        } else {
            Ok(Value::Number(Number::NegInt((val as u64).wrapping_sub(1))))
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Number;

    fn collect_tokens(input: &str) -> Vec<TokenResult> {
        StrReader::new(input).collect()
    }

    fn tokens_ok(input: &str) -> Vec<Token> {
        collect_tokens(input)
            .into_iter()
            .map(|r| r.map(|s| s.value).map_err(|e| e.value))
            .collect::<core::result::Result<Vec<Token>, Error>>()
            .unwrap()
    }

    fn parse(input: &str) -> Value {
        StrReader::new(input).into_value().unwrap()
    }

    // --- Tokenizer tests ---

    #[test]
    fn test_empty_input() {
        assert!(collect_tokens("").is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        assert!(collect_tokens("   \t\n  ").is_empty());
    }

    #[test]
    fn test_structural_chars() {
        let tokens = tokens_ok(":,;");
        assert_eq!(tokens, vec![Token::Colon, Token::Comma, Token::SemiColon]);
    }

    #[test]
    fn test_keywords() {
        let tokens = tokens_ok("null true false");
        assert_eq!(tokens, vec![Token::Null, Token::True, Token::False]);
    }

    #[test]
    fn test_numbers_basic() {
        let tokens = tokens_ok("42 -7 +3 3.14 -0.5 1e10 2.5E-3 0xFF 0x1a");
        assert_eq!(
            tokens,
            vec![
                Token::Number("42".into()),
                Token::Number("-7".into()),
                Token::Number("+3".into()),
                Token::Number("3.14".into()),
                Token::Number("-0.5".into()),
                Token::Number("1e10".into()),
                Token::Number("2.5E-3".into()),
                Token::Number("0xFF".into()),
                Token::Number("0x1a".into()),
            ]
        );
    }

    #[test]
    fn test_special_numbers() {
        let tokens = tokens_ok("nan inf +inf -inf");
        assert_eq!(
            tokens,
            vec![
                Token::Number("nan".into()),
                Token::Number("inf".into()),
                Token::Number("+inf".into()),
                Token::Number("-inf".into()),
            ]
        );
    }

    #[test]
    fn test_leading_zeros() {
        let tokens = tokens_ok("007 01.5 1e+007");
        assert_eq!(
            tokens,
            vec![
                Token::Number("007".into()),
                Token::Number("01.5".into()),
                Token::Number("1e+007".into()),
            ]
        );
    }

    #[test]
    fn test_unquoted_strings() {
        let tokens = tokens_ok("hello world foo-bar test_123");
        assert_eq!(
            tokens,
            vec![
                Token::String("hello".into()),
                Token::String("world".into()),
                Token::String("foo-bar".into()),
                Token::String("test_123".into()),
            ]
        );
    }

    #[test]
    fn test_quoted_string_basic() {
        let tokens = tokens_ok(r#""hello world""#);
        assert_eq!(tokens, vec![Token::String("hello world".into())]);
    }

    #[test]
    fn test_quoted_string_escapes() {
        let tokens = tokens_ok(r#""line1\nline2\ttab\\backslash\"quote\/slash""#);
        assert_eq!(
            tokens,
            vec![Token::String("line1\nline2\ttab\\backslash\"quote/slash".into())]
        );
    }

    #[test]
    fn test_hex_escape() {
        let tokens = tokens_ok(r#""\x41\x42""#);
        assert_eq!(tokens, vec![Token::String("AB".into())]);
    }

    #[test]
    fn test_unicode_escape_fixed() {
        let tokens = tokens_ok(r#""\u0041""#);
        assert_eq!(tokens, vec![Token::String("A".into())]);
    }

    #[test]
    fn test_unicode_escape_braced() {
        let tokens = tokens_ok(r#""\u{41}\u{1F600}""#);
        assert_eq!(tokens, vec![Token::String("A\u{1F600}".into())]);
    }

    #[test]
    fn test_unclosed_string_error() {
        let results = collect_tokens(r#""unclosed"#);
        assert!(matches!(
            results[0],
            Err(ref e) if e.value == Error::UnfinishedString
        ));
    }

    #[test]
    fn test_invalid_escape_error() {
        let results = collect_tokens(r#""\q""#);
        assert!(matches!(
            results[0],
            Err(ref e) if e.value == Error::InvalidEscape
        ));
    }

    #[test]
    fn test_incomplete_hex_escape_error() {
        let results = collect_tokens(r#""\x1""#);
        assert!(matches!(
            results[0],
            Err(ref e) if e.value == Error::InvalidEscape
        ));
    }

    #[test]
    fn test_span_tracking() {
        let tokens = collect_tokens("hello");
        let spanned = tokens[0].as_ref().unwrap();
        assert_eq!(spanned.span.line, 1);
        assert_eq!(spanned.span.column_start, 1);
        assert_eq!(spanned.span.column_end, 6);
    }

    #[test]
    fn test_span_multiline() {
        let tokens = collect_tokens("a\nb");
        assert_eq!(tokens[0].as_ref().unwrap().span.line, 1);
        assert_eq!(tokens[1].as_ref().unwrap().span.line, 2);
    }

    // --- Parser (into_value) tests ---

    #[test]
    fn test_parse_null() {
        assert_eq!(parse("null"), Value::Null);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse("true"), Value::Boolean(true));
        assert_eq!(parse("false"), Value::Boolean(false));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse("hello"), Value::String("hello".into()));
    }

    #[test]
    fn test_parse_quoted_string() {
        assert_eq!(
            parse(r#""hello world""#),
            Value::String("hello world".into())
        );
    }

    #[test]
    fn test_parse_number_integer() {
        assert_eq!(parse("42"), Value::Number(Number::PosInt(42)));
    }

    #[test]
    fn test_parse_number_negative() {
        let result = parse("-7");
        match result {
            Value::Number(Number::NegInt(n)) => {
                // NegInt uses offset encoding; verify round-trip
                assert_eq!(n, (-7i64 as u64).wrapping_sub(1));
            }
            other => panic!("expected NegInt, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_number_float() {
        assert_eq!(parse("3.14"), Value::Number(Number::Float(3.14)));
    }

    #[test]
    fn test_parse_number_hex() {
        assert_eq!(parse("0xFF"), Value::Number(Number::PosInt(255)));
    }

    #[test]
    fn test_parse_nan() {
        assert_eq!(parse("nan"), Value::Number(Number::NaN));
    }

    #[test]
    fn test_parse_inf() {
        assert_eq!(parse("inf"), Value::Number(Number::Inf { negative: false }));
        assert_eq!(parse("-inf"), Value::Number(Number::Inf { negative: true }));
    }

    #[test]
    fn test_parse_leading_zeros() {
        assert_eq!(parse("007"), Value::Number(Number::PosInt(7)));
        assert_eq!(parse("01.5"), Value::Number(Number::Float(1.5)));
    }

    #[test]
    fn test_parse_empty_vector() {
        assert_eq!(parse(":;"), Value::Vector(vec![]));
    }

    #[test]
    fn test_parse_vector() {
        assert_eq!(
            parse(":a,b;"),
            Value::Vector(vec![
                Value::String("a".into()),
                Value::String("b".into()),
            ])
        );
    }

    #[test]
    fn test_parse_empty_map() {
        assert_eq!(parse(";"), Value::Map(Map::new()));
    }

    #[test]
    fn test_parse_map() {
        let mut expected = Map::new();
        expected.insert("name".into(), Value::String("twic".into()));
        expected.insert("version".into(), Value::Number(Number::Float(0.1)));
        assert_eq!(parse("name:twic,version:0.1;"), Value::Map(expected));
    }

    #[test]
    fn test_parse_nested() {
        let mut inner = Map::new();
        inner.insert("name".into(), Value::String("twic".into()));
        inner.insert("version".into(), Value::Number(Number::Float(0.1)));

        let mut outer = Map::new();
        outer.insert("profile".into(), Value::Map(inner));
        outer.insert(
            "users".into(),
            Value::Vector(vec![
                Value::String("alice".into()),
                Value::String("bob".into()),
            ]),
        );

        assert_eq!(
            parse("profile:name:twic,version:0.1;,users::alice,bob;;"),
            Value::Map(outer)
        );
    }

    #[test]
    fn test_parse_trailing_comma_error() {
        let result = StrReader::new(":a,;").into_value();
        assert!(matches!(result, Err(Error::TrailingComma)));
    }

    #[test]
    fn test_parse_missing_semicolon_error() {
        let result = StrReader::new(":a,b").into_value();
        assert!(result.is_err());
    }

    #[test]
    fn test_a() {
        let result = StrReader::new(":a,b 0").into_value();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unexpected_trailing_error() {
        let result = StrReader::new("null;").into_value();
        assert!(result.is_err());
    }

    // --- Integration: full Twic snippets ---

    #[test]
    fn test_example_from_spec() {
        let result = parse("msg:hello!,from:twic;");
        let mut expected = Map::new();
        expected.insert("msg".into(), Value::String("hello!".into()));
        expected.insert("from".into(), Value::String("twic".into()));
        assert_eq!(result, Value::Map(expected));
    }

    #[test]
    fn test_mixed_types_in_vector() {
        let result = parse(":null,true,false,42,hello;");
        assert_eq!(
            result,
            Value::Vector(vec![
                Value::Null,
                Value::Boolean(true),
                Value::Boolean(false),
                Value::Number(Number::PosInt(42)),
                Value::String("hello".into()),
            ])
        );
    }

    #[test]
    fn test_string_with_escapes() {
        let result = parse(r#""tab\there\nnewline""#);
        assert_eq!(result, Value::String("tab\there\nnewline".into()));
    }

    #[test]
    fn test_deep_nesting() {
        let result = parse(":a:1,b::x,y;;;");
        let mut inner_map = Map::new();
        inner_map.insert("a".into(), Value::Number(Number::PosInt(1)));
        inner_map.insert(
            "b".into(),
            Value::Vector(vec![
                Value::String("x".into()),
                Value::String("y".into()),
            ]),
        );
        assert_eq!(result, Value::Vector(vec![Value::Map(inner_map)]));
    }
}
