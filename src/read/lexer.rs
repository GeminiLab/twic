use alloc::string::String;

use crate::error::Error;

use super::span::{Span, Spanned};
use super::Token;

// ---------------------------------------------------------------------------
// CharReader trait
// ---------------------------------------------------------------------------

/// Core trait for character-level source access.
///
/// Implementations provide raw character access from different sources
/// (`&str`, `std::io::Read`). The trait is kept minimal; peeking and
/// position tracking are handled by [`CharReaderState`].
///
/// There are two concrete implementations:
/// - [`StrCharReader`] — backed by `Chars<'_>` from a `&str` slice.
/// - [`CharReaderState`] — wraps any `CharReader` and adds peek/position
///   tracking (i.e., it implements `CharReader` itself via delegation).
pub(crate) trait CharReader {
    /// Returns the next character, or `None` at end of input.
    fn next_char(&mut self) -> Option<char>;
}

// ---------------------------------------------------------------------------
// CharReaderState — adds peeking and position tracking
// ---------------------------------------------------------------------------

/// Wraps a [`CharReader`] and adds single-character peek and line/column tracking.
pub(crate) struct CharReaderState<R: CharReader> {
    reader: R,
    peeked_char: Option<char>,
    has_peeked: bool,
    line: usize,
    column: usize,
}

impl<R: CharReader> CharReaderState<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            peeked_char: None,
            has_peeked: false,
            line: 1,
            column: 1,
        }
    }

    pub fn peek_char(&mut self) -> Option<char> {
        if !self.has_peeked {
            self.peeked_char = self.reader.next_char();
            self.has_peeked = true;
        }
        self.peeked_char
    }

    /// Skips whitespace characters without consuming the first non-whitespace
    /// character. After this returns, `peek_char()` returns the first
    /// non-whitespace character (or `None` at end of input).
    pub fn skip_whitespace(&mut self) {
        while self.peek_char().is_some_and(|c| c.is_whitespace()) {
            self.next_char();
        }
    }

    pub fn current_line(&self) -> usize {
        self.line
    }

    pub fn current_column(&self) -> usize {
        self.column
    }

    fn advance_line_col(&mut self, c: char) {
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }
}

impl<R: CharReader> CharReader for CharReaderState<R> {
    fn next_char(&mut self) -> Option<char> {
        let c = if self.has_peeked {
            self.has_peeked = false;
            self.peeked_char
        } else {
            self.reader.next_char()
        };
        if let Some(c) = c {
            self.advance_line_col(c);
        }
        c
    }
}

// ---------------------------------------------------------------------------
// Concrete CharReader implementations
// ---------------------------------------------------------------------------

/// `CharReader` backed by `&str` via `Chars`.
pub(crate) struct StrCharReader<'a> {
    chars: core::str::Chars<'a>,
}

impl<'a> StrCharReader<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars(),
        }
    }
}

impl<'a> CharReader for StrCharReader<'a> {
    fn next_char(&mut self) -> Option<char> {
        self.chars.next()
    }
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

/// Reads the next token from a [`CharReaderState`].
///
/// Returns `Ok(None)` at end of input. On success, returns
/// `Ok(Some(Spanned<Token>))`. On failure, returns `Err(Spanned<Error>)`.
pub(crate) fn tokenize_next<R: CharReader>(
    state: &mut CharReaderState<R>,
) -> Result<Option<Spanned<Token>>, Spanned<Error>> {
    state.skip_whitespace();

    // Position is now at the first character of the token
    let start_line = state.current_line();
    let start_col = state.current_column();

    let c = match state.next_char() {
        Some(c) => c,
        None => return Ok(None),
    };

    let result = match c {
        ',' => Ok(Token::Comma),
        ':' => Ok(Token::Colon),
        ';' => Ok(Token::SemiColon),
        '"' => read_quoted_string(state),
        _ => read_unquoted_number_or_keyword(state, c),
    };

    let end_line = state.current_line();
    let end_col = state.current_column();
    let span = Span::new(start_line, start_col, end_line, end_col);

    match result {
        Ok(token) => Ok(Some(Spanned::new(token, span))),
        Err(error) => Err(Spanned::new(error, span)),
    }
}

fn read_quoted_string<R: CharReader>(
    state: &mut CharReaderState<R>,
) -> Result<Token, Error> {
    let mut result = String::new();

    loop {
        match state.next_char() {
            None => return Err(Error::UnfinishedString),
            Some('"') => break,
            Some('\\') => {
                let c = state.next_char().ok_or(Error::UnfinishedString)?;
                match c {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\u{0008}'),
                    'f' => result.push('\u{000C}'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    'x' => {
                        let hex = read_hex_digits(state, 2)?;
                        result.push(core::char::from_u32(hex).ok_or(Error::InvalidEscape)?);
                    }
                    'u' => {
                        let pc = state.peek_char().ok_or(Error::UnfinishedString)?;
                        let ch = if pc == '{' {
                            state.next_char(); // consume '{'
                            read_braced_unicode(state)?
                        } else {
                            read_hex_char(state, 4)?
                        };
                        result.push(ch);
                    }
                    _ => return Err(Error::InvalidEscape),
                }
            }
            Some(c) => result.push(c),
        }
    }

    Ok(Token::String(result))
}

/// Accumulates a single hex digit into the running value.
/// Returns `hex * 16 + digit_value(c)`. Caller must ensure `c` is ascii hex.
fn accumulate_hex(hex: u32, c: char) -> u32 {
    hex * 16 + (c.to_digit(16).unwrap() as u32)
}

fn read_braced_unicode<R: CharReader>(
    state: &mut CharReaderState<R>,
) -> Result<char, Error> {
    let mut hex = 0u32;
    let mut count = 0u32;
    loop {
        match state.peek_char() {
            Some('}') => {
                state.next_char();
                if count == 0 || count > 8 {
                    return Err(Error::InvalidEscape);
                }
                return core::char::from_u32(hex).ok_or(Error::InvalidEscape);
            }
            Some(c) if c.is_ascii_hexdigit() => {
                state.next_char();
                count += 1;
                if count > 8 {
                    return Err(Error::InvalidEscape);
                }
                hex = accumulate_hex(hex, c);
            }
            _ => return Err(Error::InvalidEscape),
        }
    }
}

fn read_hex_char<R: CharReader>(
    state: &mut CharReaderState<R>,
    len: usize,
) -> Result<char, Error> {
    let hex = read_hex_digits(state, len)?;
    core::char::from_u32(hex).ok_or(Error::InvalidEscape)
}

fn read_hex_digits<R: CharReader>(
    state: &mut CharReaderState<R>,
    len: usize,
) -> Result<u32, Error> {
    let mut hex = 0u32;
    for _ in 0..len {
        let c = state.next_char().ok_or(Error::UnfinishedString)?;
        if !c.is_ascii_hexdigit() {
            return Err(Error::InvalidEscape);
        }
        hex = accumulate_hex(hex, c);
    }
    Ok(hex)
}

fn read_unquoted_number_or_keyword<R: CharReader>(
    state: &mut CharReaderState<R>,
    first: char,
) -> Result<Token, Error> {
    let mut buffer = String::new();
    buffer.push(first);

    loop {
        match state.peek_char() {
            Some(c) if c.is_whitespace() || c == ',' || c == ':' || c == ';' => break,
            Some(c) => {
                state.next_char();
                buffer.push(c);
            }
            None => break,
        }
    }

    match buffer.as_str() {
        "null" => return Ok(Token::Null),
        "true" => return Ok(Token::True),
        "false" => return Ok(Token::False),
        _ => {}
    }

    let is_number = buffer.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '-')
        || buffer == "nan"
        || buffer == "inf";

    if is_number {
        Ok(Token::Number(buffer))
    } else {
        Ok(Token::String(buffer))
    }
}
