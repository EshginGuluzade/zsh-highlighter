# Ralph Loop Instructions for Claude Code

You are operating in an autonomous Ralph loop. Each iteration you start fresh with no memory of previous iterations.

## Your State Sources

1. **progress.txt** — Learnings and design patterns. Read this FIRST before anything else.
2. **prd.json** — Task list. Find the highest-priority story where `passes: false`.
3. **git log** — What code has been written so far.
4. **SPEC.md** — The full project specification. Reference this for requirements.
5. **Existing source files** — ALWAYS read all files in `src/` and `plugin/` before making changes. Understand existing code before modifying it.

## Per-Iteration Workflow

1. Read `progress.txt` for design patterns and learnings
2. Read `prd.json` and find the highest-priority incomplete story (`passes: false`)
3. Read `SPEC.md` for the full project specification
4. Read ALL existing source files in `src/` (and `plugin/` if they exist) to understand current state
5. Check you are on the correct git branch (`ralph/zsh-highlighter-v1`)
6. Implement ONLY that single story — do not work on other stories
7. Run quality checks:
   - `cargo build --release 2>&1` (must compile with zero warnings)
   - `cargo clippy 2>&1` (no warnings)
   - `cargo test 2>&1` (all tests pass)
   - For zsh files: `zsh -n plugin/zsh-highlighter.zsh` (syntax check)
8. If all checks pass:
   - Stage and commit with message format: `feat: [US-XXX] - Story Title`
   - Update `prd.json`: set the story's `passes` to `true`
   - Append learnings to `progress.txt` (what you built, any gotchas, patterns for future iterations)
   - Stage and commit prd.json and progress.txt with message: `chore: mark US-XXX complete`
9. If checks fail: fix the issues and re-run checks until they pass
10. When done, output: `<prompt>COMPLETE</prompt>`

## Rules

- **One story per iteration** — do not combine stories
- **Zero external Rust dependencies** — no crates in Cargo.toml [dependencies]
- **Follow SPEC.md exactly** — it defines styles, protocol, and behavior
- **Read existing code first** — NEVER modify a file you haven't read. Understand the current implementation before changing it.
- **Do not break existing tests** — all previously passing tests must continue to pass
- **Keep the binary fast** — no unnecessary allocations, no heavy computation
- **Tests are mandatory** — every story requires tests for its acceptance criteria
- **Follow progress.txt patterns** — especially the tokenizer design (single-pass state machine)
