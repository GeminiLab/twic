mod lexer;
mod span;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::error::Error;
use crate::value::{Map, Number, Value};

pub use span::{Span, Spanned};

use lexer::{CharReader, CharReaderState, StrCharReader};

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Public API: parse_str / parse_read
// ---------------------------------------------------------------------------

/// Parses a Twic `&str` input into a [`Value`].
pub fn parse_str(input: &str) -> Result<Value, Spanned<Error>> {
    let reader = StrCharReader::new(input);
    parse(reader)
}

/// Parses Twic input from a [`std::io::Read`] stream into a [`Value`].
#[cfg(feature = "std")]
pub fn parse_read<R: std::io::Read>(mut reader: R) -> std::io::Result<Value> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    match parse_str(&buf) {
        Ok(value) => Ok(value),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
    }
}

// ---------------------------------------------------------------------------
// Internal: parse<R: CharReader>
// ---------------------------------------------------------------------------

fn parse<R: CharReader>(reader: R) -> Result<Value, Spanned<Error>> {
    let mut parser = Parser::new(reader);
    let value = parser.parse_value()?;
    // Ensure no trailing tokens
    if let Some(spanned) = parser.try_read_token()? {
        return Err(Spanned::new(
            Error::UnexpectedToken {
                expected: "end of input",
                found: token_name(&spanned.value),
            },
            spanned.span,
        ));
    }
    Ok(value)
}

// ---------------------------------------------------------------------------
// Parser<R: CharReader>
// ---------------------------------------------------------------------------

struct Parser<R: CharReader> {
    state: CharReaderState<R>,
    peeked: Option<Spanned<Token>>,
}

impl<R: CharReader> Parser<R> {
    fn new(reader: R) -> Self {
        Self {
            state: CharReaderState::new(reader),
            peeked: None,
        }
    }

    /// Returns a Span at the current reader position (for EOF errors).
    fn current_span(&self) -> Span {
        Span::new(
            self.state.current_line(),
            self.state.current_column(),
            self.state.current_line(),
            self.state.current_column(),
        )
    }

    // -- Token reading methods --

    /// Reads the next token. Returns `Ok(None)` at end of input.
    fn try_read_token(&mut self) -> Result<Option<Spanned<Token>>, Spanned<Error>> {
        if let Some(spanned) = self.peeked.take() {
            return Ok(Some(spanned));
        }
        match lexer::tokenize_next(&mut self.state) {
            None => Ok(None),
            Some(Ok(spanned)) => Ok(Some(spanned)),
            Some(Err(spanned)) => Err(spanned),
        }
    }

    /// Reads the next token and validates it with a predicate.
    /// Returns `Ok(None)` at end of input.
    fn try_read_token_of(
        &mut self,
        pred: impl Fn(&Token) -> bool,
        expected: &'static str,
    ) -> Result<Option<Spanned<Token>>, Spanned<Error>> {
        match self.try_read_token()? {
            Some(spanned) if pred(&spanned.value) => Ok(Some(spanned)),
            Some(spanned) => Err(Spanned::new(
                Error::UnexpectedToken {
                    expected,
                    found: token_name(&spanned.value),
                },
                spanned.span,
            )),
            None => Ok(None),
        }
    }

    /// Reads the next token. Returns error on end of input.
    fn read_token(&mut self, expected: &'static str) -> Result<Spanned<Token>, Spanned<Error>> {
        self.try_read_token()?
            .ok_or(Spanned::new(Error::UnexpectedEof { expected }, self.current_span()))
    }

    /// Reads the next token and validates it with a predicate.
    /// Returns error on end of input.
    fn read_token_of(
        &mut self,
        pred: impl Fn(&Token) -> bool,
        expected: &'static str,
    ) -> Result<Spanned<Token>, Spanned<Error>> {
        self.try_read_token_of(pred, expected)?
            .ok_or(Spanned::new(Error::UnexpectedEof { expected }, self.current_span()))
    }

    /// Peeks at the next token without consuming it.
    fn peek_token(&mut self) -> Result<Option<Token>, Spanned<Error>> {
        if let Some(ref spanned) = self.peeked {
            return Ok(Some(spanned.value.clone()));
        }
        match lexer::tokenize_next(&mut self.state) {
            None => Ok(None),
            Some(Ok(spanned)) => {
                let token = spanned.value.clone();
                self.peeked = Some(spanned);
                Ok(Some(token))
            }
            Some(Err(spanned)) => Err(spanned),
        }
    }

    // -- Parsing methods --

    fn parse_value(&mut self) -> Result<Value, Spanned<Error>> {
        let spanned = self.read_token("value")?;

        match spanned.value {
            Token::Null => Ok(Value::Null),
            Token::True => Ok(Value::Boolean(true)),
            Token::False => Ok(Value::Boolean(false)),
            Token::Number(s) => parse_number(&s).map_err(|e| Spanned::new(e, spanned.span)),
            Token::Colon => self.parse_vector(),
            Token::SemiColon => Ok(Value::Map(Map::new())),
            Token::String(s) => {
                match self.peek_token()? {
                    Some(Token::Colon) => {
                        self.try_read_token()?; // consume the Colon
                        self.parse_map_with_first_key(s)
                    }
                    _ => Ok(Value::String(s)),
                }
            }
            Token::Comma => Err(Spanned::new(
                Error::UnexpectedToken {
                    expected: "value",
                    found: ",",
                },
                spanned.span,
            )),
        }
    }

    fn parse_vector(&mut self) -> Result<Value, Spanned<Error>> {
        let mut vec = Vec::new();

        if !matches!(self.peek_token()?, Some(Token::SemiColon)) {
            vec.push(self.parse_value()?);
            while matches!(self.peek_token()?, Some(Token::Comma)) {
                self.try_read_token()?; // consume Comma
                vec.push(self.parse_value()?);
            }
        }

        self.read_token_of(|t| matches!(t, Token::SemiColon), ";")?;
        Ok(Value::Vector(vec))
    }

    fn parse_map_with_first_key(&mut self, first_key: String) -> Result<Value, Spanned<Error>> {
        let mut map = Map::new();
        let value = self.parse_value()?;
        map.insert(first_key, value);

        while matches!(self.peek_token()?, Some(Token::Comma)) {
            self.try_read_token()?; // consume Comma

            let key_spanned = self.read_token("string key")?;
            let key = match key_spanned.value {
                Token::String(s) => s,
                other => {
                    return Err(Spanned::new(
                        Error::UnexpectedToken {
                            expected: "string key",
                            found: token_name(&other),
                        },
                        key_spanned.span,
                    ));
                }
            };

            self.read_token_of(|t| matches!(t, Token::Colon), ":")?;
            let value = self.parse_value()?;
            map.insert(key, value);
        }

        self.read_token_of(|t| matches!(t, Token::SemiColon), ";")?;
        Ok(Value::Map(map))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn parse_number(text: &str) -> Result<Value, Error> {
    match text {
        "nan" => return Ok(Value::Number(Number::NaN)),
        "inf" | "+inf" => return Ok(Value::Number(Number::Inf { negative: false })),
        "-inf" => return Ok(Value::Number(Number::Inf { negative: true })),
        _ => {}
    }

    let text_no_sign = text.strip_prefix(['+', '-']).unwrap_or(text);
    let negative = text.starts_with('-');

    if let Some(hex_str) = text_no_sign.strip_prefix("0x") {
        let val = u64::from_str_radix(hex_str, 16)
            .map_err(|_| Error::InvalidNumber(text.to_owned()))?;
        if negative && val != 0 {
            if val > i64::MIN.unsigned_abs() {
                return Err(Error::InvalidNumber(text.to_owned()));
            }
            Ok(Value::Number(Number::NegInt(val.wrapping_neg())))
        } else {
            Ok(Value::Number(Number::PosInt(val)))
        }
    } else if text.contains('.') || text.contains('e') || text.contains('E') {
        let val: f64 = text
            .parse()
            .map_err(|_| Error::InvalidNumber(text.to_owned()))?;
        Ok(Value::Number(Number::Float(val)))
    } else if text.starts_with('-') {
        let val: i64 = text
            .parse()
            .map_err(|_| Error::InvalidNumber(text.to_owned()))?;
        if val < 0 {
            Ok(Value::Number(Number::NegInt(val as u64)))
        } else {
            // -0
            Ok(Value::Number(Number::PosInt(0)))
        }
    } else {
        let text_unsigned = text.strip_prefix('+').unwrap_or(text);
        let val: u64 = text_unsigned
            .parse()
            .map_err(|_| Error::InvalidNumber(text.to_owned()))?;
        Ok(Value::Number(Number::PosInt(val)))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;
    use crate::value::Number;

    fn parse(input: &str) -> Value {
        parse_str(input).unwrap()
    }

    fn collect_tokens(input: &str) -> Vec<Result<Spanned<Token>, Spanned<Error>>> {
        let reader = StrCharReader::new(input);
        let mut state = CharReaderState::new(reader);
        let mut tokens = Vec::new();
        while let Some(result) = lexer::tokenize_next(&mut state) {
            tokens.push(result);
        }
        tokens
    }

    fn tokens_ok(input: &str) -> Vec<Token> {
        collect_tokens(input)
            .into_iter()
            .map(|r| r.map(|s| s.value).map_err(|e| e.value))
            .collect::<Result<Vec<Token>, Error>>()
            .unwrap()
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
            vec![Token::String(
                "line1\nline2\ttab\\backslash\"quote/slash".into()
            )]
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

    // --- Parser tests ---

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
                assert_eq!(n, (-7i64) as u64);
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
            Value::Vector(vec![Value::String("a".into()), Value::String("b".into()),])
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
    fn test_parse_vector_comma_empty_map() {
        // :1,; is a vector with 1 and an empty map, terminated by ;
        assert_eq!(
            parse(":1,;;"),
            Value::Vector(vec![
                Value::Number(Number::PosInt(1)),
                Value::Map(Map::new()),
            ])
        );
    }

    #[test]
    fn test_parse_vector_nested_vectors() {
        // ::1,2;,:3,4;; is a vector of two vectors
        assert_eq!(
            parse("::1,2;,:3,4;;"),
            Value::Vector(vec![
                Value::Vector(vec![
                    Value::Number(Number::PosInt(1)),
                    Value::Number(Number::PosInt(2)),
                ]),
                Value::Vector(vec![
                    Value::Number(Number::PosInt(3)),
                    Value::Number(Number::PosInt(4)),
                ]),
            ])
        );
    }

    #[test]
    fn test_parse_missing_semicolon_error() {
        let result = parse_str(":a,b");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_vector_non_semicolon_terminator() {
        let result = parse_str(":a,b 0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unexpected_trailing_error() {
        let result = parse_str("null;");
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
            Value::Vector(vec![Value::String("x".into()), Value::String("y".into())]),
        );
        assert_eq!(result, Value::Vector(vec![Value::Map(inner_map)]));
    }

    #[test]
    fn test_negative_hex_encoding_matches_decimal() {
        let hex_result = parse("-0x1");
        let dec_result = parse("-1");
        assert_eq!(hex_result, dec_result);
    }

    #[test]
    fn test_negative_zero_hex_matches_decimal() {
        let hex_result = parse("-0x0");
        let dec_result = parse("-0");
        assert_eq!(hex_result, dec_result);
        assert_eq!(hex_result, Value::Number(Number::PosInt(0)));
    }

    #[test]
    fn test_large_positive_decimal() {
        // i64::MAX + 1 should parse as PosInt(u64)
        let val = parse("9223372036854775808");
        assert_eq!(val, Value::Number(Number::PosInt(9223372036854775808)));
        // u64::MAX
        let val = parse("18446744073709551615");
        assert_eq!(val, Value::Number(Number::PosInt(u64::MAX)));
    }

    #[test]
    fn test_negative_hex_out_of_range() {
        // -0x8000000000000001 is beyond i64::MIN and should error
        let result = parse_str("-0x8000000000000001");
        assert!(result.is_err());
        // -0x8000000000000000 == i64::MIN, should succeed
        let result = parse_str("-0x8000000000000000");
        assert!(result.is_ok());
    }

    #[test]
    fn test_span_multiline_quoted_string() {
        let tokens = collect_tokens("\"a\nb\"");
        let spanned = tokens[0].as_ref().unwrap();
        assert_eq!(spanned.span.line, 1);
        assert_eq!(spanned.span.end_line, 2);
        assert_eq!(spanned.span.column_start, 1);
    }

    #[test]
    fn test_error_carries_span() {
        let result = parse_str(r#""unclosed"#);
        let err = result.unwrap_err();
        assert_eq!(err.span.line, 1);
        assert_eq!(err.span.column_start, 1);
        assert!(err.span.line > 0);
        assert!(err.span.column_start > 0);
    }

    #[test]
    fn test_unexpected_token_carries_span() {
        let result = parse_str(":a,b 0");
        let err = result.unwrap_err();
        assert!(err.span.line > 0);
        assert!(err.span.column_start > 0);
    }

    #[test]
    fn test_parse_positive_signed_integer() {
        assert_eq!(parse("+3"), Value::Number(Number::PosInt(3)));
        assert_eq!(parse("+0"), Value::Number(Number::PosInt(0)));
    }

    // --- parse_read tests (std feature) ---

    #[cfg(feature = "std")]
    mod std_tests {
        use super::*;
        use std::io::Cursor;

        #[test]
        fn test_parse_read_null() {
            let cursor = Cursor::new(b"null");
            assert_eq!(parse_read(cursor).unwrap(), Value::Null);
        }

        #[test]
        fn test_parse_read_map() {
            let cursor = Cursor::new(b"msg:hello!,from:twic;");
            let mut expected = Map::new();
            expected.insert("msg".into(), Value::String("hello!".into()));
            expected.insert("from".into(), Value::String("twic".into()));
            assert_eq!(parse_read(cursor).unwrap(), Value::Map(expected));
        }

        #[test]
        fn test_parse_read_empty() {
            // Empty input is not valid Twic (needs at least one value)
            let cursor = Cursor::new(b"");
            assert!(parse_read(cursor).is_err());
        }

        #[test]
        fn test_parse_read_parity() {
            let input = "name:twic,version:1,items::a,b;;";
            let str_result = parse_str(input).unwrap();
            let read_result = parse_read(Cursor::new(input.as_bytes())).unwrap();
            assert_eq!(str_result, read_result);
        }
    }
}
