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

Implement a complete tokenizer (state machine) and recursive descent parser for the Twic data serialization format in the `twic-new` Rust crate, with span-tracked tokens and errors, comprehensive tests, and zero warnings.

### Acceptance Criteria

- AC-1: Test cases for all 6 Twic types + edge cases (leading zeros, escapes, nested structures)
- AC-2: Token enum with complete data + span info (AC-2.1)
- AC-3: Error types with specific variants + span info (AC-3.1)
- AC-4: Tokenizer handles all Twic syntax per formal spec
- AC-5: parse_str works correctly
- AC-6: parse_read works (std feature)
- AC-7: Parser produces correct Value trees
- AC-8: All tests pass (std + no_std), zero warnings

### Code Quality Criteria (mandatory for Codex)

- CQ-1: No dead code, zero warnings
- CQ-2: No duplicate logic
- CQ-3: Minimal type complexity (no nested Option/Result wrappers except for peeking)
- CQ-4: No unnecessary abstraction
- CQ-5: Correctness over cleverness (edge cases tested)
- CQ-6: API consistency
- CQ-7: Spec compliance

---

## MUTABLE SECTION
<!-- Update each round with justification for changes -->

### Plan Version: 1 (Updated: Round 0)

#### Plan Evolution Log
| Round | Change | Reason | Impact on AC |
|-------|--------|--------|--------------|
| 0 | Initialized from prior loop state | New RLCR loop started | All AC |
| 0 | Renamed Error to ParseError, unconditional core::error::Error impl | Codex R0 review: no_std trait gap | CQ-6 |
| 0 | Added \r/\r\n line break handling in span tracking | Codex R0 review: span correctness | CQ-5 |

#### Active Tasks
| Task | Target AC | Status | Tag | Owner | Notes |
|------|-----------|--------|-----|-------|-------|
| Self-review against CQ-1 through CQ-7 | CQ-1..7 | completed | coding | claude | Fixed 3 issues: no_std Error trait, ParseError rename, CRLF spans |

### Completed and Verified
| AC | Task | Completed Round | Verified Round | Evidence |
|----|------|-----------------|----------------|----------|
| AC-1..AC-8 | Full tokenizer+parser implementation | Prior loop | Prior loop R5 | 58 tests pass, 0 warnings, std+no_std |
| CQ-3 | Option<Option<char>> eliminated in CharReaderState | Prior loop R1 | - | peeked_char + has_peeked |
| CQ-3 | tokenize_next return type flattened | Prior loop R1 | - | Result<Option<...>> |
| CQ-6 | parse_read returns ReadError preserving Spanned<Error> | Prior loop R1 | - | ReadError enum with From impls |
| CQ-1 | Removed unused error::Result type alias | Prior loop R1 | - | Dead code removal |
| CQ-2 | Reused text_no_sign in parse_number | Prior loop R1 | - | Eliminated duplicate strip_prefix |
| CQ-5 | NegInt encoding documented | Prior loop R1 | - | Comment on wrapping_neg |
| CQ-4 | CharReader trait impls documented | Prior loop R1 | - | Doc comment |
| CQ-1 | Fixed dyn std::error::Error in span.rs for no_std | R0 | - | Changed to dyn Error (core::error::Error) |
| CQ-6 | Renamed Error to ParseError, unconditional core::error::Error | R0 | - | no_std trait impl for ParseError |
| CQ-5 | Added \r/\r\n line break handling with last_was_cr | R0 | - | Correct span tracking for CRLF/CR |

### Explicitly Deferred
| Task | Original AC | Deferred Since | Justification | When to Reconsider |
|------|-------------|----------------|---------------|-------------------|

### Open Issues
| Issue | Discovered Round | Blocking AC | Resolution Path |
|-------|-----------------|-------------|-----------------|
