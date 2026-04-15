# Round 5 Summary

## What Was Done

Addressed both issues from Codex Round 4 review: span info not surfaced to callers, and Spanned Display bypass.

### Span info surfaced through parser

Changed `parse_str` return type from `Result<Value, Error>` to `Result<Value, Spanned<Error>>` so callers receive position information on all errors.

Key changes:
- `Parser::try_read_token` now returns `Result<Option<Spanned<Token>>, Spanned<Error>>` (preserves spans)
- `Parser::peek_token` returns `Result<Option<Token>, Spanned<Error>>` (cloned token with span in errors)
- `Parser::read_token` and `read_token_of` return `Result<Spanned<Token>, Spanned<Error>>`
- Added `Parser::current_span()` method using `CharReaderState` position for EOF errors
- All parser-generated errors (`UnexpectedToken`, `UnexpectedEof`) now carry the originating token's span
- `parse_number` errors carry the number token's span

### Spanned<T> Display fix

`Spanned<T>::Display` now delegates to `Span::Display` instead of printing only `line:column_start`. Multiline spans show `line:col-end_line:col` format.

### std::error::Error for Spanned

Added `impl<T: std::error::Error> std::error::Error for Spanned<T>` behind `#[cfg(feature = "std")]` so `parse_read` can wrap `Spanned<Error>` into `std::io::Error`.

### New tests

- `test_error_carries_span`: verifies unclosed string error has non-zero line/column
- `test_unexpected_token_carries_span`: verifies parser-generated error has valid span

## Files Changed

- `src/read/mod.rs` — Parser refactored to preserve spans, 56 tests
- `src/read/span.rs` — Spanned Display fix, std::error::Error impl

## Validation

- `cargo test`: 56 unit tests passed, 0 failed, 0 warnings
- `cargo test --no-default-features`: 52 unit tests passed, 0 warnings
- `cargo test --doc`: 122 doc tests passed
- Both Codex Round 4 issues resolved

## BitLesson Delta

Action: none
Lesson ID(s): NONE
Notes: Span info now fully end-to-end from tokenizer through parser to caller.
