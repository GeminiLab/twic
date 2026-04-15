# BitLesson Knowledge Base

This file is project-specific. Keep entries precise and reusable for future rounds.

## Entry Template (Strict)

Use this exact field order for every entry:

```markdown
## Lesson: <unique-id>
Lesson ID: <BL-YYYYMMDD-short-name>
Scope: <component/subsystem/files>
Problem Description: <specific failure mode with trigger conditions>
Root Cause: <direct technical cause>
Solution: <exact fix that resolved the problem>
Constraints: <limits, assumptions, non-goals>
Validation Evidence: <tests/commands/logs/PR evidence>
Source Rounds: <round numbers where problem appeared and was solved>
```

## Entries

## Lesson: unsafe-self-ref
Lesson ID: BL-20260415-unsafe-self-ref
Scope: src/read/mod.rs (StdTokenReader)
Problem Description: Storing a Chars iterator that borrows from a co-stored String via unsafe raw pointer lifetime cast creates a self-referential struct. Moving the struct invalidates the internal borrow, causing undefined behavior.
Root Cause: Classic self-referential struct antipattern — extending a reference lifetime to 'static via pointer cast does not make the borrow valid after the owner moves.
Solution: Replace with position-based indexing: store Box<str> and a usize byte offset. Read characters via buf[pos..].chars().next() and advance by c.len_utf8(). No unsafe code needed.
Constraints: Input must be fully buffered in memory (acceptable for StdTokenReader which reads to String anyway).
Validation Evidence: cargo test: 44 unit tests + 122 doc tests pass. No unsafe blocks remain in crate.
Source Rounds: 0 (introduced), 1 (fixed)
