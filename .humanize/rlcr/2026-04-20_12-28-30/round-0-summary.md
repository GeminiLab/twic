# Round 0 Summary — Self-Review

## What Was Implemented

Self-review of existing codebase against CQ-1 through CQ-7 criteria, plus Codex feedback resolution.

### Bugs Found and Fixed

1. **`std::error::Error` references in no_std build** (`src/read/span.rs:69-83`):
   The `impl<T: Error> Error for Spanned<T>` block used `dyn std::error::Error` in method
   signatures for `source()` and `cause()`. Failed under `--no-default-features`.
   Fix: Changed to `dyn Error` (resolves to `core::error::Error`).

2. **`ParseError` (née `Error`) not implementing `core::error::Error` unconditionally** (`src/error.rs`):
   The parse error type only implemented `std::error::Error` behind `#[cfg(feature = "std")]`,
   meaning `Spanned<ParseError>` could not implement `Error` in no_std mode.
   Fix: Renamed `Error` to `ParseError` to avoid name collision with `core::error::Error` trait,
   and implemented `core::error::Error` unconditionally via `use core::error::Error`.

3. **`\r` and `\r\n` not handled as line breaks in span tracking** (`src/read/lexer.rs`):
   `advance_line_col` only treated `\n` as a line break, but `is_whitespace()` considers
   `\r` as whitespace. This produced incorrect line numbers for CRLF or CR input.
   Fix: Replaced `advance_line_col` with inline logic in `next_char` that handles `\r`,
   `\r\n` (counted as one line break), `\n`, `\u{2028}`, and `\u{2029}`.
   Added `last_was_cr: bool` field to `CharReaderState` for `\r\n` detection.

### CQ Self-Review Results

| Criterion | Status | Evidence |
|-----------|--------|----------|
| CQ-1: No dead code, zero warnings | PASS | 0 warnings under both std and no_std |
| CQ-2: No duplicate logic | PASS | `text_no_sign` reused; `accumulate_hex` shared |
| CQ-3: Minimal type complexity | PASS | No nested Option/Result; peeked_char + has_peeked |
| CQ-4: No unnecessary abstraction | PASS | CharReader trait justified by generic Parser design |
| CQ-5: Correctness over cleverness | PASS | Edge cases tested: leading zeros, neg hex, -0, u64::MAX, i64::MIN, CRLF spans |
| CQ-6: API consistency | PASS | parse_str → Result<Value, Spanned<ParseError>>; parse_read → Result<Value, ReadError> |
| CQ-7: Spec compliance | PASS | All 6 Twic types; vectors, maps, nested structures |

## Files Changed

- `src/error.rs` — Renamed `Error` to `ParseError`, unconditional `core::error::Error` impl
- `src/read/span.rs` — Fixed `dyn std::error::Error` → `dyn Error` for no_std compatibility
- `src/read/lexer.rs` — Renamed to `ParseError`, added `\r`/`\r\n` line break handling, added `last_was_cr` field
- `src/read/mod.rs` — Renamed to `ParseError`, added CRLF span tests

## Validation

- **std**: `cargo test` — 60 unit tests PASS, 122 doc tests PASS, 0 warnings
- **no_std**: `cargo test --no-default-features` — 55 unit tests PASS, 122 doc tests PASS, 0 warnings

## Remaining Items

None. All acceptance criteria (AC-1 through AC-8, CQ-1 through CQ-7) are satisfied.

## BitLesson Delta

Action: none
Lesson ID(s): NONE
Notes: Self-review round; no new pattern emerged worth a lesson entry.
