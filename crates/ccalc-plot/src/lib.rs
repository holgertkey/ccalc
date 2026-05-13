//! Plot plugin for ccalc.
//!
//! Registers `plot`, `scatter`, `bar`, `stem`, `xlabel`, `ylabel`, and `title`
//! as built-in functions. In Phase 28 these are stubs that validate the plugin
//! plumbing; real rendering is added in Phase 29.

use ccalc_engine::env::{Env, Value};
use ccalc_engine::plugin::Plugin;

/// Plot plugin stub — validates the plugin architecture end-to-end.
///
/// Exports `plot`, `scatter`, `bar`, `stem`, `xlabel`, `ylabel`, and `title`.
/// All calls print a diagnostic message and return [`Value::Void`].
pub struct PlotPlugin;

const EXPORTED: &[&str] = &[
    "plot", "scatter", "bar", "stem", "xlabel", "ylabel", "title",
];

impl Plugin for PlotPlugin {
    fn name(&self) -> &str {
        "plot"
    }

    fn exported_names(&self) -> &[&str] {
        EXPORTED
    }

    fn call(&self, args: &[Value], _env: &Env) -> Result<Value, String> {
        if args.is_empty() {
            return Err("plot: at least one argument required".into());
        }
        eprintln!("[plot stub] called with {} arg(s)", args.len());
        Ok(Value::Void)
    }
}
