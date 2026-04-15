Read and execute below with ultrathink

## Goal Tracker Setup (REQUIRED FIRST STEP)

Before starting implementation, you MUST initialize the Goal Tracker:

1. Read @/home/aarkegz/source/repos/twic/twic-new/.humanize/rlcr/2026-04-15_16-02-37/goal-tracker.md
2. If the "Ultimate Goal" section says "[To be extracted...]", extract a clear goal statement from the plan
3. If the "Acceptance Criteria" section says "[To be defined...]", define 3-7 specific, testable criteria
4. Populate the "Active Tasks" table with tasks from the plan, mapping each to an AC and filling Tag/Owner
5. Write the updated goal-tracker.md

**IMPORTANT**: The IMMUTABLE SECTION can only be modified in Round 0. After this round, it becomes read-only.

---

## Implementation Plan

For all tasks that need to be completed, please use the Task system (TaskCreate, TaskUpdate, TaskList) to track each item in order of importance.
You are strictly prohibited from only addressing the most important issues - you MUST create Tasks for ALL discovered issues and attempt to resolve each one.

## Task Tag Routing (MUST FOLLOW)

Each task must have one routing tag from the plan: `coding` or `analyze`.

- Tag `coding`: Claude executes the task directly.
- Tag `analyze`: Claude must execute via `/humanize:ask-codex`, then integrate Codex output.
- Keep Goal Tracker "Active Tasks" columns **Tag** and **Owner** aligned with execution (`coding -> claude`, `analyze -> codex`).
- If a task has no explicit tag, default to `coding` (Claude executes directly).

# Twic Tokenizer and Test Implementation

## Goal Description

Implement a complete tokenizer (state machine) and recursive descent parser (`TokenRead::into_value`) for the Twic data serialization format in the `twic-new` Rust crate. This includes: expanding the `Token` enum to carry complete data with span information, adding proper error types with span information, creating a generic internal tokenizer module shared by `StrReader` and `StdTokenReader`, implementing `TokenRead::into_value`, and generating comprehensive test cases that verify all Twic types against their expected `Value` representations.

## Acceptance Criteria

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
    - A keyword like `null` must NOT be tokenized as `Token::String`
    - A number must NOT be tokenized as `Token::String`
    - An unquoted string starting with a digit must NOT be tokenized as `Token::String`
  - AC-2.1: Tokens carry span information
    - Positive: Each token produced by the tokenizer includes a span with line number (1-based) and column range (1-based)
    - Positive: The first token in input `"hello"` has span at line 1, column 1
    - Positive: Tokens on lines after newlines have correct line numbers
    - Negative: Span line/column values are never zero

- AC-3: Error types support specific tokenizer and parser error reporting
  - Positive Tests (expected to PASS):
    - Specific error variants exist for: unfinished string, invalid escape sequence, unexpected token, unexpected end of input, invalid number
    - `Iterator::next()` returns `Option<Result<Spanned<Token>, Spanned<Error>>>` enabling error propagation with position info
  - Negative Tests (expected to FAIL):
    - `Error::Unknown` is not used for any identifiable error condition
    - The error type does not use `thiserror` or any external dependency
  - AC-3.1: Errors carry span information
    - Positive: Each error includes a span with line number (1-based) and column range (1-based)
    - Positive: An unclosed string error reports the line and column where the opening quote appeared
    - Negative: Error span values are never zero

- AC-4: Core tokenizer state machine handles all Twic syntax defined in the formal specification
  - Positive Tests (expected to PASS):
    - Whitespace skipping (Unicode whitespace) works between any tokens
    - Structural characters (`:`, `;`, `,`) are correctly tokenized
    - Quoted strings with all escape sequences (`\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t`, `\uXXXX`, `\u{X...}`, `\xXX`) are decoded
    - Unquoted strings respect the spec rules (no `:;,` content, no digit/quote/sign start, not a keyword)
    - Numbers (decimal int/float, hex `0x`, special `nan`/`inf`/`+inf`/`-inf`) are recognized
    - Leading zeros are accepted in decimal integers, in the integer part of floats, and in the exponent part of floats
    - Keywords `null`, `true`, `false` are recognized as distinct from unquoted strings
  - Negative Tests (expected to FAIL):
    - A quoted string with `\U` (old-spec 8-digit escape) is rejected as invalid escape
    - A number with leading or trailing decimal point (e.g., `.5`, `5.`) is rejected
    - An unquoted string equal to a keyword like `nan` is tokenized as a number, not a string

- AC-5: `StrReader` correctly tokenizes `&str` input into a stream of `Result<Spanned<Token>, Spanned<Error>>`
  - Positive Tests (expected to PASS):
    - `StrReader::new("input")` creates a reader from a string slice
    - `Iterator::next()` produces the correct token sequence for a given input
    - Empty input returns `None` immediately
    - Input with only whitespace returns `None`
    - Complex nested structures produce correct token sequences
    - Produced tokens carry correct span (line and column) information
  - Negative Tests (expected to FAIL):
    - Malformed input (unclosed quote, bad escape) returns `Some(Err(...))` with span pointing to the error location, not `None`

- AC-6: `StdTokenReader` correctly tokenizes `std::io::Read` streams (behind `#[cfg(feature = "std")]`)
  - Positive Tests (expected to PASS):
    - A `std::io::Cursor<&[u8]>` with valid Twic produces the same tokens as `StrReader`
    - Empty stream returns `None`
    - Produced tokens carry correct span (line and column) information
  - Negative Tests (expected to FAIL):
    - Malformed stream data returns `Some(Err(...))` with span info, not `None`

- AC-7: `TokenRead::into_value` converts a token stream into a `Value` tree
  - Positive Tests (expected to PASS):
    - `null` produces `Value::Null`
    - `true`/`false` produce `Value::Boolean(true)`/`Value::Boolean(false)`
    - Numbers produce `Value::Number` with correct `Number` variant (`PosInt`, `NegInt`, `Float`, `NaN`, `Inf`)
    - Numbers with leading zeros (e.g., `007`, `01.5`, `1e+007`) parse to the correct `Number` value
    - Strings produce `Value::String`
    - `:a,b;` produces `Value::Vector` with two elements
    - `a:1,b:2;` produces `Value::Map` with two entries
    - Deeply nested structures (maps in vectors, vectors in maps) produce correct `Value` trees
    - Empty vector `:;` produces `Value::Vector(vec![])`
    - Empty map `;` at top level produces `Value::Map(Map::new())`
  - Negative Tests (expected to FAIL):
    - Trailing comma without element (e.g., `:a,;`) produces a parse error
    - Missing closing semicolon on a vector or map produces a parse error
    - A value followed by unexpected tokens produces a parse error

- AC-8: All tests pass via `cargo test` in the `twic-new` crate
  - Positive Tests (expected to PASS):
    - `cargo test` exits with code 0
    - Doc tests continue to pass
    - Tests pass with and without the `std` feature
  - Negative Tests (expected to FAIL):
    - (Meta-criterion; no negative test applicable)

## Path Boundaries

Path boundaries define the acceptable range of implementation quality and choices.

### Upper Bound (Maximum Acceptable Scope)

The implementation includes a fully shared generic tokenizer module with a clean internal state machine, a `Span` struct and `Spanned<T>` wrapper for line/column (1-based) position tracking on tokens and errors, comprehensive error types with specific variants, both `StrReader` and `StdTokenReader` fully implemented and tested, `TokenRead::into_value` with a complete recursive descent parser, and a thorough test suite with many Twic/Value test pairs covering edge cases, escape sequences, nested structures, leading zeros, and error conditions. The `error.rs` module has well-named variants for each identifiable error condition.

### Lower Bound (Minimum Acceptable Scope)

The implementation includes a `Span` struct (line, column range, 1-based) and `Spanned<T>` wrapper, the minimum `Token` enum fixes (add `Null`, change `String` to carry data), basic error types with span info sufficient to distinguish the main failure modes, an internal tokenizer module shared between `StrReader` and `StdTokenReader`, working `StrReader`, `StdTokenReader`, and `into_value` implementations, and test cases covering each Twic type at least once with both valid and invalid inputs.

### Allowed Choices

- Can use: `alloc` crate (`String`, `Vec`, `BTreeMap`), `core::str::Chars<'_>` for `StrCharReader`, `core::char::from_u32` for escape decoding, internal buffering for `StdTokenReader`'s `CharReader`
- Can use: a private `CharReader` trait in the internal tokenizer module, with `StrReader` and `StdTokenReader` each providing their own implementation
- Can use: a generic `Spanned<T>` wrapper struct or embedding span data directly into Token/Error variants
- Cannot use: external dependencies (zero-dependency constraint)
- Cannot use: `unsafe` code (unless clearly justified and documented)
- Cannot use: `Error::Unknown` for any identifiable error condition

> **Note on Deterministic Designs**: The Twic formal specification defines a precise grammar with no ambiguity. The tokenizer must follow this grammar exactly. The path boundaries differ only in implementation quality (error richness, test thoroughness), not in syntactic behavior.

## Feasibility Hints and Suggestions

> **Note**: This section is for reference and understanding only. These are conceptual suggestions, not prescriptive requirements.

### Conceptual Approach

**Span and Spanned Types**: Define a `Span` struct tracking line number and column range (both 1-based), and a `Spanned<T>` generic wrapper that pairs a value with its source span:

```
struct Span { line: usize, column_start: usize, column_end: usize }
struct Spanned<T> { value: T, span: Span }
```

The tokenizer produces `Result<Spanned<Token>, Spanned<Error>>` — both successful tokens and errors carry position information. The `into_value` parser can optionally use span info for better error messages.

**CharReader Abstraction**: The internal tokenizer module defines a private `CharReader` trait for character-level access, and a `CharReaderState<R: CharReader>` wrapper that adds peek and line/column tracking:

```
trait CharReader {
    fn next_char(&mut self) -> Option<char>;
}

struct CharReaderState<R: CharReader> { ... }
// provides: next_char, peek_char, skip_whitespace, current_line, current_column
```

`StrCharReader` implements `CharReader` backed by `Chars<'_>` on a `&str`. `CharReaderState` wraps any `CharReader` and provides peeking and position tracking. The tokenizer operates on `CharReaderState<R>`.

**Internal Tokenizer State Machine**: The core tokenizer operates on any `CharReader` implementation, reading characters one at a time and producing `Result<Spanned<Token>, Spanned<Error>>` values. The key states are:

1. **Whitespace skip**: consume whitespace, track line/column position, transition to dispatch
2. **Dispatch**: look at next char: `,`/`:`/`;` → structural token, `"` → quoted string, digit/`+`/`-` → number-or-keyword-or-string, other → unquoted-string-or-keyword
3. **Quoted string**: consume chars until `"`, handling `\` escape sequences inline
4. **Unquoted/number/keyword**: consume chars until whitespace/`,/:/;/`, then classify: check if keyword (`null`/`true`/`false`/`nan`/`inf`), then check if number (parse pattern), else unquoted string
5. **Done**: input exhausted

**into_value Parser**: A recursive descent parser that consumes tokens from the `TokenRead` iterator:

```
parse_value():
  match next_token:
    Null => Value::Null
    True => Value::Boolean(true)
    False => Value::Boolean(false)
    Number(s) => parse_number(s) => Value::Number(...)
    String(s) => Value::String(s)
    Colon => parse_vector()  // ':' starts a vector
    _ => if at end, parse as map; else error

parse_vector():
  vec = []
  if peek != SemiColon:
    vec.push(parse_value())
    while peek == Comma:
      consume Comma
      vec.push(parse_value())
  consume SemiColon
  return Value::Vector(vec)

parse_map():
  map = Map::new()
  if peek != SemiColon:
    key = consume String
    consume Colon
    value = parse_value()
    map.insert(key, value)
    while peek == Comma:
      consume Comma
      key = consume String
      consume Colon
      value = parse_value()
      map.insert(key, value)
  consume SemiColon
  return Value::Map(map)
```

**Number Parsing in into_value**: The `Token::Number(String)` raw text needs to be parsed into the `Number` enum:
- `nan` → `Number::NaN`
- `inf`/`+inf` → `Number::Inf { negative: false }`
- `-inf` → `Number::Inf { negative: true }`
- Hex (`0x...`) → parse as u64 → `Number::PosInt` or `Number::NegInt`
- Decimal integer → parse as i64 → `Number::PosInt` or `Number::NegInt`
- Decimal float → parse as f64 → `Number::Float`

Note: Leading zeros are valid per the Twic spec for decimal integers and for the integer and exponent parts of floating-point numbers. Rust's `str::parse::<i64>()` and `str::parse::<f64>()` handle leading zeros naturally, so no special stripping is needed.

### Relevant References

- `twic-new/src/read.rs` - Target file for tokenizer and parser implementation
- `twic-new/src/value.rs` - `Value` enum definition (fully implemented)
- `twic-new/src/value/number.rs` - `Number` enum with conversion methods
- `twic-new/src/error.rs` - Error type to be expanded
- `twic-new/README.md` - Twic formal specification (grammar, syntax rules)
- `twic-old/src/parser/tokenizer.rs` - Reference tokenizer implementation (old spec, `\U` escapes differ)
- `twic-old/src/parser/test/tokenizer.rs` - Reference tokenizer tests
- `twic-old/src/parser/span.rs` - Reference `Spanned<T>` implementation

## Dependencies and Sequence

### Milestones

1. **Foundation**: Expand `Token` enum, `Span`/`Spanned` types, and `Error` types to support complete tokenization with position tracking
   - Phase A: Add `Span` struct (line, column_start, column_end, all 1-based) and `Spanned<T>` wrapper type
   - Phase B: Add `Token::Null` variant, change `Token::String` to `Token::String(String)`, change `Iterator::Item` to `Result<Spanned<Token>, Spanned<Error>>`
   - Phase C: Expand `error.rs` with specific error variants (InvalidEscape, UnfinishedString, InvalidNumber, UnexpectedToken, UnexpectedEof)

2. **Test Cases**: Generate comprehensive Twic snippet / expected `Value` test pairs
   - Phase A: Create test cases for primitive types (null, bool, numbers including leading zeros, strings)
   - Phase B: Create test cases for composite types (vectors, maps, nesting)
   - Phase C: Create test cases for error conditions (malformed input, span verification)

3. **Tokenizer**: Implement the core tokenizer state machine and readers
   - Phase A: Create internal tokenizer module with `CharReader` trait and shared state machine logic
   - Phase B: Implement `StrReader`'s `CharReader` (backed by `Peekable<CharIndices<'_>>`) and wire it to the state machine
   - Phase C: Implement `StdTokenReader`'s `CharReader` (backed by `std::io::Read` with internal buffering) and wire it to the state machine (behind `#[cfg(feature = "std")]`)

4. **Parser**: Implement `TokenRead::into_value`
   - Phase A: Implement recursive descent parser for all value types
   - Phase B: Implement number parsing from `Token::Number(String)` raw text

5. **Verification**: Run all tests and confirm passing
   - Phase A: Run test cases against implementation, fix any failures
   - Phase B: Verify `cargo test` passes with and without `std` feature

### Dependencies

- Milestone 1 has no dependencies (foundation)
- Milestone 2 depends on Milestone 1 (tests reference Token, Spanned, and Error types)
- Milestone 3 depends on Milestone 1 (tokenizer uses Token, Spanned, and Error types)
- Milestone 4 depends on Milestone 1 (parser uses Token and Error types)
- Milestone 3 and Milestone 4 are independent of each other (tokenizer produces tokens, parser consumes them)
- Milestone 5 depends on Milestones 2, 3, and 4 (all must be complete to verify)

## Task Breakdown

Each task must include exactly one routing tag:
- `coding`: implemented by Claude
- `analyze`: executed via Codex (`/humanize:ask-codex`)

| Task ID | Description | Target AC | Tag (`coding`/`analyze`) | Depends On |
|---------|-------------|-----------|--------------------------|------------|
| task1 | Add `Span` struct (line/column, 1-based) and `Spanned<T>` wrapper type | AC-2.1, AC-3.1 | coding | - |
| task2 | Expand `Token` enum: add `Null` variant, change `String` to `String(String)` | AC-2 | coding | task1 |
| task3 | Change `Iterator::Item` to `Result<Spanned<Token>, Spanned<Error>>` for both `StrReader` and `StdTokenReader` | AC-2.1, AC-3.1 | coding | task1, task2 |
| task4 | Expand `error.rs` with specific error variants | AC-3 | coding | task1 |
| task5 | Generate test cases for primitive types (null, bool, all number forms including leading zeros, quoted/unquoted strings with escapes) | AC-1 | coding | task1, task4 |
| task6 | Generate test cases for composite types (vectors, maps, nesting) and error conditions | AC-1 | coding | task1, task4 |
| task7 | Implement internal tokenizer state machine module with `CharReader` trait (whitespace skip, dispatch, quoted string, unquoted/number/keyword classification, position tracking) | AC-4 | coding | task1, task4 |
| task8 | Implement `StrReader` using the internal tokenizer module | AC-5 | coding | task7 |
| task9 | Implement `StdTokenReader` using the internal tokenizer module (behind `#[cfg(feature = "std")]`) | AC-6 | coding | task7 |
| task10 | Implement `TokenRead::into_value` recursive descent parser | AC-7 | coding | task1, task4 |
| task11 | Run all tests via `cargo test` and verify passing | AC-8 | analyze | task5, task6, task8, task9, task10 |

## Claude-Codex Deliberation

### Agreements

- The current `Token` enum is incomplete: `Null` variant and string payload are required
- `Error` types must be expanded beyond `Error::Unknown`
- A generic internal tokenizer module with a private `CharReader` trait is the right architecture for sharing logic between `StrReader` and `StdTokenReader`
- Zero external dependencies is a hard constraint
- The old tokenizer (`twic-old/`) provides useful reference but uses a different spec (`\U` escapes vs `\u{X...}`)
- Tokens and errors must carry span information (line number and column range, both 1-based)
- Leading zeros are valid in decimal integers and in the integer and exponent parts of floating-point numbers per the Twic specification

### Resolved Disagreements

- **Error handling design**: Claude identified that `Iterator<Item = Token>` cannot report errors. User decided to change the signature to `Iterator<Item = Result<Token, Error>>`, matching the old tokenizer's approach.
- **Generic tokenizer design**: User confirmed a private `CharReader` trait in the internal tokenizer module, with `StrReader` and `StdTokenReader` each providing their own `CharReader` implementation (not a public trait, not a buffer-everything approach).
- **Token enum expansion**: User confirmed adding `Null` and changing `String` to carry payload data.
- **Span tracking**: User requested span info on tokens and errors with line (1-based) and column range (1-based).
- **Leading zero rules**: User updated the spec to explicitly allow leading zeros in float integer and exponent parts; research confirmed this matches the BNF grammar.

### Convergence Status

- Final Status: `converged` (all user decisions resolved, no pending items)

## Pending User Decisions

None. All design decisions were resolved during planning and refinement.

## Code Quality Review Criteria (MANDATORY for Codex reviewer)

The Codex reviewer MUST check ALL of the following during every review round. Any violation is a finding. There is no "non-blocking" category — every finding must be fixed before COMPLETE.

### CQ-1: No Dead Code
- Every `pub`/`pub(crate)` item must be used by at least one call site (test or production code).
- Every `#[allow(dead_code)]` annotation must have a comment explaining why the code is intentionally kept.
- Private items that are genuinely unused must be removed.
- Zero compiler warnings allowed (`cargo test` must produce 0 warnings).

### CQ-2: No Duplicate Logic
- If two code paths do the same thing, extract a shared helper. Do NOT copy-paste logic.
- If a helper exists for a purpose, use it. Do NOT reimplement its logic inline.
- Trait default methods must be actually used, or the method must not exist.
- Every `match` arm pattern that duplicates another must be scrutinized.

### CQ-3: Minimal Type Complexity
- No `Option<Option<...>>`, no `Option<Result<...>>`, no nested wrappers beyond one level.
- If a type signature requires a comment to explain its nesting, it is too complex.
- State fields should store the simplest representation that supports all operations.

### CQ-4: No Unnecessary Abstraction
- Do not introduce traits, structs, or type parameters that are not needed by the current implementation.
- Every generic parameter must have at least two concrete realizations (or a clear, documented future plan).
- Do not wrap types unnecessarily — if `X` and `Wrapper<X>` have the same API surface, remove the wrapper.

### CQ-5: Correctness Over Cleverness
- Code must be obviously correct. If a reviewer has to reason about wrapping arithmetic, bit manipulation, or unsafe casts, the code needs a comment or a simpler approach.
- Number encoding edge cases (`-0`, `u64::MAX`, `i64::MIN`, `0x8000000000000000`) must be explicitly tested.
- Parse/encode round-trips must be verified by tests.

### CQ-6: API Consistency
- All public functions must have consistent error types. If `fn A()` returns `Result<T, E>` and `fn B()` wraps `A()` but returns `Result<T, OtherE>`, the relationship must be documented.
- Public types must be constructable and usable without reaching into internal modules.
- `Display` implementations must show all meaningful fields — do not silently drop data.

### CQ-7: Spec Compliance
- The implementation must match the formal specification in README.md exactly.
- Every production rule in the BNF grammar must be exercised by at least one test.
- Any deviation from the spec (e.g., accepting inputs the spec forbids, or rejecting inputs the spec allows) is a bug.

### Review Severity Scale
- **[P0]**: Crash, data corruption, security vulnerability, soundness bug
- **[P1]**: Incorrect behavior (wrong parse result, wrong span, spec violation)
- **[P2]**: Dead code, duplicate logic, unnecessary complexity, missing edge case test
- **[P3]**: Style inconsistency, unclear naming, missing documentation on non-obvious logic

All P0-P2 findings must be fixed. P3 findings should be fixed but are not blocking if justified.

## Implementation Notes

### Code Style Requirements
- Implementation code and comments must NOT contain plan-specific terminology such as "AC-", "Milestone", "Step", "Phase", or similar workflow markers
- These terms are for plan documentation only, not for the resulting codebase
- Use descriptive, domain-appropriate naming in code instead

--- Original Design Draft Start ---

Read [CLAUDE.md](./CLAUDE.md) first.

You should first read the spec, and generate as many test cases as possible. Each test case should be a valid Twic snippet, and its corresponding JSON snippet.

Then, you should implement the tokenizer. The tokenizer should be a state machine that reads the input and produces a stream of tokens. The core tokenizer should be a generic tokenizer and it should be used to implement the stub `StrReader` and `StdTokenReader` structs. The implementation should be placed at [read.rs](./twic-new/src/read.rs).

Also, the yet-to-be-implemented `TokenRead::into_value` method should be implemented. This method should be used to convert the stream of tokens into a value. The implementation should also be placed at [read.rs](./twic-new/src/read.rs).

Finally, you should use the test cases to test the implementation.

--- Original Design Draft End ---

---

## BitLesson Selection (REQUIRED FOR EACH TASK)

Before executing each task or sub-task, you MUST:

1. Read @/home/aarkegz/source/repos/twic/twic-new/.humanize/bitlesson.md
2. Run `bitlesson-selector` for each task/sub-task to select relevant lesson IDs
3. Follow the selected lesson IDs (or `NONE`) during implementation

Include a `## BitLesson Delta` section in your summary with:
- Action: none|add|update
- Lesson ID(s): NONE or comma-separated IDs
- Notes: what changed and why (required if action is add or update)

Reference: @/home/aarkegz/source/repos/twic/twic-new/.humanize/bitlesson.md

---

## Goal Tracker Rules

Throughout your work, you MUST maintain the Goal Tracker:

1. **Before starting a task**: Mark it as "in_progress" in Active Tasks
   - Confirm Tag/Owner routing is correct before execution
2. **After completing a task**: Move it to "Completed and Verified" with evidence (but mark as "pending verification")
3. **If you discover the plan has errors**:
   - Do NOT silently change direction
   - Add entry to "Plan Evolution Log" with justification
   - Explain how the change still serves the Ultimate Goal
4. **If you need to defer a task**:
   - Move it to "Explicitly Deferred" section
   - Provide strong justification
   - Explain impact on Acceptance Criteria
5. **If you discover new issues**: Add to "Open Issues" table

---

Note: You MUST NOT try to exit `start-rlcr-loop` loop by lying or edit loop state file or try to execute `cancel-rlcr-loop`

After completing the work, please:
0. If you have access to the `code-simplifier` agent, use it to review and optimize the code you just wrote
1. Finalize @/home/aarkegz/source/repos/twic/twic-new/.humanize/rlcr/2026-04-15_16-02-37/goal-tracker.md (this is Round 0, so you are initializing it - see "Goal Tracker Setup" above)
2. Commit your changes with a descriptive commit message
3. Write your work summary into @/home/aarkegz/source/repos/twic/twic-new/.humanize/rlcr/2026-04-15_16-02-37/round-0-summary.md

Note: Since `--push-every-round` is enabled, you must push your commits to remote after each round.
