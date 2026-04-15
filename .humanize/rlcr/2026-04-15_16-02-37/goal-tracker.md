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
- CQ-3: Minimal type complexity (no nested Option/Result wrappers)
- CQ-4: No unnecessary abstraction
- CQ-5: Correctness over cleverness (edge cases tested)
- CQ-6: API consistency
- CQ-7: Spec compliance

---

## MUTABLE SECTION
<!-- Update each round with justification for changes -->

### Plan Version: 2 (Updated: Round 0)

#### Plan Evolution Log
| Round | Change | Reason | Impact on AC |
|-------|--------|--------|--------------|
| 0 | Initialized with strict CQ review criteria | User requested very strict Codex reviewer | All CQ |

#### Active Tasks
| Task | Target AC | Status | Tag | Owner | Notes |
|------|-----------|--------|-----|-------|-------|
| Self-review against CQ-1 through CQ-7 | CQ-1..7 | in_progress | coding | claude | Audit existing code for violations |

### Completed and Verified
| AC | Task | Completed Round | Verified Round | Evidence |
|----|------|-----------------|----------------|----------|
| AC-1..AC-8 | Full tokenizer+parser implementation | Prior loop R0-R5 | Prior loop R5 | 57 tests pass, 0 warnings, std+no_std |
| CQ-1 | StrCharReader: Chars not CharIndices | Prior loop R6 | - | No dead code |
| CQ-2 | skip_whitespace replaces manual loop | Prior loop R6 | - | No duplicate logic |
| CQ-3 | peeked simplified to Option<Spanned<Token>> | Prior loop R6 | - | No nested wrappers |

### Explicitly Deferred
| Task | Original AC | Deferred Since | Justification | When to Reconsider |
|------|-------------|----------------|---------------|-------------------|

### Open Issues
| Issue | Discovered Round | Blocking AC | Resolution Path |
|-------|-----------------|-------------|-----------------|
