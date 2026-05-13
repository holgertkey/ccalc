# Phase 28 — Plugin Architecture

**Version:** 0.34.0

Introduces a `Plugin` trait and thread-local `PluginRegistry` so extensions can
live in separate crates and register at startup without touching the engine.
The `ccalc-plot` crate is the reference plugin.

## What's new

- **`Plugin` trait** (`ccalc-engine::plugin`) — implement `name()`, optionally
  `exported_names()`, and `call()`.
- **`PluginRegistry`** — maps exported names to plugin implementations; checked
  before the built-in table so plugins can shadow any built-in.
- **`register_plugin(p)`** — registers a `Box<dyn Plugin>` in the thread-local
  registry.
- **`ccalc-plot` crate** — stub plugin that registers `plot`, `scatter`, `bar`,
  `stem`, `xlabel`, `ylabel`, and `title`. Real rendering is added in Phase 29.
- **Tab completion** — plugin exported names appear alongside built-ins in the
  REPL's tab completer.

## Completion criteria

- `plot(1)` prints the stub message and returns without error.
- `sin(1)` continues to work (built-in fallthrough unchanged).
- An empty `PluginRegistry` produces identical behaviour to v0.33.0.
- All existing tests pass unmodified.

## See also

- [Plugins guide](../guide/plugins.md) — how to write and register a plugin.
- Phase 29 — Plot engine (fills `PlotPlugin` with real rendering).
