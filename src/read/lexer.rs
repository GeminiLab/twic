use alloc::string::String;

use crate::error::Error;

use super::span::{Span, Spanned};
use super::Token;

/// Private trait abstracting character-level source access with position tracking.
///
/// `StrReader` and `StdTokenReader` each provide their own implementation,
/// feeding characters into the shared tokenizer state machine.
pub(crate) trait CharReader {
    /// Returns the next character, or `None` at end of input.
    fn next_char(&mut self) -> Option<char>;
    /// Peeks at the next character without consuming it, or `None` at end of input.
    fn peek_char(&mut self) -> Option<char>;
    /// Current line number (1-based).
    fn current_line(&self) -> usize;
    /// Current column number (1-based). Points to the position of the next char.
    fn current_column(&self) -> usize;
}

/// Reads the next token from a [`CharReader`].
///
/// Returns `None` at end of input. On success, returns a `Spanned<Token>`.
/// On failure, returns a `Spanned<Error>`.
pub(crate) fn tokenize_next(reader: &mut dyn CharReader) -> Option<Result<Spanned<Token>, Spanned<Error>>> {
    // Skip whitespace
    loop {
        match reader.peek_char() {
            Some(c) if c.is_whitespace() => {
                reader.next_char();
            }
            Some(_) => break,
            None => return None,
        }
    }

    let start_line = reader.current_line();
    let start_col = reader.current_column();

    let c = reader.next_char().unwrap();

    let result = match c {
        ',' => Ok(Token::Comma),
        ':' => Ok(Token::Colon),
        ';' => Ok(Token::SemiColon),
        '"' => read_quoted_string(reader),
        _ => read_unquoted_number_or_keyword(reader, c),
    };

    let end_col = reader.current_column();
    let span = Span::new(start_line, start_col, end_col);

    Some(match result {
        Ok(token) => Ok(Spanned::new(token, span)),
        Err(error) => Err(Spanned::new(error, span)),
    })
}

fn read_quoted_string(reader: &mut dyn CharReader) -> Result<Token, Error> {
    let mut result = String::new();

    loop {
        match reader.next_char() {
            None => return Err(Error::UnfinishedString),
            Some('"') => break,
            Some('\\') => {
                let c = reader.next_char().ok_or(Error::UnfinishedString)?;
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
                        let hex = read_hex_digits(reader, 2)?;
                        result.push(
                            core::char::from_u32(hex).ok_or(Error::InvalidEscape)?,
                        );
                    }
                    'u' => {
                        let c = reader.peek_char().ok_or(Error::UnfinishedString)?;
                        let ch = if c == '{' {
                            reader.next_char(); // consume '{'
                            read_braced_unicode(reader)?
                        } else {
                            read_hex_char(reader, 4)?
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

fn read_braced_unicode(reader: &mut dyn CharReader) -> Result<char, Error> {
    let mut hex = 0u32;
    let mut count = 0u32;
    loop {
        match reader.peek_char() {
            Some('}') => {
                reader.next_char();
                if count == 0 || count > 8 {
                    return Err(Error::InvalidEscape);
                }
                return core::char::from_u32(hex).ok_or(Error::InvalidEscape);
            }
            Some(c) if c.is_ascii_hexdigit() => {
                reader.next_char();
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

fn read_hex_char(reader: &mut dyn CharReader, len: usize) -> Result<char, Error> {
    let hex = read_hex_digits(reader, len)?;
    core::char::from_u32(hex).ok_or(Error::InvalidEscape)
}

fn read_hex_digits(reader: &mut dyn CharReader, len: usize) -> Result<u32, Error> {
    let mut hex = 0u32;
    for _ in 0..len {
        let c = reader.next_char().ok_or(Error::UnfinishedString)?;
        if !c.is_ascii_hexdigit() {
            return Err(Error::InvalidEscape);
        }
        hex = hex * 16 + (c.to_digit(16).unwrap() as u32);
    }
    Ok(hex)
}

fn read_unquoted_number_or_keyword(
    reader: &mut dyn CharReader,
    first: char,
) -> Result<Token, Error> {
    let mut buffer = String::new();
    buffer.push(first);

    loop {
        match reader.peek_char() {
            Some(c) if c.is_whitespace() || c == ',' || c == ':' || c == ';' => break,
            Some(c) => {
                reader.next_char();
                buffer.push(c);
            }
            None => break,
        }
    }

    // Check keywords first
    match buffer.as_str() {
        "null" => return Ok(Token::Null),
        "true" => return Ok(Token::True),
        "false" => return Ok(Token::False),
        _ => {}
    }

    // Check if it looks like a number:
    // - starts with digit, +, -
    // - is one of the special number keywords: nan, inf
    let is_number = buffer.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '-')
        || buffer == "nan"
        || buffer == "inf";

    if is_number {
        Ok(Token::Number(buffer))
    } else {
        Ok(Token::String(buffer))
    }
}
