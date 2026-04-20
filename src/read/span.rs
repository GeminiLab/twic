use core::fmt;
use core::error::Error;

/// Source position: line number and character column range, both 1-based.
///
/// Columns count Unicode code points (characters), not bytes. For example,
/// in the string `"éx"`, the character `'é'` is at column 1 and `'x'` is at
/// column 2, regardless of how many bytes `'é'` occupies in UTF-8.
///
/// For single-line tokens, `line` and `end_line` are equal. For multiline
/// tokens (e.g., quoted strings containing newlines), `end_line` records
/// the line where the token ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start line number (1-based).
    pub line: usize,
    /// Start column (1-based, inclusive). Counts Unicode characters, not bytes.
    pub column_start: usize,
    /// End line number (1-based).
    pub end_line: usize,
    /// End column (1-based, exclusive). Counts Unicode characters, not bytes.
    pub column_end: usize,
}

impl Span {
    pub fn new(line: usize, column_start: usize, end_line: usize, column_end: usize) -> Self {
        Self {
            line,
            column_start,
            end_line,
            column_end,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == self.end_line {
            write!(f, "{}:{}-{}", self.line, self.column_start, self.column_end)
        } else {
            write!(
                f,
                "{}:{}-{}:{}",
                self.line, self.column_start, self.end_line, self.column_end
            )
        }
    }
}

/// A value paired with its source [`Span`].
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.value, self.span)
    }
}

impl<T: Error> Error for Spanned<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Error::source(&self.value)
    }

    fn description(&self) -> &str {
        #[expect(deprecated)]
        Error::description(&self.value)
    }

    fn cause(&self) -> Option<&dyn Error> {
        #[expect(deprecated)]
        Error::cause(&self.value)
    }
}
