# Round 1 Summary

## What Was Done

Addressed two issues found by Codex code review of Round 0:

### Issue 1 (High): Unsound `StdTokenReader` with `unsafe` raw pointer cast

**Problem**: `StdTokenReader` stored a `Peekable<Chars<'static>>` created via an unsafe raw pointer cast from a `Box<String>`. This was a classic self-referential struct pattern — moving the struct would invalidate the internal borrow, causing undefined behavior.

**Fix**: Replaced with a fully safe design. `StdTokenReader` now stores:
- `buf: Box<str>` — the buffered input string
- `pos: usize` — current byte offset into the buffer
- `next_char()` reads `buf[pos..].chars().next()` and advances by `c.len_utf8()`
- `peek_char()` reads `buf[pos..].chars().next()` without advancing

No `unsafe` code remains anywhere in the crate.

### Issue 2 (Medium): Column semantics documentation

**Problem**: Span column tracking was not clearly documented as counting Unicode characters vs bytes.

**Fix**: Added documentation to the `Span` struct clarifying that columns count Unicode code points (characters), not bytes. The implementation was already correct — `CharIndices` yields `(byte_offset, char)`, and we increment column by 1 per `char` regardless of UTF-8 byte width.

## Files Changed

- `src/read/mod.rs` — Replaced `StdTokenReader` with safe position-based `CharReader` implementation
- `src/read/span.rs` — Added Unicode column semantics documentation to `Span` struct

## Validation

- `cargo test`: 44 unit tests passed, 0 failed, 0 warnings
- `cargo test --doc`: 122 doc tests passed, 0 failed
- Clean build with zero warnings, zero unsafe blocks

## Remaining Items

None — all tasks complete, both Codex issues resolved.

## BitLesson Delta

Action: add
Lesson ID(s): BL-20260415-unsafe-self-ref
Notes: Self-referential struct pattern (storing an iterator that borrows from a co-stored buffer) is unsound in Rust. Moving the struct invalidates the borrow. Use position-based indexing into the buffer instead.
