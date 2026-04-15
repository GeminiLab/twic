use core::fmt;

/// Source position: line number and column range, both 1-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Line number (1-based).
    pub line: usize,
    /// Start column (1-based, inclusive).
    pub column_start: usize,
    /// End column (1-based, exclusive).
    pub column_end: usize,
}

impl Span {
    pub fn new(line: usize, column_start: usize, column_end: usize) -> Self {
        Self {
            line,
            column_start,
            column_end,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}-{}",
            self.line, self.column_start, self.column_end
        )
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
        write!(f, "{} @ {}:{}", self.value, self.span.line, self.span.column_start)
    }
}
