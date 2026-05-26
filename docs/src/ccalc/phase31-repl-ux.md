# Phase 31 — Configurable REPL Prompt + Syntax Highlighting

Introduced in **v0.42.0**.

Phase 31 adds two user-experience improvements to the interactive REPL:
a fully configurable prompt (31a) and real-time syntax highlighting (31b + 31c).

---

## 31a — Configurable prompt

The prompt template is controlled by two keys in `~/.config/ccalc/config.toml`:

```toml
[repl]
prompt1 = "[ {ans} ]: "    # primary prompt (default)
prompt2 = "  >> "           # continuation prompt inside multi-line blocks
```

### Content placeholders

| Placeholder | Expands to |
|-------------|------------|
| `{ans}` | Formatted value of `ans` |
| `{line}` | Session command counter |
| `{user}` | Current OS username |
| `{host}` | Short hostname (before the first dot) |
| `{cwd}` | Full current working directory |
| `{cwd_short}` | Last path component of the current directory |
| `{time}` | Current time as `HH:MM:SS` (UTC) |

### Colour placeholders

Named ANSI colours: `{reset}`, `{bold}`, `{dim}`, `{black}`, `{red}`,
`{green}`, `{yellow}`, `{blue}`, `{magenta}`, `{cyan}`, `{white}`, `{gray}`,
and eight `{bright_*}` variants.

24-bit truecolor: `{#RRGGBB}` (e.g. `{#FF8800}` for orange).

```toml
[repl]
prompt1 = "{gray}({line}){reset} [ {ans} ]: "
prompt1 = "{green}{user}@{host}{reset}:{cyan}{cwd_short}{reset}$ "
prompt1 = "{#FF8800}ccalc{reset} [{line}] {ans} > "
```

### Architecture: dual-output `render_prompt()`

rustyline computes cursor position from the plain prompt string. Injecting ANSI
escape codes into the string passed to `readline()` shifts input text sideways.
The fix uses the `Highlighter` trait's `highlight_prompt()` instead:

- `render_prompt()` returns `(plain: String, colored: String)`.
- `plain` (no ANSI) is passed to `readline()` for correct cursor math.
- `colored` is stored in `CcalcHelper.colored_prompt` and returned by
  `highlight_prompt()` so the terminal renders the colours.

**Implementation files:**
- `crates/ccalc/src/repl.rs` — `render_prompt()`, `parse_rgb_placeholder()`,
  `CcalcHelper.colored_prompt`, `update_prompt()`, `highlight_prompt()`.
- `crates/ccalc/src/config.rs` — `ReplConfig { prompt1, prompt2 }`,
  `[repl]` section in `DEFAULT_CONFIG`, `Config::prompt1()` / `prompt2()`.

---

## 31b — Syntax highlighting

ccalc highlights the current input line in real time as you type. Highlighting
is implemented via `rustyline::Highlighter` on `CcalcHelper`.

### Colour categories

| Category | Default colour | Tokens |
|----------|---------------|--------|
| Keywords | yellow | `if` `for` `while` `end` `function` `else` `elseif` `return` `break` `continue` `do` `until` `switch` `case` `otherwise` `try` `catch` `global` `persistent` |
| Numbers | cyan | `42`, `3.14`, `1e-3`, `0xFF` |
| Strings | green | `'hello'`, `"world"` |
| Comments | dark gray | `% comment`, `# comment` |
| Built-ins | bright cyan | `sin`, `plot`, `zeros`, `reshape`, … |
| Errors | red | Unclosed `'`, `"`, `[`, `(` |
| User variables / operators | default | everything else |

### Shadowing

If a keyword or built-in name is assigned as a user variable (e.g. `end = 42`),
it gets default colour — consistent with evaluation semantics where the variable
shadows the keyword.

### Implementation

`highlight_line(line, env_keys, builtin_keys, colors) -> String` is a standalone
function in `crates/ccalc/src/highlight.rs`. It uses a character-level scanner
with a `Prev` state enum to correctly distinguish `'` as a transpose operator
(after an identifier, number, `)`, `]`) from the start of a char-array string
literal.

---

## 31c — Configurable colour scheme

Colours are set in the `[highlight]` section of `config.toml`:

```toml
[highlight]
enabled  = true         # set to false to disable highlighting

# keywords = "yellow"
# numbers  = "cyan"
# strings  = "green"
# comments = "dark_gray"
# builtins = "bright_cyan"
# errors   = "red"
```

### Colour formats

| Format | Example | Notes |
|--------|---------|-------|
| Named 4-bit | `"yellow"`, `"bright_cyan"`, `"dark_gray"` | 16 standard colours |
| 8-bit palette | `"color256(220)"` | 256-colour extended palette |
| 24-bit truecolor | `"#FFD700"` | Requires a true-color terminal |
| Bold prefix | `"bold:yellow"` | Combines bold with any colour above |

Unknown values are silently ignored and the built-in default is used.

### Applying changes

```
[ 0 ]: config reload
Config reloaded.
```

Colour scheme changes take effect immediately without restarting ccalc.

### REPL help

```
[ 0 ]: help highlight
```
