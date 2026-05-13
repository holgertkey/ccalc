# Plugins

ccalc's built-in list is extended via a lightweight **plugin system**. A plugin
is a separate Rust crate that implements the `Plugin` trait and registers itself
at startup. The engine checks the plugin registry before its own built-in table,
so plugins can shadow existing built-ins when needed.

## Writing a plugin

Add `ccalc-engine` as a dependency and implement the `Plugin` trait:

```toml
# my-plugin/Cargo.toml
[dependencies]
ccalc-engine = { path = "../ccalc-engine" }
```

```rust
use ccalc_engine::env::{Env, Value};
use ccalc_engine::plugin::Plugin;

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "myfunc" }

    fn call(&self, args: &[Value], _env: &Env) -> Result<Value, String> {
        if args.is_empty() {
            return Err("myfunc: at least one argument required".into());
        }
        Ok(args[0].clone())
    }
}
```

### Exporting multiple names

A single plugin registration can expose several function names. Override
`exported_names` with a `const`-backed slice:

```rust
const NAMES: &[&str] = &["myfunc", "myother", "mythird"];

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "myfunc" }

    fn exported_names(&self) -> &[&str] { NAMES }

    fn call(&self, args: &[Value], _env: &Env) -> Result<Value, String> {
        // dispatch internally based on args or other state
        Ok(Value::Void)
    }
}
```

All exported names appear in tab completion automatically.

## Registering a plugin

In your fork of `crates/ccalc/src/main.rs`, call `register_plugin` after
`exec::init()`:

```rust
fn run() {
    ccalc_engine::exec::init();
    ccalc_engine::plugin::register_plugin(Box::new(MyPlugin));
    // …
}
```

Add your crate to the workspace and as a dependency of `ccalc`:

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["crates/ccalc", "crates/ccalc-engine", "crates/my-plugin"]

# crates/ccalc/Cargo.toml
[dependencies]
my-plugin = { path = "../my-plugin" }
```

## Built-in plugins

`ccalc-plot` is the reference plugin shipped with ccalc. It registers the
`plot`, `scatter`, `bar`, `stem`, `xlabel`, `ylabel`, and `title` names.
See [Phase 29 — Plot engine](../ccalc/phase29-plot.md) for rendering details.
