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
pub(crate) trait CharReader {
    /// Returns the next character, or `None` at end of input.
    fn next_char(&mut self) -> Option<char>;

    /// Advances past any whitespace, returning the first non-whitespace
    /// character or `None` at end of input.
    #[allow(dead_code)]
    fn next_non_whitespace_char(&mut self) -> Option<char> {
        loop {
            match self.next_char() {
                Some(c) if c.is_whitespace() => continue,
                Some(c) => return Some(c),
                None => return None,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CharReaderState — adds peeking and position tracking
// ---------------------------------------------------------------------------

/// Wraps a [`CharReader`] and adds single-character peek and line/column tracking.
pub(crate) struct CharReaderState<R: CharReader> {
    reader: R,
    peeked: Option<Option<char>>,
    line: usize,
    column: usize,
}

impl<R: CharReader> CharReaderState<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            peeked: None,
            line: 1,
            column: 1,
        }
    }

    pub fn peek_char(&mut self) -> Option<char> {
        if let Some(peeked) = self.peeked {
            peeked
        } else {
            let c = self.reader.next_char();
            self.peeked = Some(c);
            c
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
        let c = if let Some(peeked) = self.peeked.take() {
            peeked
        } else {
            self.reader.next_char()
        };
        if let Some(c) = c {
            self.advance_line_col(c);
        }
        c
    }

    fn next_non_whitespace_char(&mut self) -> Option<char> {
        loop {
            match self.next_char() {
                Some(c) if c.is_whitespace() => continue,
                Some(c) => return Some(c),
                None => return None,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Concrete CharReader implementations
// ---------------------------------------------------------------------------

/// `CharReader` backed by `&str` via `CharIndices`.
pub(crate) struct StrCharReader<'a> {
    chars: core::str::CharIndices<'a>,
}

impl<'a> StrCharReader<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.char_indices(),
        }
    }
}

impl<'a> CharReader for StrCharReader<'a> {
    fn next_char(&mut self) -> Option<char> {
        self.chars.next().map(|(_, c)| c)
    }
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

/// Reads the next token from a [`CharReaderState`].
///
/// Returns `None` at end of input. On success, returns a `Spanned<Token>`.
/// On failure, returns a `Spanned<Error>`.
pub(crate) fn tokenize_next<R: CharReader>(
    state: &mut CharReaderState<R>,
) -> Option<Result<Spanned<Token>, Spanned<Error>>> {
    // Skip whitespace without consuming non-whitespace
    loop {
        match state.peek_char() {
            Some(c) if c.is_whitespace() => {
                state.next_char();
            }
            Some(_) => break,
            None => return None,
        }
    }

    // Position is now at the first character of the token
    let start_line = state.current_line();
    let start_col = state.current_column();

    let c = state.next_char()?;

    let result = match c {
        ',' => Ok(Token::Comma),
        ':' => Ok(Token::Colon),
        ';' => Ok(Token::SemiColon),
        '"' => read_quoted_string(state),
        _ => read_unquoted_number_or_keyword(state, c),
    };

    let end_col = state.current_column();
    let span = Span::new(start_line, start_col, end_col);

    Some(match result {
        Ok(token) => Ok(Spanned::new(token, span)),
        Err(error) => Err(Spanned::new(error, span)),
    })
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
                hex = hex * 16 + (c.to_digit(16).unwrap() as u32);
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
        hex = hex * 16 + (c.to_digit(16).unwrap() as u32);
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
