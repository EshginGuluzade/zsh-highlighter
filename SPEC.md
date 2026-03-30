# zsh-highlighter — Specification

## Philosophy

Do less, but do it at compiled speed. Zero configuration. Install and forget.

---

## Architecture

```
┌────────────────────────────────────────────┐
│  zsh plugin (~40 lines)                    │
│  - hooks into zle-line-pre-redraw          │
│  - skips if buffer unchanged               │
│  - calls binary with $BUFFER as argv       │
│  - binary inherits $_ZH_CMDS env var       │
│  - sets region_highlight from stdout       │
└──────────────┬─────────────────────────────┘
               │  fork+exec per buffer change
┌──────────────▼─────────────────────────────┐
│  zsh-highlighter binary (Rust, ~500 LOC)   │
│  - reads buffer from argv[1]               │
│  - reads known commands from $_ZH_CMDS env │
│  - tokenizes → classifies → emits          │
│  - outputs region_highlight entries        │
│  - exits                                   │
└────────────────────────────────────────────┘
```

**No daemon. No sockets. No config files. No PID management.**

The binary is invoked per buffer change (NOT per keystroke — the zsh plugin
skips if `$BUFFER` is unchanged). It receives the command string as `argv[1]`,
reads known commands from the inherited `$_ZH_CMDS` environment variable,
and writes `region_highlight` entries to stdout. Total round-trip: ~2–5ms on
macOS, well below the human perception threshold (~13ms).

### Why not a daemon?

- Daemon lifecycle adds complexity (start/stop/restart, crash recovery, orphaned processes)
- 2–5ms is already imperceptible
- A stateless binary is trivial to debug, upgrade, and reason about
- If speed becomes insufficient, we can later upgrade to a long-running coprocess without changing the user-facing interface

### Why not pure zsh?

- Pure zsh highlighters (zsh-syntax-highlighting, fast-syntax-highlighting) suffer from O(n) interpreter overhead per token
- They exhibit O(n^2) behavior with array operations on older zsh versions
- A compiled tokenizer handles edge cases (nested quotes, escapes) more correctly and consistently
- Even a minimal pure-zsh highlighter takes 5–50ms; our binary completes in <1ms

---

## What to Highlight

7 categories, ordered by value:

| # | Category            | Style                 | Detection                              |
|---|---------------------|-----------------------|----------------------------------------|
| 1 | Invalid command     | `fg=red,underline`    | Word in command position, not in `$_ZH_CMDS` |
| 2 | Valid command        | `fg=green,bold`       | Word in command position, found in `$_ZH_CMDS` |
| 3 | Builtin/alias/func  | `fg=green,bold`       | Same as valid command (no distinction — simpler mental model) |
| 4 | Reserved word        | `fg=yellow,bold`      | Keyword list (see below)              |
| 5 | String               | `fg=yellow`           | `'...'`, `"..."`, `$'...'`           |
| 6 | Comment              | `fg=8` (gray)         | `#` to end of line, outside strings   |
| 7 | Operator/redirect    | `fg=cyan`             | `\|`, `\|\|`, `&&`, `;`, `;;`, `>`, `>>`, `<`, `<<`, `&` |

Arguments and everything else: **no styling** (terminal default).

### Reserved words

```
if then else elif fi
for in while until do done
case esac
function
select repeat time
```

**Note on `{ }` and `[[ ]]`:** These are tokenized as Operators (they break
tokens like `(` `)`) but styled as reserved words (`fg=yellow,bold`) by the
classifier. This avoids ambiguity in the tokenizer — they behave structurally
like operators but visually like reserved words.

### What is NOT highlighted (and why)

| Skipped               | Reason                                               |
|-----------------------|------------------------------------------------------|
| `$var`, `${var}`      | Requires deep quoting-context tracking; low ROI       |
| `$(...)` contents     | Would need recursive parsing; rare interactively      |
| Options (`-x`, `--f`) | Fish doesn't highlight these either; visual noise     |
| File paths            | Requires `stat()` per token — expensive, flickers     |
| Glob patterns         | Hard to distinguish from arguments; low value         |
| Here-documents        | Extremely rare in interactive input                   |
| Escape sequences      | Diminishing returns for the complexity                |

---

## Colors

Use **ANSI named colors** from the terminal palette — NOT hardcoded RGB. The
highlighter automatically inherits the user's terminal color scheme (Solarized,
Dracula, Nord, Catppuccin, default). Zero configuration needed.

```
Valid command:     fg=green,bold
Invalid command:   fg=red,underline
Reserved word:     fg=yellow,bold
String:            fg=yellow
Comment:           fg=8                (256-color gray — works on dark and light)
Operator:          fg=cyan
Everything else:   (no styling)
```

Why `fg=8` for comments instead of `fg=bright black`:
- `fg=8` is the 256-color code for "bright black" and is more portable
- Renders as gray on virtually all terminal themes

---

## Tokenizer Design (Rust)

A **minimal zsh lexer** — NOT a full shell parser. Only enough structure to
identify command position and token boundaries.

### Handles

- **Word splitting**: on whitespace and operator characters
- **Command position detection**: first word after `|`, `||`, `&&`, `;`, `;;`, `(`, `{`, or start-of-input; also after reserved words that expect a command (`then`, `else`, `do`, `!`, `time`)
- **Single-quoted strings**: `'...'` (no escape processing, terminated by next `'`)
- **Double-quoted strings**: `"..."` (handle `\"` and `\\` for correct boundary detection)
- **ANSI-C strings**: `$'...'` (handle `\'` escape for correct boundary detection)
- **Backtick strings**: `` `...` `` (for correct boundary detection)
- **Comments**: `# ...` — `#` starts a comment ONLY when preceded by whitespace or at position 0 (not mid-word like `foo#bar`). Extends to end of line, only outside strings.
- **Operators**: `|`, `||`, `|&`, `&&`, `;`, `;;`, `>`, `>>`, `<`, `<<`, `<<<`, `&`, `(`, `)`, `{`, `}`, `[[`, `]]`
- **Line continuations**: `\<newline>` (backslash immediately before newline)
- **Subcommand awareness**: after `|`, `&&`, `||`, etc., the next word is in command position

### Does NOT handle

- `$(...)` command substitution (treated as part of surrounding word)
- `${...}` parameter expansion (treated as part of surrounding word)
- `$((...))` arithmetic
- Here-documents (`<<EOF`)
- Process substitution `<(...)` / `>(...)`
- Nested quoting edge cases

This covers ~95% of interactive shell input while keeping the lexer under ~300 lines of Rust.

### Command validation

The binary reads the `$_ZH_CMDS` environment variable (newline-separated list of
known commands: builtins + aliases + functions + PATH executables). On startup,
it parses this into a `HashSet<&str>` for O(1) lookup.

For tokens in **command position**:
- If the token is in the reserved words list → `reserved_word`
- Else if the token is in `$_ZH_CMDS` → `valid_command`
- Else → `invalid_command`

Typical `$_ZH_CMDS` size: 2000–5000 entries, ~30–50KB. Parsing into a HashSet takes <0.1ms.

---

## Binary I/O Protocol

### Input

- `argv[1]`: the shell buffer (`$BUFFER`)
- `$_ZH_CMDS` environment variable: newline-separated known commands

### Output

One line per highlighted region on stdout:

```
<start> <end> <style>
```

Where:
- `start`: 0-indexed byte offset into the buffer
- `end`: 0-indexed byte offset (exclusive)
- `style`: zsh highlight spec (e.g., `fg=green,bold`)

Example for `git commit -m "hello"`:
```
0 3 fg=green,bold
4 10 fg=yellow,bold
14 21 fg=yellow
```

Empty output (no lines) means no highlighting applied. The binary exits with
code 0 on success. Any non-zero exit means "skip highlighting for this buffer."

---

## Zsh Plugin Design

~40 lines of zsh. Core logic:

```
1. On load:
   - Locate binary (next to plugin file, or in PATH)
   - Build $_ZH_CMDS from: ${(k)commands}, ${(k)builtins}, ${(k)aliases}, ${(k)functions}, reserved words
   - Export $_ZH_CMDS
   - Register zle-line-pre-redraw hook

2. On each line-pre-redraw:
   - Return immediately if $BUFFER is empty
   - Return immediately if $BUFFER == $_zh_prev_buffer (unchanged)
   - Return immediately if ${#BUFFER} > 10000 (safety cap: 10KB)
   - Set _zh_prev_buffer=$BUFFER
   - Call: result=$($ZH_BIN "$BUFFER")
   - Parse result lines into region_highlight array

3. zh-reload function:
   - Run 'rehash' (tells zsh to rebuild its command hash table)
   - Rebuild $_ZH_CMDS
   - Clear _zh_prev_buffer (force re-highlight on next keystroke)

4. chpwd hook:
   - Rebuild $_ZH_CMDS (handles directory-dependent PATH entries)
```

### Paste performance

The `line-pre-redraw` hook fires on every redraw. With bracketed paste enabled
(default on modern terminals), the entire paste arrives as one buffer change, so
only one binary invocation occurs. No special handling needed.

If a user has `bracketed-paste-magic` enabled, individual characters during paste
may trigger redraws. At ~3ms per invocation, even a 100-char paste completes
highlighting in ~300ms — acceptable and far better than pure-zsh highlighters.

### Graceful degradation

- If binary not found: plugin loads silently, no highlighting applied
- If binary returns non-zero: skip highlighting for that buffer
- If binary output is malformed: skip highlighting for that buffer
- If `$_ZH_CMDS` is empty: all commands classified as valid (safe default)

---

## Performance Targets

| Metric                | Target     | Notes                                        |
|-----------------------|------------|----------------------------------------------|
| Shell startup overhead | < 5ms     | Plugin is ~40 lines; builds env var from zsh hash tables |
| Per-change latency     | < 5ms     | fork+exec (~2ms) + tokenize (<1ms) + I/O (<1ms) |
| Binary startup         | < 2ms     | Rust with LTO, panic=abort, no heavy deps    |
| Binary size            | < 2MB     | Minimal deps (no syntect, no serde, no regex crate) |
| Memory (binary)        | < 5MB RSS | Short-lived, only HashSet + small buffers     |
| Max input              | 10KB      | Beyond this, skip highlighting               |

### Rust build optimization

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

---

## Minimum Requirements

- **zsh**: 5.1+ (for `region_highlight` support and `add-zle-hook-widget`)
- **OS**: macOS (primary), Linux (supported)
- **Rust**: 1.70+ (for build)

---

## User-Facing Interface

### Installation

```sh
brew install zsh-highlighter
```

### Setup (one line in ~/.zshrc)

```zsh
source "$(brew --prefix)/share/zsh-highlighter/zsh-highlighter.zsh"
```

### Commands

| Command      | Description                          |
|--------------|--------------------------------------|
| `zh-reload`  | Rebuild command cache after installing new CLI tools |

That's it. No config files. No environment variables to set. No themes to choose.

---

## Distribution

### Homebrew formula

- Builds Rust binary from source (or pre-built bottles for macOS ARM/Intel, Linux x86_64)
- Installs binary to `$(brew --prefix)/bin/zsh-highlighter`
- Installs plugin to `$(brew --prefix)/share/zsh-highlighter/zsh-highlighter.zsh`

### Also supported (but not primary)

- Manual install: clone repo, `cargo build --release`, source the plugin
- Plugin managers (zinit, antidote, sheldon): compatible, but not a priority

---

## Project Structure

```
zsh-highlighter/
├── Cargo.toml
├── src/
│   ├── main.rs              # Read argv + env, call tokenizer, format output
│   ├── tokenizer.rs         # Minimal zsh lexer (~300 lines)
│   └── classifier.rs        # Map tokens to region_highlight styles (~100 lines)
├── plugin/
│   └── zsh-highlighter.zsh  # Zsh plugin (~40 lines)
├── Formula/
│   └── zsh-highlighter.rb   # Homebrew formula
├── SPEC.md                  # This file
└── README.md
```

~500 lines of Rust + ~40 lines of zsh.

---

## Future Considerations (NOT for v1)

These are escape hatches if the per-invocation model proves insufficient:

1. **Coprocess mode**: keep the binary alive as a zsh coprocess, communicate via stdin/stdout. Eliminates ~2ms fork+exec overhead per change. Same binary, add a `--server` flag.

2. **Async highlighting**: tokenize synchronously for instant feedback, validate commands asynchronously, apply updates on completion.

3. **Subcommand-aware highlighting**: recognize `git commit`, `docker run`, etc. and highlight subcommands. Requires a registry of known multi-word commands.

None of these change the user-facing interface.
