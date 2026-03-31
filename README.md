# zsh-highlighter

Minimal, ultra-fast zsh syntax highlighter written in Rust. Zero configuration — install and forget.

## Installation

```sh
brew tap EshginGuluzade/zsh-highlighter
brew install zsh-highlighter
```

Add to your `~/.zshrc`:

```zsh
source "$(brew --prefix)/share/zsh-highlighter/zsh-highlighter.zsh"
```

Restart your shell or run `source ~/.zshrc`.

## What it highlights

| Category         | Style              | Example              |
|------------------|--------------------|----------------------|
| Valid command     | green, bold        | `git`, `ls`, `echo`  |
| Invalid command   | red, underline     | `gti`, `sl`          |
| Reserved word     | yellow, bold       | `if`, `then`, `done` |
| String            | yellow             | `"hello"`, `'world'` |
| Comment           | gray               | `# comment`          |
| Operator          | cyan               | `|`, `&&`, `;`, `>`  |
| Arguments         | unstyled (default) | `-la`, `/tmp`        |

Colors use your terminal's palette, so they automatically match your theme (Solarized, Dracula, Nord, etc.).

## Usage

Highlighting works automatically as you type. No configuration needed.

### Commands

| Command     | Description                                          |
|-------------|------------------------------------------------------|
| `zh-reload` | Rebuild command cache after installing new CLI tools  |

The command cache also refreshes automatically when you change directories.

## How it works

A ~40-line zsh plugin hooks into the line editor and calls a compiled Rust binary on each buffer change. The binary tokenizes the input, classifies tokens, and outputs `region_highlight` entries. Total round-trip is under 5ms.

## Requirements

- zsh 5.1+
- macOS or Linux

## Manual installation

```sh
git clone https://github.com/EshginGuluzade/zsh-highlighter.git
cd zsh-highlighter
cargo build --release
```

Then add to `~/.zshrc`:

```zsh
source /path/to/zsh-highlighter/plugin/zsh-highlighter.zsh
```

## License

MIT — see [LICENSE](LICENSE).
