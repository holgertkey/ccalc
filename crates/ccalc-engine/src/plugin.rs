//! Plugin system for extending ccalc with new built-in functions.
//!
//! Third-party crates implement the [`plugin::Plugin`] trait and register via
//! [`plugin::register_plugin`]. The engine checks the registry before its own
//! built-in table, so plugins can shadow any existing built-in if needed.
//!
//! # Minimal plugin example
//!
//! ```rust,ignore
//! use ccalc_engine::env::{Env, Value};
//! use ccalc_engine::plugin::Plugin;
//!
//! pub struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn name(&self) -> &str { "myfunc" }
//!
//!     fn call(&self, _name: &str, args: &[Value], _env: &Env) -> Result<Value, String> {
//!         if args.is_empty() {
//!             return Err("myfunc: at least one argument required".into());
//!         }
//!         Ok(args[0].clone())
//!     }
//! }
//!
//! // In main.rs / startup:
//! ccalc_engine::plugin::register_plugin(Box::new(MyPlugin));
//! ```
//!
//! Plugins that expose several names (e.g. a plot plugin with `plot`, `scatter`,
//! `bar`, …) should also override [`plugin::Plugin::exported_names`] and return a
//! `const`-backed slice. The `name` argument to `call` identifies which exported
//! name was invoked, enabling a single plugin to dispatch multiple functions:
//!
//! ```rust,ignore
//! const NAMES: &[&str] = &["plot", "scatter", "bar"];
//!
//! fn exported_names(&self) -> &[&str] { NAMES }
//!
//! fn call(&self, name: &str, args: &[Value], _env: &Env) -> Result<Value, String> {
//!     match name {
//!         "plot"    => { /* ... */ Ok(Value::Void) }
//!         "scatter" => { /* ... */ Ok(Value::Void) }
//!         _         => Err(format!("{name}: not implemented"))
//!     }
//! }
//! ```

use std::cell::RefCell;

use crate::env::{Env, Value};

/// Trait that all ccalc plugins must implement.
///
/// Implement this in a separate crate and register an instance via
/// [`register_plugin`] before any evaluation takes place.
pub trait Plugin: Send + Sync {
    /// The primary name of this plugin.
    ///
    /// Used as the canonical identifier when looking up the plugin if
    /// [`Plugin::exported_names`] is empty.
    fn name(&self) -> &str;

    /// All names exported by this plugin (used for dispatch and tab completion).
    ///
    /// Defaults to `&[]`. If you return a non-empty slice the engine dispatches
    /// by the slice; otherwise it falls back to [`Plugin::name`].
    ///
    /// For multi-function plugins, override this with a `const`-backed slice:
    ///
    /// ```rust,ignore
    /// const NAMES: &[&str] = &["plot", "scatter", "bar"];
    /// fn exported_names(&self) -> &[&str] { NAMES }
    /// ```
    fn exported_names(&self) -> &[&str] {
        &[]
    }

    /// Evaluate a call to one of this plugin's exported names.
    ///
    /// # Arguments
    ///
    /// * `name` — the exact function name that was called (one of [`Plugin::exported_names`])
    /// * `args` — evaluated argument values (already evaluated by the engine)
    /// * `env`  — current variable environment (read-only)
    ///
    /// # Errors
    ///
    /// Return `Err(msg)` to propagate an error to the user.
    fn call(&self, name: &str, args: &[Value], env: &Env) -> Result<Value, String>;
}

/// Registry that maps exported names to their plugin implementations.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Registers a plugin, making all its exported names available for dispatch.
    pub fn register(&mut self, p: Box<dyn Plugin>) {
        self.plugins.push(p);
    }

    /// Returns the plugin that handles `name`, or `None` if no plugin claims it.
    ///
    /// Checks `exported_names()` first; falls back to `name()` for plugins that
    /// did not override `exported_names`.
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| {
                let exported = p.exported_names();
                if exported.is_empty() {
                    p.name() == name
                } else {
                    exported.contains(&name)
                }
            })
            .map(|p| p.as_ref())
    }

    /// Returns every name exported by all registered plugins.
    pub fn all_names(&self) -> Vec<String> {
        self.plugins
            .iter()
            .flat_map(|p| {
                let exported = p.exported_names();
                if exported.is_empty() {
                    vec![p.name().to_string()]
                } else {
                    exported.iter().map(|s| s.to_string()).collect()
                }
            })
            .collect()
    }
}

thread_local! {
    static REGISTRY: RefCell<PluginRegistry> = RefCell::new(PluginRegistry::new());
}

/// Registers a plugin in the thread-local plugin registry.
///
/// Call this once at program startup (before any evaluation) for each plugin.
///
/// # Examples
///
/// ```rust,ignore
/// ccalc_engine::plugin::register_plugin(Box::new(MyPlugin));
/// ```
pub fn register_plugin(p: Box<dyn Plugin>) {
    REGISTRY.with(|r| r.borrow_mut().register(p));
}

/// Calls the plugin that handles `name`, if one is registered.
///
/// Returns `Some(result)` when a plugin claims the name, `None` otherwise.
/// Used by `call_builtin` to dispatch before the built-in table.
pub(crate) fn call_plugin(name: &str, args: &[Value], env: &Env) -> Option<Result<Value, String>> {
    REGISTRY.with(|r| {
        let reg = r.borrow();
        reg.get(name).map(|p| p.call(name, args, env))
    })
}

/// Returns all names exported by currently registered plugins.
///
/// Used for tab completion in the REPL.
pub fn plugin_names() -> Vec<String> {
    REGISTRY.with(|r| r.borrow().all_names())
}
