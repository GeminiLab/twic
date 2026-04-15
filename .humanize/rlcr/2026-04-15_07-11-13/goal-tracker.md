# Goal Tracker

<!--
This file tracks the ultimate goal, acceptance criteria, and plan evolution.
It prevents goal drift by maintaining a persistent anchor across all rounds.

RULES:
- IMMUTABLE SECTION: Do not modify after initialization
- MUTABLE SECTION: Update each round, but document all changes
- Every task must be in one of: Active, Completed, or Deferred
- Deferred items require explicit justification
-->

## IMMUTABLE SECTION
<!-- Do not modify after initialization -->

### Ultimate Goal

Implement a complete tokenizer (state machine) and recursive descent parser (`TokenRead::into_value`) for the Twic data serialization format in the `twic-new` Rust crate. This includes: expanding the `Token` enum to carry complete data with span information, adding proper error types with span information, creating a generic internal tokenizer module shared by `StrReader` and `StdTokenReader`, implementing `TokenRead::into_value`, and generating comprehensive test cases that verify all Twic types against their expected `Value` representations.

## Acceptance Criteria

### Acceptance Criteria
<!-- Each criterion must be independently verifiable -->
<!-- Claude must extract or define these in Round 0 -->


Following TDD philosophy, each criterion includes positive and negative tests for deterministic verification.

- AC-1: Test cases exist as Twic snippets paired with expected `Value` representations, covering all six data types and their edge cases
  - Positive Tests (expected to PASS):
    - Each primitive type (null, true, false, integers, floats, hex numbers, special numbers) has at least one test case producing the correct `Value`
    - Numbers with leading zeros in decimal integers (e.g., `007`), in the integer part of floats (e.g., `01.5`), and in the exponent part (e.g., `1e+007`) are correctly parsed
    - Strings (unquoted, quoted, with escape sequences including `\uXXXX`, `\u{X...}`, `\xXX`) produce correct `Value::String`
    - Vectors (`:elements;`), empty vectors (`:;`), and nested vectors produce correct `Value::Vector`
    - Maps (`key:value;`), empty maps (`;`), and nested maps produce correct `Value::Map`
    - Whitespace between tokens is correctly ignored
  - Negative Tests (expected to FAIL):
    - Unclosed quoted string (e.g., `"unclosed`) produces an error
    - Invalid escape sequence (e.g., `"\q"`) produces an error
    - Incomplete hex escape (e.g., `"\x1"`) produces an error
    - Incomplete unicode escape produces an error

- AC-2: The `Token` enum carries complete data for all token types
  - Positive Tests (expected to PASS):
    - `Token::Null` variant exists and is produced for the `null` keyword
    - `Token::String(String)` variant exists and carries decoded string content (not raw source text)
    - `Token::Number(String)` carries the raw number text for deferred parsing
    - `Token::True`, `Token::False` exist for boolean keywords
    - `Token::Colon`, `Token::Comma`, `Token::SemiColon` exist for structural tokens
  - Negative Tests (expected to FAIL):

---

## MUTABLE SECTION
<!-- Update each round with justification for changes -->

### Plan Version: 1 (Updated: Round 0)

#### Plan Evolution Log
<!-- Document any changes to the plan with justification -->
| Round | Change | Reason | Impact on AC |
|-------|--------|--------|--------------|
| 0 | Initial plan | - | - |

#### Active Tasks
<!-- Map each task to its target Acceptance Criterion and routing tag -->
| Task | Target AC | Status | Tag | Owner | Notes |
|------|-----------|--------|-----|-------|-------|
| task1: Add Span struct and Spanned<T> wrapper | AC-2.1, AC-3.1 | completed | coding | claude | Foundation for position tracking |
| task2: Expand Token enum (Null, String(String)) | AC-2 | completed | coding | claude | Depends on task1 |
| task3: Change Iterator::Item to Result<Spanned<Token>, Spanned<Error>> | AC-2.1, AC-3.1 | completed | coding | claude | Depends on task1, task2 |
| task4: Expand error.rs with specific error variants | AC-3 | completed | coding | claude | Depends on task1 |
| task5: Generate primitive type test cases | AC-1 | completed | coding | claude | Depends on task1, task4 |
| task6: Generate composite type and error test cases | AC-1 | completed | coding | claude | Depends on task1, task4 |
| task7: Implement tokenizer state machine with CharReader trait | AC-4 | completed | coding | claude | Depends on task1, task4 |
| task8: Implement StrReader using internal tokenizer | AC-5 | completed | coding | claude | Depends on task7 |
| task9: Implement StdTokenReader using internal tokenizer | AC-6 | completed | coding | claude | Depends on task7 |
| task10: Implement TokenRead::into_value parser | AC-7 | completed | coding | claude | Depends on task1, task4 |
| task11: Run all tests via cargo test | AC-8 | completed | analyze | claude | Depends on task5-10 |

### Completed and Verified
<!-- Only move tasks here after Codex verification -->
| AC | Task | Completed Round | Verified Round | Evidence |
|----|------|-----------------|----------------|----------|
| AC-2.1, AC-3.1 | task1: Span/Spanned types | 0 | 0 | src/read/span.rs, tests pass |
| AC-2 | task2: Token enum expansion | 0 | 0 | src/read/mod.rs:22-31 |
| AC-2.1, AC-3.1 | task3: TokenResult type alias | 0 | 0 | src/read/mod.rs:18 |
| AC-3 | task4: Error enum expansion | 0 | 0 | src/error.rs |
| AC-1 | task5-6: Test cases (43 tests) | 0 | 0 | src/read/mod.rs tests module |
| AC-4 | task7: CharReader + tokenizer | 0 | 0 | src/read/lexer.rs |
| AC-5 | task8: StrReader | 0 | 0 | src/read/mod.rs:55-116 |
| AC-6 | task9: StdTokenReader | 0 | 0 | src/read/mod.rs (std feature) |
| AC-7 | task10: into_value parser | 0 | 0 | src/read/mod.rs Parser struct |
| AC-8 | task11: All tests pass | 0 | 0 | cargo test: 43+122 pass, 0 fail |

### Explicitly Deferred
<!-- Items here require strong justification -->
| Task | Original AC | Deferred Since | Justification | When to Reconsider |
|------|-------------|----------------|---------------|-------------------|

### Open Issues
<!-- Issues discovered during implementation -->
| Issue | Discovered Round | Blocking AC | Resolution Path |
|-------|-----------------|-------------|-----------------|
