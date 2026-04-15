# Round 2 Summary

## What Was Done

Addressed all 5 issues from Codex code review:

### P1 (Critical): NegInt encoding inconsistency
Both hex and decimal paths had incorrect NegInt offset encoding. The `Number::NegInt(n)` type uses `n = u64::MAX - |val| + 1`. Fixed both paths to use the correct encoding: hex uses `val.wrapping_neg()`, decimal uses `val as u64` (which wraps correctly for negative i64). Added `test_negative_hex_encoding_matches_decimal` to verify `-0x1 == -1`.

### P2 (Medium): Wrong error for trailing tokens
`into_value` returned `Error::TrailingComma` for any trailing token, even when no comma was present. Changed to `Error::UnexpectedToken { expected: "end of input", found: token_name(&token) }` in both StrReader and StdTokenReader impls.

### P4 (Medium): Missing StdTokenReader tests
Added 6 tests behind `#[cfg(feature = "std")]`: empty input, token parity with StrReader, parse null, parse map, span tracking, malformed input.

### P5 (Low): Map trailing comma inconsistent
`a:1,;` returned `UnexpectedToken` instead of `TrailingComma`. Added check for `SemiColon` after consuming Comma in `parse_map_with_first_key`, returning `TrailingComma` consistently with vector behavior.

### P8 (Trivial): Non-descriptive test name
Renamed `test_a` to `test_parse_vector_non_semicolon_terminator`.

### Additional fix: no_std build
Added `alloc::borrow::ToOwned` and `alloc::vec` imports needed for `--no-default-features` compilation.

## Files Changed

- `src/read/mod.rs` — All fixes + new tests (52 total unit tests)
- `src/read/span.rs` — (unchanged from Round 1)

## Validation

- `cargo test`: 52 unit tests passed, 0 failed, 0 warnings
- `cargo test --doc`: 122 doc tests passed, 0 failed
- `cargo test --no-default-features`: Compiles and passes (46 unit tests, StdTokenReader tests excluded)
- All Codex P1-P5, P8 issues resolved

## BitLesson Delta

Action: none
Lesson ID(s): NONE
Notes: All review findings addressed.
