use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::io::Write;

use indexmap::IndexMap;
use ndarray::Array2;
use rand::{Rng, SeedableRng, rngs::SmallRng};

use crate::env::{Env, LambdaFn, Value};
use crate::io::IoContext;

// ── User function call hook ──────────────────────────────────────────────────

/// Signature for the hook that executes named user-defined functions.
///
/// Registered once by `exec::init()` before the REPL loop starts.
/// Called by `eval_inner` when a `Value::Function` is invoked.
/// `name` is the function name (from the call site); `caller_env` is passed so
/// the function body can access other user-defined functions (enabling recursion
/// and mutual recursion).
pub type FnCallHook = fn(
    name: &str,
    func: &Value,
    args: &[Value],
    caller_env: &Env,
    io: &mut IoContext,
) -> Result<Value, String>;

thread_local! {
    static FN_CALL_HOOK: Cell<Option<FnCallHook>> = const { Cell::new(None) };
}

/// Registers the hook that executes named user-defined functions.
///
/// Must be called by `exec::init()` before any user function can be called.
pub fn set_fn_call_hook(f: FnCallHook) {
    FN_CALL_HOOK.with(|c| c.set(Some(f)));
}

// ── Autoload hook ───────────────────────────────────────────────────────────

/// Signature for the hook that auto-loads a function file by name.
///
/// Called by `eval_inner` when a name is not found in the environment and not
/// a built-in. The hook searches for `<name>.calc` / `<name>.m` on the path
/// and, if found, inserts the primary function into the autoload cache via
/// [`autoload_cache_insert`]. Returns `true` if the function was loaded.
pub type AutoloadHook = fn(name: &str) -> bool;

thread_local! {
    static AUTOLOAD_HOOK: Cell<Option<AutoloadHook>> = const { Cell::new(None) };
    /// Cache of autoloaded functions — populated by the autoload hook, read by eval_inner.
    static AUTOLOAD_CACHE: RefCell<Env> = RefCell::new(Env::new());
}

/// Registers the autoload hook. Called by `exec::init()`.
pub fn set_autoload_hook(f: AutoloadHook) {
    AUTOLOAD_HOOK.with(|c| c.set(Some(f)));
}

/// Inserts a function into the autoload cache. Called by `exec::try_autoload`.
pub fn autoload_cache_insert(name: String, val: Value) {
    AUTOLOAD_CACHE.with(|c| c.borrow_mut().insert(name, val));
}

/// Returns an autoloaded function by name, triggering the autoload hook if needed.
///
/// Checks the cache first; if not present, fires the registered hook (which searches
/// the session path for `<name>.calc` / `<name>.m`). Returns `None` when neither the
/// cache nor the hook can resolve the name.
pub fn resolve_autoloaded(name: &str) -> Option<Value> {
    let cached = AUTOLOAD_CACHE.with(|c| c.borrow().get(name).cloned());
    if cached.is_some() {
        return cached;
    }
    let hook = AUTOLOAD_HOOK.with(|c| c.get());
    if let Some(f) = hook {
        f(name);
    }
    AUTOLOAD_CACHE.with(|c| c.borrow().get(name).cloned())
}

// ── Last error (thread-local) ────────────────────────────────────────────────

thread_local! {
    static LAST_ERR: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Sets the last-error string (called on every caught runtime error).
pub fn set_last_err(msg: &str) {
    LAST_ERR.with(|e| *e.borrow_mut() = msg.to_string());
}

/// Returns the last-error string.
pub fn get_last_err() -> String {
    LAST_ERR.with(|e| e.borrow().clone())
}

// ── Nargout (number of expected outputs, set by exec_stmts) ─────────────────

thread_local! {
    static NARGOUT: Cell<usize> = const { Cell::new(1) };
}

/// Sets the number of output values requested by the calling assignment statement.
///
/// Called by `exec_stmts` before evaluating the RHS expression, so that
/// multi-output built-ins (`eig`, `svd`, `lu`, `qr`) can determine whether to
/// return a full `Value::Tuple` or a single value.
pub fn set_nargout(n: usize) {
    NARGOUT.with(|c| c.set(n));
}

fn get_nargout() -> usize {
    NARGOUT.with(|c| c.get())
}

// ── Display context (thread-local, set by exec_stmts) ────────────────────────

thread_local! {
    static DISPLAY_FMT:     RefCell<FormatMode> = const { RefCell::new(FormatMode::Short) };
    static DISPLAY_BASE:    Cell<Base>           = const { Cell::new(Base::Dec) };
    static DISPLAY_COMPACT: Cell<bool>           = const { Cell::new(false) };
}

/// Sets the display context used when executing function bodies.
///
/// Called at the start of `exec_stmts` so that named functions called from
/// within a block inherit the caller's display settings.
pub fn set_display_ctx(fmt: &FormatMode, base: Base, compact: bool) {
    DISPLAY_FMT.with(|f| *f.borrow_mut() = fmt.clone());
    DISPLAY_BASE.with(|b| b.set(base));
    DISPLAY_COMPACT.with(|c| c.set(compact));
}

/// Returns the current display format mode stored in the thread-local context.
pub fn get_display_fmt() -> FormatMode {
    DISPLAY_FMT.with(|f| f.borrow().clone())
}

/// Returns the current numeric base stored in the thread-local context.
pub fn get_display_base() -> Base {
    DISPLAY_BASE.with(|b| b.get())
}

/// Returns the current compact flag stored in the thread-local context.
pub fn get_display_compact() -> bool {
    DISPLAY_COMPACT.with(|c| c.get())
}

// ── Global variable store ────────────────────────────────────────────────────

thread_local! {
    /// Shared global workspace — variables declared `global` in any scope live here.
    ///
    /// Persists for the lifetime of the process. Each call to `global x` in any scope
    /// makes `x` refer to this store rather than the local environment.
    static GLOBAL_ENV: RefCell<Env> = RefCell::new(Env::new());

    /// Stack of per-scope global name sets.
    ///
    /// Frame 0 = top level / script scope; each `call_user_function` call pushes a new frame
    /// and pops it on return. `global x` in a scope adds `x` to the current (top) frame.
    static GLOBAL_NAMES_STACK: RefCell<Vec<HashSet<String>>> =
        RefCell::new(vec![HashSet::new()]);
}

/// Pushes an empty global-names frame (called on function entry by `exec.rs`).
pub fn global_frame_push() {
    GLOBAL_NAMES_STACK.with(|s| s.borrow_mut().push(HashSet::new()));
}

/// Pops the top global-names frame (called on function exit by `exec.rs`).
pub fn global_frame_pop() {
    GLOBAL_NAMES_STACK.with(|s| {
        s.borrow_mut().pop();
    });
}

/// Declares `name` as global in the current scope.
pub fn global_declare(name: &str) {
    GLOBAL_NAMES_STACK.with(|s| {
        if let Some(frame) = s.borrow_mut().last_mut() {
            frame.insert(name.to_string());
        }
    });
}

/// Returns `true` if `name` is declared global in the innermost active scope.
pub fn is_global(name: &str) -> bool {
    GLOBAL_NAMES_STACK.with(|s| s.borrow().last().is_some_and(|f| f.contains(name)))
}

/// Gets a value from the shared global store.
pub fn global_get(name: &str) -> Option<Value> {
    GLOBAL_ENV.with(|e| e.borrow().get(name).cloned())
}

/// Sets a value in the shared global store.
pub fn global_set(name: &str, val: Value) {
    GLOBAL_ENV.with(|e| e.borrow_mut().insert(name.to_string(), val));
}

/// Initialises `name` in the global store to `Scalar(0.0)` if not already present.
pub fn global_init_if_absent(name: &str) {
    GLOBAL_ENV.with(|e| {
        e.borrow_mut()
            .entry(name.to_string())
            .or_insert(Value::Scalar(0.0));
    });
}

/// Refreshes all names declared global in the current scope from `GLOBAL_ENV` into `env`.
///
/// Called at the end of `exec_stmts` to ensure that modifications made to global variables
/// inside called functions are visible to the current scope's environment.
pub fn global_refresh_into_env(env: &mut crate::env::Env) {
    GLOBAL_NAMES_STACK.with(|s| {
        GLOBAL_ENV.with(|ge| {
            if let Some(frame) = s.borrow().last() {
                let store = ge.borrow();
                for name in frame {
                    if let Some(val) = store.get(name) {
                        env.insert(name.clone(), val.clone());
                    }
                }
            }
        });
    });
}

// ── Persistent variable store ────────────────────────────────────────────────

thread_local! {
    /// Persistent variable values — keyed by `"funcname\x00varname"`.
    ///
    /// Values survive individual function calls and are restored on the next call
    /// to the same function.
    static PERSISTENT_STORE: RefCell<HashMap<String, Value>> =
        RefCell::new(HashMap::new());

    /// Stack of function names for constructing persistent-store keys.
    ///
    /// Empty string = top-level scope. `call_user_function` pushes the function name
    /// before executing the body and pops it on return.
    static FUNC_NAME_STACK: RefCell<Vec<String>> =
        RefCell::new(vec![String::new()]);

    /// Stack of per-scope persistent name sets — mirrors `GLOBAL_NAMES_STACK`.
    static PERSISTENT_NAMES_STACK: RefCell<Vec<HashSet<String>>> =
        RefCell::new(vec![HashSet::new()]);
}

/// Pushes a function scope for persistent tracking (called on function entry).
pub fn persistent_frame_push(func_name: &str) {
    FUNC_NAME_STACK.with(|s| s.borrow_mut().push(func_name.to_string()));
    PERSISTENT_NAMES_STACK.with(|s| s.borrow_mut().push(HashSet::new()));
}

/// Pops the persistent frame and returns `(func_name, declared_persistent_names)`.
pub fn persistent_frame_pop() -> (String, HashSet<String>) {
    let func_name = FUNC_NAME_STACK.with(|s| s.borrow_mut().pop().unwrap_or_default());
    let names = PERSISTENT_NAMES_STACK.with(|s| s.borrow_mut().pop().unwrap_or_default());
    (func_name, names)
}

/// Declares `name` as persistent in the current function scope.
pub fn persistent_declare(name: &str) {
    PERSISTENT_NAMES_STACK.with(|s| {
        if let Some(frame) = s.borrow_mut().last_mut() {
            frame.insert(name.to_string());
        }
    });
}

/// Gets a saved persistent value for `(func_name, var_name)`.
pub fn persistent_load(func_name: &str, var_name: &str) -> Option<Value> {
    let key = format!("{func_name}\x00{var_name}");
    PERSISTENT_STORE.with(|s| s.borrow().get(&key).cloned())
}

/// Saves a persistent value for `(func_name, var_name)`.
pub fn persistent_save(func_name: &str, var_name: &str, val: Value) {
    let key = format!("{func_name}\x00{var_name}");
    PERSISTENT_STORE.with(|s| s.borrow_mut().insert(key, val));
}

/// Returns the name of the currently executing function (top of `FUNC_NAME_STACK`).
///
/// Returns an empty string when executing at the top level (REPL / script scope).
pub fn current_func_name() -> String {
    FUNC_NAME_STACK.with(|s| s.borrow().last().cloned().unwrap_or_default())
}

/// Returns `true` if `name` is declared `persistent` in the current function frame.
pub fn is_persistent(name: &str) -> bool {
    PERSISTENT_NAMES_STACK.with(|s| s.borrow().last().is_some_and(|frame| frame.contains(name)))
}

// ── Random-number state ──────────────────────────────────────────────────────

thread_local! {
    /// Per-thread PRNG used by `rand`, `randn`, and `randi`.
    ///
    /// Seeded from OS entropy on first use. Reseed with `rng(seed)` or `rng('shuffle')`.
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_entropy());
}

/// Reseeds the thread-local RNG with the given 64-bit seed.
pub fn rng_seed(seed: u64) {
    RNG.with(|r| *r.borrow_mut() = SmallRng::seed_from_u64(seed));
}

/// Reseeds the thread-local RNG from OS entropy.
pub fn rng_shuffle() {
    RNG.with(|r| *r.borrow_mut() = SmallRng::from_entropy());
}

/// Generates one uniform [0, 1) sample.
fn rand_uniform() -> f64 {
    RNG.with(|r| r.borrow_mut().gen_range(0.0_f64..1.0))
}

/// Generates one standard-normal sample via the Box-Muller transform.
fn rand_normal() -> f64 {
    let u1 = rand_uniform().max(f64::EPSILON);
    let u2 = rand_uniform();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

// ── AST types ────────────────────────────────────────────────────────────────

/// An expression node in the AST.
///
/// Produced by the parser and consumed by [`eval`] / [`eval_with_io`].
#[derive(Debug, Clone)]
pub enum Expr {
    /// A numeric literal (e.g. `3`, `2.5`, `1e-3`).
    Number(f64),
    /// A variable or constant reference (e.g. `x`, `pi`, `ans`).
    Var(String),
    /// Arithmetic negation: `-expr`.
    UnaryMinus(Box<Expr>),
    /// Logical NOT: `~expr`. Result is 1.0 if expr == 0.0, else 0.0.
    UnaryNot(Box<Expr>),
    /// Binary operation: `lhs op rhs`.
    BinOp(Box<Expr>, Op, Box<Expr>),
    /// Function call or variable indexing: `name(arg1, arg2, ...)`.
    ///
    /// Disambiguation happens at eval time: if `name` exists in the environment
    /// it is treated as indexing, otherwise as a built-in or user function call.
    Call(String, Vec<Expr>),
    /// Matrix literal: `[row1; row2; ...]` where each row is a list of expressions.
    Matrix(Vec<Vec<Expr>>),
    /// Conjugate transpose: `A'`. For complex scalars, returns the conjugate.
    Transpose(Box<Expr>),
    /// Range expression: `start:stop` or `start:step:stop`.
    /// Evaluates to a 1×N row vector.
    Range(Box<Expr>, Option<Box<Expr>>, Box<Expr>),
    /// Bare `:` used as an all-elements index in `A(:,j)` or `A(i,:)`.
    /// Only valid as an argument inside an indexing expression.
    Colon,
    /// Single-quoted char array literal.
    StrLiteral(String),
    /// Double-quoted string object literal.
    StringObjLiteral(String),
    /// Anonymous function: `@(params) body_expr`.
    ///
    /// At evaluation time this is converted to `Value::Lambda`, capturing the
    /// current environment as a lexical closure.
    Lambda {
        /// Parameter names in declaration order (e.g. `["x", "n"]`).
        params: Vec<String>,
        /// Body expression evaluated when the lambda is called.
        body: Box<Expr>,
        /// Source text for display (e.g. `@(x) x.^2 + 1`), stored at parse time.
        source: String,
    },
    /// Non-conjugate (plain) transpose: `A.'`.
    ///
    /// Transposes without complex conjugation. For real matrices, identical to `A'`.
    /// For complex: `z.'` returns `z` unchanged (no sign flip on imaginary part).
    PlainTranspose(Box<Expr>),
    /// Cell array literal: `{e1, e2, e3}`.
    ///
    /// Evaluates each element and produces `Value::Cell`.
    CellLiteral(Vec<Expr>),
    /// Cell array brace-indexing: `c{i}`.
    ///
    /// The first expression must evaluate to `Value::Cell`; the second is the
    /// 1-based integer index.
    CellIndex(Box<Expr>, Box<Expr>),
    /// Function handle: `@funcname`.
    ///
    /// Produces a `Value::Lambda` that forwards its arguments to the named
    /// built-in or user function.
    FuncHandle(String),
    /// Struct field read: `s.field` or chained `s.a.b` (parsed as `FieldGet(FieldGet(s,"a"),"b")`).
    ///
    /// At eval time the base expression must evaluate to `Value::Struct`.
    FieldGet(Box<Expr>, String),
    /// Package-qualified function call: `pkg.func(args)` or `pkg.sub.func(args)`.
    ///
    /// `segments` holds the dot-separated name components, e.g. `["utils", "my_function"]`.
    /// At eval time:
    /// - If `segments[0]` is in the environment (a struct or callable), the chain is followed
    ///   as field accesses and the final value is called with the given arguments.
    /// - Otherwise, the segments are treated as a package call: the autoload hook searches
    ///   for `+utils/my_function.calc` (or `+utils/+sub/func.calc` for nested packages)
    ///   on the session path and loads the function on demand.
    DotCall(Vec<String>, Vec<Expr>),
    /// Not-a-Time sentinel: `NaT`. Evaluates to `Value::DateTime(f64::NAN)`.
    NaT,
}

/// A binary operator used in [`Expr::BinOp`].
#[derive(Debug, Clone)]
pub enum Op {
    /// Addition: `a + b` or element-wise matrix addition.
    Add,
    /// Subtraction: `a - b` or element-wise matrix subtraction.
    Sub,
    /// Multiplication: scalar `a * b` or matrix product `A * B`.
    Mul,
    /// Division: scalar `a / b` or matrix right-division `A / B` (solves `X * B = A`).
    Div,
    /// Exponentiation: scalar `a ^ b` or matrix power `A ^ n`.
    Pow,
    /// Element-wise multiplication: `A .* B`.
    ElemMul,
    /// Element-wise division: `A ./ B`.
    ElemDiv,
    /// Element-wise exponentiation: `A .^ B`.
    ElemPow,
    // --- Comparison (element-wise, return 0.0/1.0) ---
    /// Equality comparison: `a == b`. Returns 1.0 if equal, 0.0 otherwise.
    Eq,
    /// Inequality comparison: `a ~= b`. Returns 1.0 if not equal, 0.0 otherwise.
    NotEq,
    /// Less-than comparison: `a < b`.
    Lt,
    /// Greater-than comparison: `a > b`.
    Gt,
    /// Less-than-or-equal comparison: `a <= b`.
    LtEq,
    /// Greater-than-or-equal comparison: `a >= b`.
    GtEq,
    // --- Short-circuit logical (scalars only) ---
    /// Short-circuit logical AND: `a && b`. Only evaluates `b` if `a` is truthy.
    And,
    /// Short-circuit logical OR: `a || b`. Only evaluates `b` if `a` is falsy.
    Or,
    // --- Element-wise logical (matrices allowed, no short-circuit) ---
    /// Element-wise logical AND: `A & B`. Evaluates both sides; works on matrices.
    ElemAnd,
    /// Element-wise logical OR: `A | B`. Evaluates both sides; works on matrices.
    ElemOr,
    /// Left division: `A \ b` solves `A*x = b`. Scalar: `a \ b = b / a`.
    LDiv,
}

/// The numeric base used when displaying integer-valued scalars.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Base {
    /// Decimal (base 10) — the default.
    #[default]
    Dec,
    /// Hexadecimal (base 16), prefix `0x` (e.g. `0xff`).
    Hex,
    /// Binary (base 2), prefix `0b` (e.g. `0b1010`).
    Bin,
    /// Octal (base 8), prefix `0o` (e.g. `0o17`).
    Oct,
}

/// Controls how numbers are displayed (MATLAB-compatible format modes).
#[derive(Debug, Clone, PartialEq)]
pub enum FormatMode {
    /// 5 significant digits, auto fixed/scientific (MATLAB `format short`).
    Short,
    /// 15 significant digits, auto fixed/scientific (MATLAB `format long`).
    Long,
    /// Always scientific notation, 4 decimal places — 5 sig digits.
    ShortE,
    /// Always scientific notation, 14 decimal places — 15 sig digits.
    LongE,
    /// Same as `Short` for scalars (MATLAB `format shortG`).
    ShortG,
    /// Same as `Long` for scalars (MATLAB `format longG`).
    LongG,
    /// Fixed 2 decimal places — currency (MATLAB `format bank`).
    Bank,
    /// Rational approximation `p/q` (MATLAB `format rat`).
    Rat,
    /// IEEE 754 hexadecimal bit pattern, 16 uppercase hex digits (MATLAB `format hex`).
    Hex,
    /// Sign character only: `+`, `-`, or ` ` for zero (MATLAB `format +`).
    Plus,
    /// N decimal places, auto fixed/scientific — legacy precision= setting.
    Custom(usize),
}

impl Default for FormatMode {
    fn default() -> Self {
        FormatMode::Custom(10)
    }
}

impl FormatMode {
    /// Human-readable name for display in `config` / status messages.
    pub fn name(&self) -> String {
        match self {
            FormatMode::Short => "short".to_string(),
            FormatMode::Long => "long".to_string(),
            FormatMode::ShortE => "shortE".to_string(),
            FormatMode::LongE => "longE".to_string(),
            FormatMode::ShortG => "shortG".to_string(),
            FormatMode::LongG => "longG".to_string(),
            FormatMode::Bank => "bank".to_string(),
            FormatMode::Rat => "rat".to_string(),
            FormatMode::Hex => "hex".to_string(),
            FormatMode::Plus => "+".to_string(),
            FormatMode::Custom(n) => format!("custom({n})"),
        }
    }
}

/// Evaluates an expression without file I/O context.
/// This is the public API used by tests and non-I/O evaluation paths.
pub fn eval(expr: &Expr, env: &Env) -> Result<Value, String> {
    eval_inner(expr, env, None)
}

/// Evaluates an expression with an I/O context (file descriptor table).
/// Used by the REPL to support `fopen`/`fclose`/`fgetl`/`fgets`/`fprintf(fd,...)`.
pub fn eval_with_io(expr: &Expr, env: &Env, io: &mut IoContext) -> Result<Value, String> {
    eval_inner(expr, env, Some(io))
}

fn eval_inner(expr: &Expr, env: &Env, mut io: Option<&mut IoContext>) -> Result<Value, String> {
    match expr {
        Expr::Number(n) => Ok(Value::Scalar(*n)),
        Expr::Var(name) => env.get(name).cloned().ok_or(()).or_else(|_| {
            // Check the shared global store when the name is declared global in this scope.
            if is_global(name)
                && let Some(val) = global_get(name)
            {
                return Ok(val);
            }
            // 'e' falls back to Euler's number if not defined in env
            if name == "e" {
                Ok(Value::Scalar(std::f64::consts::E))
            } else {
                let hint = suggest_similar(name, env);
                match hint {
                    Some(s) => Err(format!("Undefined variable '{name}'; did you mean '{s}'?")),
                    None => Err(format!("Undefined variable: '{name}'")),
                }
            }
        }),
        Expr::UnaryMinus(e) => match eval_inner(e, env, io)? {
            Value::Void => Err("Unary minus is not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(-n)),
            Value::Matrix(m) => Ok(Value::Matrix(m.mapv(|x| -x))),
            Value::Complex(re, im) => Ok(Value::Complex(-re, -im)),
            Value::Str(s) => match str_to_numeric(&s) {
                Value::Scalar(n) => Ok(Value::Scalar(-n)),
                Value::Matrix(m) => Ok(Value::Matrix(m.mapv(|x| -x))),
                _ => unreachable!(),
            },
            Value::StringObj(_) => {
                Err("Unary minus is not applicable to string objects".to_string())
            }
            Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("Unary minus is not applicable to this type".to_string())
            }
        },
        Expr::UnaryNot(e) => match eval_inner(e, env, io)? {
            Value::Void => Err("Logical NOT is not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(if n == 0.0 { 1.0 } else { 0.0 })),
            Value::Matrix(m) => Ok(Value::Matrix(m.mapv(|x| if x == 0.0 { 1.0 } else { 0.0 }))),
            Value::Complex(re, im) => Ok(Value::Scalar(if re == 0.0 && im == 0.0 {
                1.0
            } else {
                0.0
            })),
            Value::Str(s) => match str_to_numeric(&s) {
                Value::Scalar(n) => Ok(Value::Scalar(if n == 0.0 { 1.0 } else { 0.0 })),
                Value::Matrix(m) => Ok(Value::Matrix(m.mapv(|x| if x == 0.0 { 1.0 } else { 0.0 }))),
                _ => unreachable!(),
            },
            Value::StringObj(_) => {
                Err("Logical NOT is not applicable to string objects".to_string())
            }
            Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("Logical NOT is not applicable to this type".to_string())
            }
        },
        Expr::BinOp(left, op, right) => {
            let l = eval_inner(left, env, io.as_deref_mut())?;
            let r = eval_inner(right, env, io)?;
            eval_binop(l, op, r)
        }
        Expr::Call(name, args) => {
            // try(expr, default) — special form: evaluate expr; on error evaluate default.
            // Arguments are NOT pre-evaluated; lazy semantics.
            if name == "try" && args.len() == 2 {
                return match eval_inner(&args[0], env, io.as_deref_mut()) {
                    Ok(v) => Ok(v),
                    Err(msg) => {
                        set_last_err(&msg);
                        eval_inner(&args[1], env, io.as_deref_mut())
                    }
                };
            }

            // If the name resolves to a variable in env, check its type.
            // User functions (Lambda, Function) are called; other values are indexed.
            // Variables shadow built-in function names (Octave semantics).
            if let Some(val) = env.get(name).cloned() {
                match &val {
                    Value::Lambda(f) => {
                        // Evaluate arguments and call the closure directly.
                        // Empty call → inject ans (convenience: sq() = sq(ans)).
                        let mut evaled = Vec::with_capacity(args.len().max(1));
                        for a in args {
                            evaled.push(eval_inner(a, env, io.as_deref_mut())?);
                        }
                        if evaled.is_empty() {
                            evaled.push(env.get("ans").cloned().unwrap_or(Value::Scalar(0.0)));
                        }
                        let f = f.clone();
                        return f.0(&evaled, io);
                    }
                    Value::Function { .. } => {
                        // Evaluate arguments and dispatch to the registered hook in exec.rs.
                        // User functions receive the raw arg list — NO ans injection. Empty call
                        // means no arguments (varargin = {}), matching MATLAB semantics.
                        let mut evaled = Vec::with_capacity(args.len());
                        for a in args {
                            evaled.push(eval_inner(a, env, io.as_deref_mut())?);
                        }
                        return match io.as_deref_mut() {
                            Some(io_ref) => FN_CALL_HOOK.with(|c| match c.get() {
                                Some(hook) => hook(name, &val, &evaled, env, io_ref),
                                None => Err(format!(
                                    "'{name}': user function execution not initialized \
                                         (call exec::init() first)"
                                )),
                            }),
                            None => {
                                // No I/O context — create a temporary one (functions that do
                                // file I/O in this path will silently fail to open files).
                                let mut tmp_io = IoContext::new();
                                FN_CALL_HOOK.with(|c| match c.get() {
                                    Some(hook) => hook(name, &val, &evaled, env, &mut tmp_io),
                                    None => Err(format!(
                                        "'{name}': user function execution not initialized"
                                    )),
                                })
                            }
                        };
                    }
                    _ => return eval_index(&val, args, env),
                }
            }
            // Autoload: if name is not in env and not yet tried as a builtin,
            // ask exec.rs to search for <name>.calc / <name>.m on the path.
            // If found, the function is inserted into env and we call it directly.
            // Autoload: search for <name>.calc / <name>.m if not in env or cache.
            let cached = AUTOLOAD_CACHE.with(|c| c.borrow().get(name).cloned());
            let autoloaded_val = cached.or_else(|| {
                let loaded = AUTOLOAD_HOOK
                    .with(|c| c.get())
                    .is_some_and(|hook| hook(name));
                if loaded {
                    AUTOLOAD_CACHE.with(|c| c.borrow().get(name).cloned())
                } else {
                    None
                }
            });
            if let Some(val) = autoloaded_val {
                let mut evaled = Vec::with_capacity(args.len());
                for a in args {
                    evaled.push(eval_inner(a, env, io.as_deref_mut())?);
                }
                return match io.as_deref_mut() {
                    Some(io_ref) => FN_CALL_HOOK.with(|c| match c.get() {
                        Some(hook) => hook(name, &val, &evaled, env, io_ref),
                        None => Err(format!("'{name}': exec::init() not called")),
                    }),
                    None => {
                        let mut tmp_io = IoContext::new();
                        FN_CALL_HOOK.with(|c| match c.get() {
                            Some(hook) => hook(name, &val, &evaled, env, &mut tmp_io),
                            None => Err(format!("'{name}': exec::init() not called")),
                        })
                    }
                };
            }

            // Builtin path: empty call → inject ans (sqrt() = sqrt(ans)).
            let mut evaled = Vec::with_capacity(args.len().max(1));
            for a in args {
                evaled.push(eval_inner(a, env, io.as_deref_mut())?);
            }
            // Don't inject ans for functions that take explicit struct/cell args
            // or constructors where zero args is meaningful.
            let no_ans_inject = matches!(
                name.as_str(),
                "struct"
                    | "fieldnames"
                    | "isfield"
                    | "rmfield"
                    | "isstruct"
                    | "cell"
                    | "iscell"
                    | "isempty"
                    | "cellfun"
                    | "error"
                    | "warning"
                    | "lasterr"
                    | "pcall"
                    | "rand"
                    | "randn"
                    | "rng"
            );
            if evaled.is_empty() && !no_ans_inject {
                evaled.push(env.get("ans").cloned().unwrap_or(Value::Scalar(0.0)));
            }
            call_builtin(name, &evaled, env, io)
        }

        Expr::Lambda {
            params,
            body,
            source,
        } => {
            // Capture the current environment and body expression at definition time.
            // The resulting Value::Lambda is a closure that binds params on each call.
            let captured_env = env.clone();
            let captured_params = params.clone();
            let captured_body = *body.clone();
            let src = source.clone();
            let lambda = LambdaFn(
                std::rc::Rc::new(move |args: &[Value], io: Option<&mut IoContext>| {
                    // Allow up to params.len()+1 args: the parser injects `ans` for empty f() calls.
                    let effective = if args.len() > captured_params.len() {
                        if args.len() > captured_params.len() + 1 {
                            return Err(format!(
                                "Lambda: too many arguments (expected at most {}, got {})",
                                captured_params.len(),
                                args.len()
                            ));
                        }
                        &args[..captured_params.len()]
                    } else {
                        args
                    };
                    let mut local_env = captured_env.clone();
                    for (p, a) in captured_params.iter().zip(effective.iter()) {
                        local_env.insert(p.clone(), a.clone());
                    }
                    local_env.insert("nargin".to_string(), Value::Scalar(effective.len() as f64));
                    eval_inner(&captured_body, &local_env, io)
                }),
                src,
            );
            Ok(Value::Lambda(lambda))
        }
        Expr::CellLiteral(elems) => {
            let mut vals = Vec::with_capacity(elems.len());
            for e in elems {
                vals.push(eval_inner(e, env, io.as_deref_mut())?);
            }
            Ok(Value::Cell(vals))
        }
        Expr::CellIndex(cell_expr, idx_expr) => {
            let cell = eval_inner(cell_expr, env, io.as_deref_mut())?;
            let idx = eval_inner(idx_expr, env, io)?;
            match (cell, idx) {
                (Value::Cell(v), Value::Scalar(i)) => {
                    let i = i as isize;
                    if i < 1 || i as usize > v.len() {
                        Err(format!("Cell index {} out of range (1..{})", i, v.len()))
                    } else {
                        Ok(v[(i - 1) as usize].clone())
                    }
                }
                (Value::Cell(_), _) => Err("Cell index must be a scalar integer".to_string()),
                _ => Err("Brace indexing '{}' is only valid on cell arrays".to_string()),
            }
        }
        Expr::FieldGet(base_expr, field) => {
            let base_val = eval_inner(base_expr, env, io)?;
            match base_val {
                Value::Struct(map) => map
                    .get(field)
                    .cloned()
                    .ok_or_else(|| format!("No field '{field}' in struct")),
                // s.field on a struct array — collect field values across all elements
                Value::StructArray(arr) => {
                    let mut values: Vec<Value> = Vec::with_capacity(arr.len());
                    for (idx, elem) in arr.iter().enumerate() {
                        let v = elem.get(field).cloned().ok_or_else(|| {
                            format!("No field '{field}' in struct array element {}", idx + 1)
                        })?;
                        values.push(v);
                    }
                    // If all values are scalars, return a 1×N matrix; otherwise a cell.
                    let all_scalar = values.iter().all(|v| matches!(v, Value::Scalar(_)));
                    if all_scalar {
                        let nums: Vec<f64> = values
                            .into_iter()
                            .map(|v| {
                                if let Value::Scalar(n) = v {
                                    n
                                } else {
                                    unreachable!()
                                }
                            })
                            .collect();
                        let n = nums.len();
                        Ok(Value::Matrix(Array2::from_shape_vec((1, n), nums).unwrap()))
                    } else {
                        Ok(Value::Cell(values))
                    }
                }
                _ => Err(format!(
                    "Cannot access field '{field}' on a non-struct value"
                )),
            }
        }
        Expr::DotCall(segs, args) => {
            let qualified = segs.join(".");
            // If the head segment is a variable, follow the field chain and call the result.
            if let Some(head_val) = env.get(&segs[0]).cloned() {
                let mut val = head_val;
                for field in &segs[1..] {
                    val = match val {
                        Value::Struct(ref map) => map
                            .get(field)
                            .cloned()
                            .ok_or_else(|| format!("No field '{field}' in struct"))?,
                        _ => {
                            return Err(format!(
                                "Cannot access field '{field}' on a non-struct value"
                            ));
                        }
                    };
                }
                let mut evaled = Vec::with_capacity(args.len());
                for a in args {
                    evaled.push(eval_inner(a, env, io.as_deref_mut())?);
                }
                return match val {
                    Value::Lambda(f) => {
                        if evaled.is_empty() {
                            evaled.push(env.get("ans").cloned().unwrap_or(Value::Scalar(0.0)));
                        }
                        f.0(&evaled, io)
                    }
                    Value::Function { .. } => match io.as_deref_mut() {
                        Some(io_ref) => FN_CALL_HOOK.with(|c| match c.get() {
                            Some(hook) => hook(&qualified, &val, &evaled, env, io_ref),
                            None => Err(format!("'{qualified}': exec::init() not called")),
                        }),
                        None => {
                            let mut tmp_io = IoContext::new();
                            FN_CALL_HOOK.with(|c| match c.get() {
                                Some(hook) => hook(&qualified, &val, &evaled, env, &mut tmp_io),
                                None => Err(format!("'{qualified}': exec::init() not called")),
                            })
                        }
                    },
                    _ => Err(format!("'{qualified}': not a callable")),
                };
            }
            // Package call: autoload from +pkg/func.calc then invoke.
            let cached = AUTOLOAD_CACHE.with(|c| c.borrow().get(&qualified).cloned());
            let autoloaded_val = cached.or_else(|| {
                let loaded = AUTOLOAD_HOOK
                    .with(|c| c.get())
                    .is_some_and(|hook| hook(&qualified));
                if loaded {
                    AUTOLOAD_CACHE.with(|c| c.borrow().get(&qualified).cloned())
                } else {
                    None
                }
            });
            if let Some(val) = autoloaded_val {
                let mut evaled = Vec::with_capacity(args.len());
                for a in args {
                    evaled.push(eval_inner(a, env, io.as_deref_mut())?);
                }
                return match io.as_deref_mut() {
                    Some(io_ref) => FN_CALL_HOOK.with(|c| match c.get() {
                        Some(hook) => hook(&qualified, &val, &evaled, env, io_ref),
                        None => Err(format!("'{qualified}': exec::init() not called")),
                    }),
                    None => {
                        let mut tmp_io = IoContext::new();
                        FN_CALL_HOOK.with(|c| match c.get() {
                            Some(hook) => hook(&qualified, &val, &evaled, env, &mut tmp_io),
                            None => Err(format!("'{qualified}': exec::init() not called")),
                        })
                    }
                };
            }
            Err(format!("Unknown package function: '{qualified}'"))
        }
        Expr::FuncHandle(name) => {
            let name = name.clone();
            let captured_env = env.clone();
            let src = format!("@{name}");
            let lambda = LambdaFn(
                std::rc::Rc::new(move |args: &[Value], io: Option<&mut IoContext>| {
                    // First try the environment (user-defined function), then fall back to builtin.
                    if let Some(f) = captured_env.get(&name) {
                        let f = f.clone();
                        call_function_value(&f, args, io)
                    } else {
                        call_builtin(&name, args, &captured_env, io)
                    }
                }),
                src,
            );
            Ok(Value::Lambda(lambda))
        }
        Expr::PlainTranspose(e) => match eval_inner(e, env, io)? {
            Value::Void => Err("Transpose is not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(n)),
            Value::Matrix(m) => Ok(Value::Matrix(m.t().to_owned())),
            // Plain transpose: no conjugation — imaginary part unchanged
            Value::Complex(re, im) => Ok(Value::Complex(re, im)),
            Value::Str(s) => Ok(Value::Str(s)),
            Value::StringObj(s) => Ok(Value::StringObj(s)),
            // Arrays: orientation is ignored (Vec<f64> is always 1-D), return as-is.
            v @ (Value::DateTimeArray(_) | Value::DurationArray(_)) => Ok(v),
            Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_) => Err("Transpose is not applicable to this type".to_string()),
        },
        Expr::Colon => Err("':' is only valid inside index expressions".to_string()),
        Expr::NaT => Ok(Value::DateTime(f64::NAN)),
        Expr::Matrix(rows) => {
            if rows.is_empty() {
                return Ok(Value::Matrix(Array2::<f64>::zeros((0, 0))));
            }

            // Pass 1: evaluate all elements, skipping empty rows.
            let mut evaluated: Vec<Vec<Value>> = Vec::with_capacity(rows.len());
            for row in rows {
                if row.is_empty() {
                    continue;
                }
                let mut ev_row: Vec<Value> = Vec::with_capacity(row.len());
                for elem_expr in row {
                    ev_row.push(eval_inner(elem_expr, env, io.as_deref_mut())?);
                }
                evaluated.push(ev_row);
            }
            if evaluated.is_empty() {
                return Ok(Value::Matrix(Array2::<f64>::zeros((0, 0))));
            }

            // Pass 2: dispatch on the type of the first element.
            enum MatKind {
                Numeric,
                DateTime,
                Duration,
            }
            let kind = match &evaluated[0][0] {
                Value::Scalar(_) | Value::Matrix(_) => MatKind::Numeric,
                Value::DateTime(_) | Value::DateTimeArray(_) => MatKind::DateTime,
                Value::Duration(_) | Value::DurationArray(_) => MatKind::Duration,
                Value::Void => {
                    return Err("Void value cannot be used in matrix literal".to_string());
                }
                Value::Complex(_, _) => {
                    return Err(
                        "Complex elements in matrix literals are not supported yet".to_string()
                    );
                }
                Value::Str(_) | Value::StringObj(_) => {
                    return Err("String elements in matrix literals are not supported".to_string());
                }
                Value::Lambda(_)
                | Value::Function { .. }
                | Value::Tuple(_)
                | Value::Cell(_)
                | Value::Struct(_)
                | Value::StructArray(_) => {
                    return Err("This type cannot be used in matrix literals".to_string());
                }
            };

            match kind {
                MatKind::DateTime => {
                    let mut ts: Vec<f64> = Vec::new();
                    for ev_row in &evaluated {
                        for val in ev_row {
                            match val {
                                Value::DateTime(t) => ts.push(*t),
                                Value::DateTimeArray(v) => ts.extend_from_slice(v),
                                _ => {
                                    return Err(
                                        "Matrix literal: cannot mix datetime with other types"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                    Ok(Value::DateTimeArray(ts))
                }
                MatKind::Duration => {
                    let mut sv: Vec<f64> = Vec::new();
                    for ev_row in &evaluated {
                        for val in ev_row {
                            match val {
                                Value::Duration(s) => sv.push(*s),
                                Value::DurationArray(v) => sv.extend_from_slice(v),
                                _ => {
                                    return Err(
                                        "Matrix literal: cannot mix duration with other types"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                    Ok(Value::DurationArray(sv))
                }
                MatKind::Numeric => {
                    // Each row is horizontally concatenated into an Array2 block;
                    // blocks are then vertically concatenated.
                    let mut row_blocks: Vec<Array2<f64>> = Vec::with_capacity(evaluated.len());
                    for ev_row in &evaluated {
                        let mut elem_mats: Vec<Array2<f64>> = Vec::with_capacity(ev_row.len());
                        for val in ev_row {
                            match val {
                                Value::Scalar(n) => {
                                    elem_mats.push(Array2::from_elem((1, 1), *n));
                                }
                                Value::Matrix(m) => elem_mats.push(m.clone()),
                                Value::Void => {
                                    return Err(
                                        "Void value cannot be used in matrix literal".to_string()
                                    );
                                }
                                Value::Complex(_, _) => {
                                    return Err(
                                        "Complex elements in matrix literals are not supported yet"
                                            .to_string(),
                                    );
                                }
                                Value::Str(_) | Value::StringObj(_) => {
                                    return Err(
                                        "String elements in matrix literals are not supported"
                                            .to_string(),
                                    );
                                }
                                _ => {
                                    return Err(
                                        "This type cannot be used in matrix literals".to_string()
                                    );
                                }
                            }
                        }
                        let nrows = elem_mats[0].nrows();
                        for (i, m) in elem_mats.iter().enumerate().skip(1) {
                            if m.nrows() != nrows {
                                return Err(format!(
                                    "Matrix row height mismatch: expected {} rows, element {} has {} rows",
                                    nrows,
                                    i + 1,
                                    m.nrows()
                                ));
                            }
                        }
                        let ncols: usize = elem_mats.iter().map(|m| m.ncols()).sum();
                        let mut flat: Vec<f64> = Vec::with_capacity(nrows * ncols);
                        for r in 0..nrows {
                            for m in &elem_mats {
                                flat.extend(m.row(r).iter().copied());
                            }
                        }
                        row_blocks.push(
                            Array2::from_shape_vec((nrows, ncols), flat)
                                .map_err(|e| format!("Matrix shape error: {e}"))?,
                        );
                    }
                    if row_blocks.is_empty() {
                        return Ok(Value::Matrix(Array2::<f64>::zeros((0, 0))));
                    }
                    let ncols = row_blocks[0].ncols();
                    if ncols == 0 {
                        let total_rows: usize = row_blocks.iter().map(|b| b.nrows()).sum();
                        return Ok(Value::Matrix(Array2::zeros((total_rows, 0))));
                    }
                    for (i, blk) in row_blocks.iter().enumerate().skip(1) {
                        if blk.ncols() != ncols {
                            return Err(format!(
                                "Matrix column count mismatch: expected {} columns, row {} has {} columns",
                                ncols,
                                i + 1,
                                blk.ncols()
                            ));
                        }
                    }
                    let total_rows: usize = row_blocks.iter().map(|b| b.nrows()).sum();
                    let mut flat: Vec<f64> = Vec::with_capacity(total_rows * ncols);
                    for blk in &row_blocks {
                        flat.extend(blk.iter().copied());
                    }
                    let m = Array2::from_shape_vec((total_rows, ncols), flat)
                        .map_err(|e| format!("Matrix shape error: {e}"))?;
                    Ok(Value::Matrix(m))
                }
            }
        }
        Expr::Transpose(e) => match eval_inner(e, env, io)? {
            Value::Void => Err("Transpose is not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(n)),
            Value::Matrix(m) => Ok(Value::Matrix(m.t().to_owned())),
            Value::Complex(re, im) => Ok(Value::Complex(re, -im)),
            // Transpose of a char array or string object: return as-is (1×N not fully supported)
            Value::Str(s) => Ok(Value::Str(s)),
            Value::StringObj(s) => Ok(Value::StringObj(s)),
            // Arrays: orientation is ignored (Vec<f64> is always 1-D), return as-is.
            v @ (Value::DateTimeArray(_) | Value::DurationArray(_)) => Ok(v),
            Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_) => Err("Transpose is not applicable to this type".to_string()),
        },
        Expr::StrLiteral(s) => Ok(Value::Str(s.clone())),
        Expr::StringObjLiteral(s) => Ok(Value::StringObj(s.clone())),
        Expr::Range(start_expr, step_expr, stop_expr) => {
            let start = match eval_inner(start_expr, env, io.as_deref_mut())? {
                Value::Scalar(n) => n,
                _ => return Err("Range bounds must be real scalars".to_string()),
            };
            let stop = match eval_inner(stop_expr, env, io.as_deref_mut())? {
                Value::Scalar(n) => n,
                _ => return Err("Range bounds must be real scalars".to_string()),
            };
            let step = match step_expr {
                None => 1.0,
                Some(s) => match eval_inner(s, env, io)? {
                    Value::Scalar(n) => n,
                    _ => return Err("Range step must be a real scalar".to_string()),
                },
            };
            if step == 0.0 {
                return Err("Range step cannot be zero".to_string());
            }
            let n_float = (stop - start) / step;
            if n_float < -1e-10 {
                // Empty range: step points in the wrong direction
                return Ok(Value::Matrix(Array2::zeros((1, 0))));
            }
            let n = (n_float + 1e-10).floor() as usize + 1;
            let vals: Vec<f64> = (0..n).map(|i| start + i as f64 * step).collect();
            let m =
                Array2::from_shape_vec((1, n), vals).map_err(|e| format!("Range error: {e}"))?;
            Ok(Value::Matrix(m))
        }
    }
}

fn eval_binop(l: Value, op: &Op, r: Value) -> Result<Value, String> {
    match (l, r) {
        (Value::Void, _) | (_, Value::Void) => {
            Err("Cannot apply operator to void value".to_string())
        }
        // --- String object operations ---
        (Value::StringObj(a), Value::StringObj(b)) => match op {
            Op::Add => Ok(Value::StringObj(a + &b)),
            Op::Eq => Ok(Value::Scalar(bool_to_f64(a == b))),
            Op::NotEq => Ok(Value::Scalar(bool_to_f64(a != b))),
            _ => Err("Operator not supported on string objects".to_string()),
        },
        // Char array: convert to numeric, re-dispatch
        (Value::Str(s), r) => eval_binop(str_to_numeric(&s), op, r),
        (l, Value::Str(s)) => eval_binop(l, op, str_to_numeric(&s)),
        // String object mixed with other types: error
        (Value::StringObj(_), _) | (_, Value::StringObj(_)) => {
            Err("String object cannot be combined with non-string values".to_string())
        }
        // Functions, tuples, cell arrays, structs, and struct arrays are not numeric
        (Value::Lambda(_), _)
        | (_, Value::Lambda(_))
        | (Value::Function { .. }, _)
        | (_, Value::Function { .. })
        | (Value::Tuple(_), _)
        | (_, Value::Tuple(_))
        | (Value::Cell(_), _)
        | (_, Value::Cell(_))
        | (Value::Struct(_), _)
        | (_, Value::Struct(_))
        | (Value::StructArray(_), _)
        | (_, Value::StructArray(_)) => Err("Cannot apply operator to a struct value".to_string()),
        // --- DateTime / Duration arithmetic ---
        // datetime + duration → datetime
        (Value::DateTime(t), Value::Duration(d)) => match op {
            Op::Add => Ok(Value::DateTime(t + d)),
            Op::Sub => Ok(Value::DateTime(t - d)),
            _ => Err("Unsupported operator between datetime and duration".to_string()),
        },
        // duration + datetime → datetime (commutative add only)
        (Value::Duration(d), Value::DateTime(t)) => match op {
            Op::Add => Ok(Value::DateTime(t + d)),
            _ => Err("Unsupported operator between duration and datetime".to_string()),
        },
        // datetime - datetime → duration
        (Value::DateTime(t1), Value::DateTime(t2)) => match op {
            Op::Sub => Ok(Value::Duration(t1 - t2)),
            Op::Eq => Ok(Value::Scalar(bool_to_f64(
                (t1 - t2).abs() < 1e-9 || (t1.is_nan() && t2.is_nan()),
            ))),
            Op::NotEq => Ok(Value::Scalar(bool_to_f64(
                (t1 - t2).abs() >= 1e-9 && !(t1.is_nan() && t2.is_nan()),
            ))),
            Op::Lt => Ok(Value::Scalar(bool_to_f64(t1 < t2))),
            Op::Gt => Ok(Value::Scalar(bool_to_f64(t1 > t2))),
            Op::LtEq => Ok(Value::Scalar(bool_to_f64(t1 <= t2))),
            Op::GtEq => Ok(Value::Scalar(bool_to_f64(t1 >= t2))),
            _ => Err("Unsupported operator between two datetimes".to_string()),
        },
        // duration ± duration → duration; duration */ scalar → duration; duration / duration → scalar
        (Value::Duration(d1), Value::Duration(d2)) => match op {
            Op::Add => Ok(Value::Duration(d1 + d2)),
            Op::Sub => Ok(Value::Duration(d1 - d2)),
            Op::Div | Op::ElemDiv => Ok(Value::Scalar(d1 / d2)),
            Op::Eq => Ok(Value::Scalar(bool_to_f64((d1 - d2).abs() < 1e-9))),
            Op::NotEq => Ok(Value::Scalar(bool_to_f64((d1 - d2).abs() >= 1e-9))),
            Op::Lt => Ok(Value::Scalar(bool_to_f64(d1 < d2))),
            Op::Gt => Ok(Value::Scalar(bool_to_f64(d1 > d2))),
            Op::LtEq => Ok(Value::Scalar(bool_to_f64(d1 <= d2))),
            Op::GtEq => Ok(Value::Scalar(bool_to_f64(d1 >= d2))),
            _ => Err("Unsupported operator between two durations".to_string()),
        },
        (Value::Duration(d), Value::Scalar(s)) => match op {
            Op::Mul | Op::ElemMul => Ok(Value::Duration(d * s)),
            Op::Div | Op::ElemDiv => Ok(Value::Duration(d / s)),
            _ => Err("Unsupported operator between duration and scalar".to_string()),
        },
        (Value::Scalar(s), Value::Duration(d)) => match op {
            Op::Mul | Op::ElemMul => Ok(Value::Duration(s * d)),
            _ => Err("Unsupported operator between scalar and duration".to_string()),
        },
        // DateTime/Duration + arrays
        (Value::DateTime(t), Value::DurationArray(dv)) => match op {
            Op::Add => Ok(Value::DateTimeArray(dv.iter().map(|d| t + d).collect())),
            Op::Sub => Ok(Value::DateTimeArray(dv.iter().map(|d| t - d).collect())),
            _ => Err("Unsupported operator between datetime and duration array".to_string()),
        },
        (Value::DurationArray(dv), Value::DateTime(t)) => match op {
            Op::Add => Ok(Value::DateTimeArray(dv.iter().map(|d| t + d).collect())),
            _ => Err("Unsupported operator between duration array and datetime".to_string()),
        },
        (Value::DateTimeArray(tv), Value::Duration(d)) => match op {
            Op::Add => Ok(Value::DateTimeArray(tv.iter().map(|t| t + d).collect())),
            Op::Sub => Ok(Value::DateTimeArray(tv.iter().map(|t| t - d).collect())),
            _ => Err("Unsupported operator between datetime array and duration".to_string()),
        },
        (Value::DateTimeArray(tv), Value::DurationArray(dv)) => match op {
            Op::Add if tv.len() == dv.len() => Ok(Value::DateTimeArray(
                tv.iter().zip(&dv).map(|(t, d)| t + d).collect(),
            )),
            Op::Sub if tv.len() == dv.len() => Ok(Value::DateTimeArray(
                tv.iter().zip(&dv).map(|(t, d)| t - d).collect(),
            )),
            _ => Err("Unsupported or mismatched datetime/duration array operation".to_string()),
        },
        (Value::DateTimeArray(tv1), Value::DateTimeArray(tv2)) => match op {
            Op::Sub if tv1.len() == tv2.len() => Ok(Value::DurationArray(
                tv1.iter().zip(&tv2).map(|(a, b)| a - b).collect(),
            )),
            _ => Err("Unsupported operator between two datetime arrays".to_string()),
        },
        (Value::DurationArray(dv), Value::Scalar(s)) => match op {
            Op::Mul | Op::ElemMul => Ok(Value::DurationArray(dv.iter().map(|d| d * s).collect())),
            Op::Div | Op::ElemDiv => Ok(Value::DurationArray(dv.iter().map(|d| d / s).collect())),
            _ => Err("Unsupported operator between duration array and scalar".to_string()),
        },
        (Value::Scalar(s), Value::DurationArray(dv)) => match op {
            Op::Mul | Op::ElemMul => Ok(Value::DurationArray(dv.iter().map(|d| s * d).collect())),
            _ => Err("Unsupported operator between scalar and duration array".to_string()),
        },
        // Catch-all: DateTime/Duration mixed with unsupported types
        (Value::DateTime(_), _)
        | (_, Value::DateTime(_))
        | (Value::Duration(_), _)
        | (_, Value::Duration(_))
        | (Value::DateTimeArray(_), _)
        | (_, Value::DateTimeArray(_))
        | (Value::DurationArray(_), _)
        | (_, Value::DurationArray(_)) => {
            Err("Unsupported operation on datetime or duration value".to_string())
        }
        // --- Complex arithmetic ---
        (Value::Complex(re1, im1), Value::Complex(re2, im2)) => {
            complex_binop(re1, im1, op, re2, im2)
        }
        (Value::Complex(re, im), Value::Scalar(s)) => complex_binop(re, im, op, s, 0.0),
        (Value::Scalar(s), Value::Complex(re, im)) => complex_binop(s, 0.0, op, re, im),
        (Value::Complex(_, _), Value::Matrix(_)) | (Value::Matrix(_), Value::Complex(_, _)) => {
            Err("Operations between complex scalars and matrices are not supported".to_string())
        }
        (Value::Scalar(lv), Value::Scalar(rv)) => {
            let result = match op {
                Op::Add => lv + rv,
                Op::Sub => lv - rv,
                Op::Mul | Op::ElemMul => lv * rv,
                Op::Div | Op::ElemDiv => {
                    if rv == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    lv / rv
                }
                Op::LDiv => {
                    if lv == 0.0 {
                        return Err("Left division by zero (a \\ b requires a ≠ 0)".to_string());
                    }
                    rv / lv
                }
                Op::Pow | Op::ElemPow => lv.powf(rv),
                Op::Eq => bool_to_f64(lv == rv),
                Op::NotEq => bool_to_f64(lv != rv),
                Op::Lt => bool_to_f64(lv < rv),
                Op::Gt => bool_to_f64(lv > rv),
                Op::LtEq => bool_to_f64(lv <= rv),
                Op::GtEq => bool_to_f64(lv >= rv),
                Op::And | Op::ElemAnd => bool_to_f64(lv != 0.0 && rv != 0.0),
                Op::Or | Op::ElemOr => bool_to_f64(lv != 0.0 || rv != 0.0),
            };
            Ok(Value::Scalar(result))
        }
        (Value::Matrix(lm), Value::Matrix(rm)) => match op {
            Op::Add => {
                check_same_shape(&lm, &rm)?;
                Ok(Value::Matrix(&lm + &rm))
            }
            Op::Sub => {
                check_same_shape(&lm, &rm)?;
                Ok(Value::Matrix(&lm - &rm))
            }
            Op::Mul => {
                if lm.ncols() != rm.nrows() {
                    return Err(format!(
                        "Inner dimensions must agree: {}x{} * {}x{}",
                        lm.nrows(),
                        lm.ncols(),
                        rm.nrows(),
                        rm.ncols()
                    ));
                }
                Ok(Value::Matrix(lm.dot(&rm)))
            }
            Op::ElemMul => {
                check_same_shape(&lm, &rm)?;
                Ok(Value::Matrix(&lm * &rm))
            }
            Op::ElemDiv => {
                check_same_shape(&lm, &rm)?;
                Ok(Value::Matrix(&lm / &rm))
            }
            Op::ElemPow => {
                check_same_shape(&lm, &rm)?;
                Ok(Value::Matrix(
                    ndarray::Zip::from(&lm)
                        .and(&rm)
                        .map_collect(|a, b| a.powf(*b)),
                ))
            }
            Op::Eq | Op::NotEq | Op::Lt | Op::Gt | Op::LtEq | Op::GtEq => {
                check_same_shape(&lm, &rm)?;
                Ok(Value::Matrix(
                    ndarray::Zip::from(&lm)
                        .and(&rm)
                        .map_collect(|a, b| bool_to_f64(cmp_op(op, *a, *b))),
                ))
            }
            Op::And | Op::Or | Op::ElemAnd | Op::ElemOr => {
                check_same_shape(&lm, &rm)?;
                Ok(Value::Matrix(
                    ndarray::Zip::from(&lm)
                        .and(&rm)
                        .map_collect(|a, b| bool_to_f64(cmp_op(op, *a, *b))),
                ))
            }
            Op::Div => Err("Matrix / Matrix: use inv(B)*A or A*inv(B)".to_string()),
            Op::LDiv => Ok(Value::Matrix(solve_linear(&lm, &rm)?)),
            Op::Pow => Err("Matrix ^ Matrix: not supported".to_string()),
        },
        (Value::Scalar(s), Value::Matrix(m)) => match op {
            Op::Add => Ok(Value::Matrix(s + &m)),
            Op::Sub => Ok(Value::Matrix(m.mapv(|x| s - x))),
            Op::Mul | Op::ElemMul => Ok(Value::Matrix(s * &m)),
            Op::Div => Err("Scalar / Matrix: not supported".to_string()),
            Op::ElemDiv => Err("Scalar ./ Matrix: not supported".to_string()),
            Op::LDiv => {
                if s == 0.0 {
                    return Err("Left division by zero (a \\ B requires a ≠ 0)".to_string());
                }
                Ok(Value::Matrix(m.mapv(|x| x / s)))
            }
            Op::Pow | Op::ElemPow => Ok(Value::Matrix(m.mapv(|x| s.powf(x)))),
            Op::Eq
            | Op::NotEq
            | Op::Lt
            | Op::Gt
            | Op::LtEq
            | Op::GtEq
            | Op::And
            | Op::Or
            | Op::ElemAnd
            | Op::ElemOr => Ok(Value::Matrix(m.mapv(|x| bool_to_f64(cmp_op(op, s, x))))),
        },
        (Value::Matrix(m), Value::Scalar(s)) => match op {
            Op::Add => Ok(Value::Matrix(&m + s)),
            Op::Sub => Ok(Value::Matrix(&m - s)),
            Op::Mul | Op::ElemMul => Ok(Value::Matrix(&m * s)),
            Op::Div | Op::ElemDiv => Ok(Value::Matrix(m.mapv(|x| x / s))),
            Op::LDiv => {
                let b = Array2::from_elem((m.nrows(), 1), s);
                Ok(Value::Matrix(solve_linear(&m, &b)?))
            }
            Op::Pow | Op::ElemPow => Ok(Value::Matrix(m.mapv(|x| x.powf(s)))),
            Op::Eq
            | Op::NotEq
            | Op::Lt
            | Op::Gt
            | Op::LtEq
            | Op::GtEq
            | Op::And
            | Op::Or
            | Op::ElemAnd
            | Op::ElemOr => Ok(Value::Matrix(m.mapv(|x| bool_to_f64(cmp_op(op, x, s))))),
        },
    }
}

#[inline]
fn bool_to_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

/// Applies a comparison or logical op to two scalar values.
fn cmp_op(op: &Op, a: f64, b: f64) -> bool {
    match op {
        Op::Eq => a == b,
        Op::NotEq => a != b,
        Op::Lt => a < b,
        Op::Gt => a > b,
        Op::LtEq => a <= b,
        Op::GtEq => a >= b,
        Op::And | Op::ElemAnd => a != 0.0 && b != 0.0,
        Op::Or | Op::ElemOr => a != 0.0 || b != 0.0,
        _ => unreachable!(),
    }
}

/// Performs binary operations on two complex numbers `(re1+im1*i) OP (re2+im2*i)`.
fn complex_binop(re1: f64, im1: f64, op: &Op, re2: f64, im2: f64) -> Result<Value, String> {
    match op {
        Op::Add => Ok(make_complex(re1 + re2, im1 + im2)),
        Op::Sub => Ok(make_complex(re1 - re2, im1 - im2)),
        Op::Mul | Op::ElemMul => {
            // (a+bi)(c+di) = (ac-bd) + (ad+bc)i
            Ok(make_complex(re1 * re2 - im1 * im2, re1 * im2 + im1 * re2))
        }
        Op::Div | Op::ElemDiv => {
            // (a+bi)/(c+di) = ((ac+bd) + (bc-ad)i) / (c²+d²)
            let denom = re2 * re2 + im2 * im2;
            if denom == 0.0 {
                return Err("Division by zero (complex)".to_string());
            }
            Ok(make_complex(
                (re1 * re2 + im1 * im2) / denom,
                (im1 * re2 - re1 * im2) / denom,
            ))
        }
        Op::Pow | Op::ElemPow => {
            let r1 = (re1 * re1 + im1 * im1).sqrt();
            if r1 == 0.0 {
                if re2 > 0.0 {
                    return Ok(Value::Scalar(0.0));
                }
                return Ok(Value::Complex(f64::NAN, f64::NAN));
            }
            // For integer exponents with zero imaginary part, use repeated multiplication
            // to avoid polar-form floating-point error (e.g. i^2 = -1 exactly).
            if im2 == 0.0 && re2.fract() == 0.0 && re2.abs() < 1_000_000.0 {
                let n = re2 as i64;
                if n == 0 {
                    return Ok(Value::Scalar(1.0));
                }
                // positive power: repeated squaring
                let abs_n = n.unsigned_abs();
                let (mut rr, mut ri) = (1.0_f64, 0.0_f64);
                let (mut br, mut bi) = (re1, im1);
                let mut exp = abs_n;
                while exp > 0 {
                    if exp & 1 == 1 {
                        let nr = rr * br - ri * bi;
                        let ni = rr * bi + ri * br;
                        rr = nr;
                        ri = ni;
                    }
                    let nr = br * br - bi * bi;
                    let ni = 2.0 * br * bi;
                    br = nr;
                    bi = ni;
                    exp >>= 1;
                }
                if n < 0 {
                    // invert: 1/(rr+ri*i)
                    let denom = rr * rr + ri * ri;
                    return Ok(make_complex(rr / denom, -ri / denom));
                }
                return Ok(make_complex(rr, ri));
            }
            // General case: via polar form exp((c+di) * ln(a+bi))
            let theta1 = im1.atan2(re1);
            let ln_r1 = r1.ln();
            let exp_re = re2 * ln_r1 - im2 * theta1;
            let exp_im = im2 * ln_r1 + re2 * theta1;
            let mag = exp_re.exp();
            Ok(make_complex(mag * exp_im.cos(), mag * exp_im.sin()))
        }
        Op::Eq => Ok(Value::Scalar(bool_to_f64(re1 == re2 && im1 == im2))),
        Op::NotEq => Ok(Value::Scalar(bool_to_f64(re1 != re2 || im1 != im2))),
        Op::Lt | Op::Gt | Op::LtEq | Op::GtEq => {
            Err("Ordering is not defined for complex numbers".to_string())
        }
        Op::And | Op::ElemAnd => Ok(Value::Scalar(bool_to_f64(
            (re1 != 0.0 || im1 != 0.0) && (re2 != 0.0 || im2 != 0.0),
        ))),
        Op::Or | Op::ElemOr => Ok(Value::Scalar(bool_to_f64(
            re1 != 0.0 || im1 != 0.0 || re2 != 0.0 || im2 != 0.0,
        ))),
        Op::LDiv => Err("Left division (\\) is not supported for complex numbers".to_string()),
    }
}

/// Constructs a `Value::Complex` or collapses to `Value::Scalar` when `im` is exactly zero.
#[inline]
fn make_complex(re: f64, im: f64) -> Value {
    if im == 0.0 {
        Value::Scalar(re)
    } else {
        Value::Complex(re, im)
    }
}

/// Converts a char array string to its numeric representation.
/// Single char → Scalar(code), multi-char → 1×N Matrix, empty → 1×0 Matrix.
fn str_to_numeric(s: &str) -> Value {
    let codes: Vec<f64> = s.chars().map(|c| c as u32 as f64).collect();
    match codes.len() {
        0 => Value::Matrix(Array2::zeros((1, 0))),
        1 => Value::Scalar(codes[0]),
        n => Value::Matrix(Array2::from_shape_vec((1, n), codes).unwrap()),
    }
}

/// Extracts a string slice from a Str or StringObj value.
fn string_arg<'a>(v: &'a Value, fname: &str, pos: usize) -> Result<&'a str, String> {
    match v {
        Value::Str(s) | Value::StringObj(s) => Ok(s.as_str()),
        _ => Err(format!(
            "Function '{fname}' argument {pos} must be a string"
        )),
    }
}

fn check_same_shape(lm: &Array2<f64>, rm: &Array2<f64>) -> Result<(), String> {
    if lm.shape() != rm.shape() {
        return Err(format!(
            "Matrix size mismatch: {}x{} vs {}x{}",
            lm.nrows(),
            lm.ncols(),
            rm.nrows(),
            rm.ncols()
        ));
    }
    Ok(())
}

fn scalar_arg(v: &Value, fname: &str, pos: usize) -> Result<f64, String> {
    match v {
        Value::Void => Err(format!(
            "Function '{fname}' argument {pos} must be a scalar, got void"
        )),
        Value::Scalar(n) => Ok(*n),
        Value::Complex(re, im) if *im == 0.0 => Ok(*re),
        Value::Complex(_, _) => Err(format!(
            "Function '{fname}' argument {pos} must be real, got a complex number"
        )),
        Value::Matrix(_) => Err(format!(
            "Function '{fname}' argument {pos} must be a scalar, got a matrix"
        )),
        Value::Str(s) if s.chars().count() == 1 => Ok(s.chars().next().unwrap() as u32 as f64),
        Value::Str(_) | Value::StringObj(_) => Err(format!(
            "Function '{fname}' argument {pos} must be a scalar, got a string"
        )),
        Value::Lambda(_)
        | Value::Function { .. }
        | Value::Tuple(_)
        | Value::Cell(_)
        | Value::Struct(_)
        | Value::StructArray(_)
        | Value::DateTime(_)
        | Value::Duration(_)
        | Value::DateTimeArray(_)
        | Value::DurationArray(_) => Err(format!(
            "Function '{fname}' argument {pos} must be a scalar, got a non-numeric value"
        )),
    }
}

/// Applies a scalar function element-wise to a scalar or matrix.
/// Parses the first argument of `randi` into an inclusive `[lo, hi]` integer range.
///
/// Accepts either a scalar `max` (→ `[1, max]`) or a 1×2 / 2×1 vector `[min, max]`.
fn randi_range(v: &Value) -> Result<(i64, i64), String> {
    match v {
        Value::Scalar(n) => {
            let hi = *n as i64;
            if hi < 1 {
                return Err("randi: max must be a positive integer".to_string());
            }
            Ok((1, hi))
        }
        Value::Matrix(m) if m.len() == 2 => {
            let vals: Vec<f64> = m.iter().copied().collect();
            let lo = vals[0] as i64;
            let hi = vals[1] as i64;
            if lo > hi {
                return Err("randi: [min, max] range is empty".to_string());
            }
            Ok((lo, hi))
        }
        _ => Err("randi: first argument must be a scalar max or a [min, max] vector".to_string()),
    }
}

// ── Descriptive statistics helpers ───────────────────────────────────────────

/// Extracts a flat `Vec<f64>` from a `Scalar` or `Matrix` value.
fn numeric_vec(v: &Value, fname: &str) -> Result<Vec<f64>, String> {
    match v {
        Value::Scalar(n) => Ok(vec![*n]),
        Value::Matrix(m) => Ok(m.iter().copied().collect()),
        _ => Err(format!("{fname}: argument must be numeric")),
    }
}

/// Computes the variance of a slice.  Returns `NaN` for empty, `0.0` for singletons.
/// `population = true` divides by N; `false` divides by N-1.
fn stat_var_vec(vals: &[f64], population: bool) -> f64 {
    let n = vals.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return 0.0;
    }
    let mean = vals.iter().sum::<f64>() / n as f64;
    let ss: f64 = vals.iter().map(|&x| (x - mean).powi(2)).sum();
    let denom = if population { n as f64 } else { (n - 1) as f64 };
    ss / denom
}

/// Applies a column-wise statistical closure returning one scalar per column.
///
/// - Scalar → passes `[n]` to `f`.
/// - Vector (1×N or N×1) → scalar.
/// - M×N matrix (M>1, N>1) → 1×N row vector.
fn apply_stat<F>(v: &Value, mut f: F, fname: &str) -> Result<Value, String>
where
    F: FnMut(&[f64]) -> f64,
{
    match v {
        Value::Scalar(n) => Ok(Value::Scalar(f(&[*n]))),
        Value::Matrix(m) => {
            if m.nrows() == 1 || m.ncols() == 1 {
                let vals: Vec<f64> = m.iter().copied().collect();
                Ok(Value::Scalar(f(&vals)))
            } else {
                let ncols = m.ncols();
                let result: Vec<f64> = (0..ncols)
                    .map(|c| {
                        let col: Vec<f64> = m.column(c).iter().copied().collect();
                        f(&col)
                    })
                    .collect();
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, ncols), result).unwrap(),
                ))
            }
        }
        _ => Err(format!("{fname}: argument must be numeric")),
    }
}

/// Computes the p-th percentile (0–100) of a pre-sorted slice via linear interpolation.
fn percentile_sorted(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return sorted[0];
    }
    let p = p.clamp(0.0, 100.0);
    let idx = p / 100.0 * (n - 1) as f64;
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    let frac = idx - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

fn apply_elem<F: Fn(f64) -> f64>(v: &Value, f: F) -> Result<Value, String> {
    match v {
        Value::Void => Err("Element-wise function not applicable to void".to_string()),
        Value::Scalar(n) => Ok(Value::Scalar(f(*n))),
        Value::Matrix(m) => Ok(Value::Matrix(m.mapv(f))),
        Value::Complex(re, im) if *im == 0.0 => Ok(Value::Scalar(f(*re))),
        Value::Complex(_, _) => {
            Err("Element-wise real function not applicable to complex values".to_string())
        }
        Value::Str(_) | Value::StringObj(_) => {
            Err("Element-wise function not applicable to strings".to_string())
        }
        Value::Lambda(_)
        | Value::Function { .. }
        | Value::Tuple(_)
        | Value::Cell(_)
        | Value::Struct(_)
        | Value::StructArray(_)
        | Value::DateTime(_)
        | Value::Duration(_)
        | Value::DateTimeArray(_)
        | Value::DurationArray(_) => {
            Err("Element-wise function not applicable to this type".to_string())
        }
    }
}

/// Reduces a scalar or matrix to a scalar (for vectors) or 1×N row vector (for M×N matrices).
///
/// - Scalar → apply `f` to `[n]`.
/// - Vector (1×N or N×1) → apply `f` to all elements, return scalar.
/// - M×N matrix (M>1, N>1) → apply `f` column-wise, return 1×N row vector.
fn apply_reduction<F>(v: &Value, f: F) -> Result<Value, String>
where
    F: Fn(&[f64]) -> f64,
{
    match v {
        Value::Void => Err("Reduction not applicable to void".to_string()),
        Value::Scalar(n) => Ok(Value::Scalar(f(&[*n]))),
        Value::Complex(_, _) => Err("Reduction not applicable to complex values".to_string()),
        Value::Str(_) | Value::StringObj(_) => {
            Err("Reduction not applicable to strings".to_string())
        }
        Value::Lambda(_)
        | Value::Function { .. }
        | Value::Tuple(_)
        | Value::Cell(_)
        | Value::Struct(_)
        | Value::StructArray(_)
        | Value::DateTime(_)
        | Value::Duration(_)
        | Value::DateTimeArray(_)
        | Value::DurationArray(_) => Err("Reduction not applicable to this type".to_string()),
        Value::Matrix(m) => {
            if m.nrows() == 1 || m.ncols() == 1 {
                let vals: Vec<f64> = m.iter().copied().collect();
                Ok(Value::Scalar(f(&vals)))
            } else {
                let ncols = m.ncols();
                let result: Vec<f64> = (0..ncols)
                    .map(|c| {
                        let col: Vec<f64> = m.column(c).iter().copied().collect();
                        f(&col)
                    })
                    .collect();
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, ncols), result).unwrap(),
                ))
            }
        }
    }
}

/// Computes a cumulative scan (cumsum / cumprod) along a vector or column-wise on a matrix.
///
/// `combine(accumulator, element) -> new_accumulator` — e.g. `|a, x| a + x` for cumsum.
fn apply_cumulative<F>(v: &Value, combine: F) -> Result<Value, String>
where
    F: Fn(f64, f64) -> f64,
{
    match v {
        Value::Void => Err("Cumulative reduction not applicable to void".to_string()),
        Value::Scalar(n) => Ok(Value::Scalar(*n)),
        Value::Complex(_, _) => {
            Err("Cumulative reduction not applicable to complex values".to_string())
        }
        Value::Str(_) | Value::StringObj(_) => {
            Err("Cumulative reduction not applicable to strings".to_string())
        }
        Value::Lambda(_)
        | Value::Function { .. }
        | Value::Tuple(_)
        | Value::Cell(_)
        | Value::Struct(_)
        | Value::StructArray(_)
        | Value::DateTime(_)
        | Value::Duration(_)
        | Value::DateTimeArray(_)
        | Value::DurationArray(_) => {
            Err("Cumulative reduction not applicable to this type".to_string())
        }
        Value::Matrix(m) => {
            let initial = combine(0.0, 0.0); // detect identity: 0+0=0 or 0*0=0
            // Use 0.0 as additive identity, 1.0 as multiplicative identity.
            // We detect the identity from f(1.0, 1.0) vs f(0.0, 0.0).
            let identity = if (combine(1.0, 1.0) - 1.0).abs() < 1e-15 && initial == 0.0 {
                1.0 // product
            } else {
                0.0 // sum
            };
            let (nrows, ncols) = (m.nrows(), m.ncols());
            let mut result = m.clone();
            if nrows == 1 || ncols == 1 {
                // Vector: scan along all elements in order
                let mut acc = identity;
                for v in result.iter_mut() {
                    acc = combine(acc, *v);
                    *v = acc;
                }
            } else {
                // Matrix: scan each column independently
                for c in 0..ncols {
                    let mut acc = identity;
                    for r in 0..nrows {
                        acc = combine(acc, result[[r, c]]);
                        result[[r, c]] = acc;
                    }
                }
            }
            Ok(Value::Matrix(result))
        }
    }
}

/// Returns column-major 1-based indices of non-zero elements, up to `max_k`.
fn find_nonzero(v: &Value, max_k: usize) -> Result<Value, String> {
    match v {
        Value::Void => Err("find: not applicable to void".to_string()),
        Value::Str(_) | Value::StringObj(_) => Err("find: not applicable to strings".to_string()),
        Value::Lambda(_)
        | Value::Function { .. }
        | Value::Tuple(_)
        | Value::Cell(_)
        | Value::Struct(_)
        | Value::StructArray(_)
        | Value::DateTime(_)
        | Value::Duration(_)
        | Value::DateTimeArray(_)
        | Value::DurationArray(_) => Err("find: not applicable to this type".to_string()),
        Value::Complex(re, im) => {
            if (*re != 0.0 || *im != 0.0) && max_k >= 1 {
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, 1), vec![1.0]).unwrap(),
                ))
            } else {
                Ok(Value::Matrix(Array2::zeros((1, 0))))
            }
        }
        Value::Scalar(n) => {
            if *n != 0.0 && max_k >= 1 {
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, 1), vec![1.0]).unwrap(),
                ))
            } else {
                Ok(Value::Matrix(Array2::zeros((1, 0))))
            }
        }
        Value::Matrix(m) => {
            let nrows = m.nrows();
            let total = m.len();
            let mut idxs: Vec<f64> = Vec::new();
            for i in 0..total {
                if idxs.len() >= max_k {
                    break;
                }
                let row = i % nrows;
                let col = i / nrows;
                if m[[row, col]] != 0.0 {
                    idxs.push((i + 1) as f64);
                }
            }
            let n = idxs.len();
            if n == 0 {
                Ok(Value::Matrix(Array2::zeros((1, 0))))
            } else {
                Ok(Value::Matrix(Array2::from_shape_vec((1, n), idxs).unwrap()))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// C-style printf format engine
// ---------------------------------------------------------------------------

/// Formats `args` using a C-style `fmt` string.
///
/// Supported specifiers: `%d` `%i` `%f` `%e` `%g` `%s` `%%`.
/// Flags: `-` (left-align), `+` (force sign), `0` (zero-pad), ` ` (space sign).
/// Width and `.precision` follow standard C `printf` conventions.
/// Escape sequences `\n` `\t` `\\` are also processed.
///
/// Octave behaviour: if `args` is longer than the number of specifiers the
/// format string is repeated until all args are consumed.
pub fn format_printf(fmt: &str, args: &[Value]) -> Result<String, String> {
    let mut result = String::new();
    let mut arg_idx = 0;

    loop {
        let consumed_before = arg_idx;
        let mut chars = fmt.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('\'') => result.push('\''),
                    Some('"') => result.push('"'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
                continue;
            }

            if c != '%' {
                result.push(c);
                continue;
            }

            // `%%` → literal `%`
            if chars.peek() == Some(&'%') {
                chars.next();
                result.push('%');
                continue;
            }

            // Parse flags
            let mut flag_minus = false;
            let mut flag_plus = false;
            let mut flag_zero = false;
            let mut flag_space = false;
            loop {
                match chars.peek() {
                    Some('-') => {
                        flag_minus = true;
                        chars.next();
                    }
                    Some('+') => {
                        flag_plus = true;
                        chars.next();
                    }
                    Some('0') => {
                        flag_zero = true;
                        chars.next();
                    }
                    Some(' ') => {
                        flag_space = true;
                        chars.next();
                    }
                    _ => break,
                }
            }

            // Parse width
            let mut width_str = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    width_str.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            let width: usize = width_str.parse().unwrap_or(0);

            // Parse precision
            let mut precision: Option<usize> = None;
            if chars.peek() == Some(&'.') {
                chars.next();
                let mut p = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        p.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                precision = Some(p.parse().unwrap_or(0));
            }

            // Specifier character
            let spec = match chars.next() {
                Some(s) => s,
                None => {
                    return Err("fprintf: incomplete format specifier at end of string".to_string());
                }
            };

            // No more args — silently skip remaining specifiers
            if arg_idx >= args.len() {
                continue;
            }

            let arg = &args[arg_idx];
            arg_idx += 1;

            let formatted = match spec {
                'd' | 'i' => {
                    let n = printf_scalar(arg, spec)?;
                    let i = n.trunc() as i64;
                    let s = printf_sign_str(i >= 0, flag_plus, flag_space, format!("{}", i.abs()));
                    printf_pad(s, width, flag_minus, flag_zero)
                }
                'f' => {
                    let n = printf_scalar(arg, spec)?;
                    let prec = precision.unwrap_or(6);
                    let s = printf_sign_str(
                        n >= 0.0,
                        flag_plus,
                        flag_space,
                        format!("{:.prec$}", n.abs(), prec = prec),
                    );
                    printf_pad(s, width, flag_minus, flag_zero)
                }
                'e' | 'E' => {
                    let n = printf_scalar(arg, spec)?;
                    let prec = precision.unwrap_or(6);
                    let s = printf_format_sci(n, prec, flag_plus, flag_space, spec == 'E');
                    printf_pad(s, width, flag_minus, flag_zero)
                }
                'g' | 'G' => {
                    let n = printf_scalar(arg, spec)?;
                    let prec = precision.unwrap_or(6).max(1);
                    let s = printf_format_g(n, prec, flag_plus, flag_space, spec == 'G');
                    printf_pad(s, width, flag_minus, flag_zero)
                }
                's' => {
                    let s = printf_string(arg)?;
                    let s = if let Some(max_len) = precision {
                        s.chars().take(max_len).collect::<String>()
                    } else {
                        s
                    };
                    printf_pad(s, width, flag_minus, false)
                }
                other => return Err(format!("fprintf: unknown format specifier '%{other}'")),
            };

            result.push_str(&formatted);
        }

        // Stop if all args consumed or no specifiers were found (infinite loop guard)
        if arg_idx >= args.len() || arg_idx == consumed_before {
            break;
        }
    }

    Ok(result)
}

/// Extracts a scalar f64 from a Value for use in numeric printf specifiers.
fn printf_scalar(v: &Value, spec: char) -> Result<f64, String> {
    match v {
        Value::Scalar(n) => Ok(*n),
        Value::Complex(re, im) if *im == 0.0 => Ok(*re),
        Value::Str(s) if s.chars().count() == 1 => Ok(s.chars().next().unwrap() as u32 as f64),
        _ => Err(format!(
            "fprintf: expected numeric argument for '%{spec}', got {:?}",
            std::mem::discriminant(v)
        )),
    }
}

/// Extracts a string from a Value for use in `%s`.
fn printf_string(v: &Value) -> Result<String, String> {
    match v {
        Value::Str(s) | Value::StringObj(s) => Ok(s.clone()),
        Value::Scalar(n) => Ok(format_number(*n)),
        Value::Complex(re, im) => Ok(format_complex(*re, *im, &FormatMode::Custom(6))),
        Value::Void => Err("fprintf: cannot format void as string".to_string()),
        Value::Matrix(_) => Err("fprintf: cannot format matrix as string".to_string()),
        Value::DateTime(ts) => Ok(crate::datetime::format_datetime(*ts)),
        Value::Duration(s) => Ok(crate::datetime::format_duration(*s)),
        Value::Lambda(_)
        | Value::Function { .. }
        | Value::Tuple(_)
        | Value::Cell(_)
        | Value::Struct(_)
        | Value::StructArray(_)
        | Value::DateTimeArray(_)
        | Value::DurationArray(_) => Err("fprintf: cannot format this type as string".to_string()),
    }
}

/// Builds a sign-prefixed string: `+n`, ` n`, `-n`, or bare `n`.
fn printf_sign_str(positive: bool, flag_plus: bool, flag_space: bool, digits: String) -> String {
    if positive {
        if flag_plus {
            format!("+{digits}")
        } else if flag_space {
            format!(" {digits}")
        } else {
            digits
        }
    } else {
        format!("-{digits}")
    }
}

/// Right- or left-pads `s` to at least `width` chars, optionally zero-pads.
fn printf_pad(s: String, width: usize, left_align: bool, zero_pad: bool) -> String {
    if s.len() >= width {
        return s;
    }
    let pad_len = width - s.len();
    if left_align {
        format!("{s}{}", " ".repeat(pad_len))
    } else if zero_pad {
        // Insert zeros after optional sign
        let (prefix, rest) = if s.starts_with(['+', '-', ' ']) {
            s.split_at(1)
        } else {
            ("", s.as_str())
        };
        format!("{prefix}{}{rest}", "0".repeat(pad_len))
    } else {
        format!("{}{s}", " ".repeat(pad_len))
    }
}

/// Formats `n` in scientific notation matching C `%e` / `%E`.
/// Always produces at least 2 exponent digits with an explicit sign: `1.23e+04`.
fn printf_format_sci(
    n: f64,
    prec: usize,
    flag_plus: bool,
    flag_space: bool,
    upper: bool,
) -> String {
    if n == 0.0 {
        let zeros = "0".repeat(prec);
        let sep = if prec > 0 {
            format!(".{zeros}")
        } else {
            String::new()
        };
        let e_char = if upper { 'E' } else { 'e' };
        let sign = if flag_plus {
            "+"
        } else if flag_space {
            " "
        } else {
            ""
        };
        return format!("{sign}0{sep}{e_char}+00");
    }

    let neg = n < 0.0;
    let abs_n = n.abs();
    let exp = abs_n.log10().floor() as i32;
    let mantissa = abs_n / 10f64.powi(exp);
    let man_str = format!("{:.prec$}", mantissa, prec = prec);

    let e_char = if upper { 'E' } else { 'e' };
    let exp_sign = if exp >= 0 { '+' } else { '-' };
    let exp_abs = exp.unsigned_abs();
    let exp_str = if exp_abs < 10 {
        format!("{e_char}{exp_sign}0{exp_abs}")
    } else {
        format!("{e_char}{exp_sign}{exp_abs}")
    };

    let sign_str = if neg {
        "-"
    } else if flag_plus {
        "+"
    } else if flag_space {
        " "
    } else {
        ""
    };
    format!("{sign_str}{man_str}{exp_str}")
}

/// Formats `n` using `%g` / `%G` rules:
/// uses `%e` if exponent < -4 or >= prec, otherwise `%f`; trims trailing zeros.
fn printf_format_g(n: f64, prec: usize, flag_plus: bool, flag_space: bool, upper: bool) -> String {
    if n == 0.0 {
        let sign = if flag_plus {
            "+"
        } else if flag_space {
            " "
        } else {
            ""
        };
        return format!("{sign}0");
    }
    let abs_n = n.abs();
    let exp = abs_n.log10().floor() as i32;
    if exp < -4 || exp >= prec as i32 {
        let s = printf_format_sci(n, prec.saturating_sub(1), flag_plus, flag_space, upper);
        trim_g_sci(s, upper)
    } else {
        let decimal_places = (prec as i32 - 1 - exp).max(0) as usize;
        let neg = n < 0.0;
        let s = format!("{:.prec$}", abs_n, prec = decimal_places);
        let s = if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        };
        let sign = if neg {
            "-"
        } else if flag_plus {
            "+"
        } else if flag_space {
            " "
        } else {
            ""
        };
        format!("{sign}{s}")
    }
}

/// Trims trailing zeros from the mantissa of a scientific-notation string `1.230e+04` → `1.23e+04`.
fn trim_g_sci(s: String, upper: bool) -> String {
    let e_char = if upper { 'E' } else { 'e' };
    if let Some(e_pos) = s.find(e_char) {
        let mantissa = &s[..e_pos];
        let exp_part = &s[e_pos..];
        let trimmed = if mantissa.contains('.') {
            mantissa.trim_end_matches('0').trim_end_matches('.')
        } else {
            mantissa
        };
        format!("{trimmed}{exp_part}")
    } else {
        s
    }
}

/// Calls a `Lambda` or `Function` value with the given arguments.
///
/// Used by `cellfun` and `arrayfun` to apply a function to each element
/// without going through the name-lookup path.
fn call_function_value(
    f: &Value,
    args: &[Value],
    io: Option<&mut IoContext>,
) -> Result<Value, String> {
    match f {
        Value::Lambda(lf) => {
            let lf = lf.clone();
            lf.0(args, io)
        }
        Value::Function { .. } => {
            // Named function called via cellfun/arrayfun — name is unknown at this point.
            // Use a minimal env that doesn't export any user variables to avoid
            // polluting the caller's scope. Functions see their own scope via exec.
            let empty_env = Env::new();
            match io {
                Some(io_ref) => FN_CALL_HOOK.with(|c| match c.get() {
                    Some(hook) => hook("<anonymous>", f, args, &empty_env, io_ref),
                    None => Err("User function execution not initialized".to_string()),
                }),
                None => {
                    let mut tmp_io = IoContext::new();
                    FN_CALL_HOOK.with(|c| match c.get() {
                        Some(hook) => hook("<anonymous>", f, args, &empty_env, &mut tmp_io),
                        None => Err("User function execution not initialized".to_string()),
                    })
                }
            }
        }
        _ => Err("cellfun/arrayfun: first argument must be a function or lambda (@fn)".to_string()),
    }
}

/// Names of all built-in functions recognized by [`call_builtin`].
///
/// Used for REPL tab completion and "did you mean?" suggestions.
pub fn builtin_names() -> &'static [&'static str] {
    &[
        "abs",
        "acos",
        "all",
        "angle",
        "any",
        "arrayfun",
        "asin",
        "assert",
        "atan",
        "atan2",
        "bitand",
        "bitnot",
        "bitor",
        "bitshift",
        "bitxor",
        "ceil",
        "cell",
        "cellfun",
        "chol",
        "complex",
        "cond",
        "cross",
        "conj",
        "contains",
        "conv",
        "cos",
        "cov",
        "cumprod",
        "cumsum",
        "datenum",
        "datestr",
        "datevec",
        "datetime",
        "day",
        "days",
        "deconv",
        "det",
        "diag",
        "diff",
        "dot",
        "disp",
        "dlmread",
        "dlmwrite",
        "eig",
        "endsWith",
        "erf",
        "erfc",
        "exist",
        "exp",
        "eye",
        "fclose",
        "fgetl",
        "fgets",
        "fieldnames",
        "find",
        "fliplr",
        "flipud",
        "floor",
        "fopen",
        "fprintf",
        "genpath",
        "histc",
        "hour",
        "hours",
        "hypot",
        "imag",
        "ind2sub",
        "int2str",
        "interp1",
        "intersect",
        "inv",
        "iqr",
        "iscell",
        "ismember",
        "ischar",
        "isdatetime",
        "isduration",
        "isempty",
        "isfield",
        "isfile",
        "isfinite",
        "isfolder",
        "isinf",
        "isnan",
        "isnat",
        "isreal",
        "isstring",
        "isstruct",
        "jsonencode",
        "jsondecode",
        "kron",
        "kurtosis",
        "lasterr",
        "length",
        "linspace",
        "load",
        "log",
        "log10",
        "log2",
        "lower",
        "lu",
        "mat2str",
        "max",
        "mean",
        "median",
        "milliseconds",
        "min",
        "minute",
        "minutes",
        "mod",
        "mode",
        "month",
        "nan",
        "norm",
        "normcdf",
        "normpdf",
        "not",
        "null",
        "num2str",
        "numel",
        "ones",
        "orth",
        "pinv",
        "poly",
        "polyfit",
        "polyval",
        "posixtime",
        "prctile",
        "prod",
        "qr",
        "rand",
        "randi",
        "randn",
        "rank",
        "readmatrix",
        "readtable",
        "real",
        "regexp",
        "regexpi",
        "regexprep",
        "rem",
        "repelem",
        "repmat",
        "reshape",
        "rmfield",
        "rng",
        "roots",
        "round",
        "setdiff",
        "second",
        "seconds",
        "sign",
        "sin",
        "size",
        "skewness",
        "sort",
        "sprintf",
        "sqrt",
        "startsWith",
        "std",
        "str2double",
        "str2num",
        "strcmp",
        "strcmpi",
        "strjoin",
        "strrep",
        "strsplit",
        "strtrim",
        "sub2ind",
        "sum",
        "svd",
        "tan",
        "trace",
        "tril",
        "triu",
        "union",
        "unique",
        "upper",
        "var",
        "writetable",
        "xor",
        "year",
        "years",
        "zeros",
        "zscore",
    ]
}

/// Computes the Levenshtein edit distance between two strings.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    let mut row: Vec<usize> = (0..=n).collect();
    for i in 1..=m {
        let mut prev = row[0];
        row[0] = i;
        for j in 1..=n {
            let next = if a[i - 1] == b[j - 1] {
                prev
            } else {
                1 + prev.min(row[j]).min(row[j - 1])
            };
            prev = row[j];
            row[j] = next;
        }
    }
    row[n]
}

/// Finds the closest name in `env` keys and built-in names within Levenshtein distance 2.
fn suggest_similar(name: &str, env: &Env) -> Option<String> {
    const MAX_DIST: usize = 2;
    let mut best: Option<(String, usize)> = None;
    let mut update = |candidate: &str| {
        let d = levenshtein(name, candidate);
        if d <= MAX_DIST && best.as_ref().is_none_or(|(_, bd)| d < *bd) {
            best = Some((candidate.to_string(), d));
        }
    };
    for key in env.keys() {
        update(key);
    }
    for &bname in builtin_names() {
        update(bname);
    }
    best.map(|(s, _)| s)
}

/// Checks equality of two values for `assert(a, b[, tol])`.
fn assert_values_equal(a: &Value, b: &Value, tol: Option<f64>) -> Result<Value, String> {
    match (a, b) {
        (Value::Scalar(x), Value::Scalar(y)) => {
            let ok = match tol {
                None => x == y,
                Some(t) => (x - y).abs() <= t,
            };
            if ok {
                Ok(Value::Void)
            } else if let Some(t) = tol {
                Err(format!(
                    "assert: |{x} - {y}| = {} exceeds tolerance {t}",
                    (x - y).abs()
                ))
            } else {
                Err(format!("assert: {x} ~= {y}"))
            }
        }
        (Value::Matrix(ma), Value::Matrix(mb)) => {
            if ma.shape() != mb.shape() {
                return Err(format!(
                    "assert: size mismatch [{}×{}] vs [{}×{}]",
                    ma.nrows(),
                    ma.ncols(),
                    mb.nrows(),
                    mb.ncols()
                ));
            }
            for (x, y) in ma.iter().zip(mb.iter()) {
                let ok = match tol {
                    None => x == y,
                    Some(t) => (x - y).abs() <= t,
                };
                if !ok {
                    if let Some(t) = tol {
                        return Err(format!(
                            "assert: difference {} exceeds tolerance {t}",
                            (x - y).abs()
                        ));
                    } else {
                        return Err(format!("assert: {x} ~= {y}"));
                    }
                }
            }
            Ok(Value::Void)
        }
        _ => {
            if tol.is_some() {
                return Err("assert: tolerance requires numeric arguments".to_string());
            }
            if a == b {
                Ok(Value::Void)
            } else {
                Err("assert: values not equal".to_string())
            }
        }
    }
}

fn call_builtin(
    name: &str,
    args: &[Value],
    env: &Env,
    mut io: Option<&mut IoContext>,
) -> Result<Value, String> {
    match (name, args.len()) {
        // --- 1-argument scalar functions ---
        ("sqrt", 1) => apply_elem(&args[0], |x| x.sqrt()),
        ("floor", 1) => apply_elem(&args[0], |x| x.floor()),
        ("ceil", 1) => apply_elem(&args[0], |x| x.ceil()),
        ("round", 1) => apply_elem(&args[0], |x| x.round()),
        ("sign", 1) => apply_elem(&args[0], |x| x.signum()),
        ("log", 1) => apply_elem(&args[0], |x| x.ln()),
        ("log2", 1) => apply_elem(&args[0], |x| x.log2()),
        ("log10", 1) => apply_elem(&args[0], |x| x.log10()),
        ("exp", 1) => apply_elem(&args[0], |x| x.exp()),
        ("sin", 1) => apply_elem(&args[0], |x| x.sin()),
        ("cos", 1) => apply_elem(&args[0], |x| x.cos()),
        ("tan", 1) => apply_elem(&args[0], |x| x.tan()),
        ("asin", 1) => apply_elem(&args[0], |x| x.asin()),
        ("acos", 1) => apply_elem(&args[0], |x| x.acos()),
        ("atan", 1) => apply_elem(&args[0], |x| x.atan()),
        // --- Special functions (erf, normal distribution) ---
        ("erf", 1) => apply_elem(&args[0], libm::erf),
        ("erfc", 1) => apply_elem(&args[0], libm::erfc),
        ("normcdf", 1) => apply_elem(&args[0], |x| {
            0.5 * (1.0 + libm::erf(x / std::f64::consts::SQRT_2))
        }),
        ("normcdf", 3) => {
            let mu = scalar_arg(&args[1], name, 2)?;
            let s = scalar_arg(&args[2], name, 3)?;
            if s <= 0.0 {
                return Err("normcdf: sigma must be positive".to_string());
            }
            apply_elem(&args[0], move |x| {
                0.5 * (1.0 + libm::erf((x - mu) / (s * std::f64::consts::SQRT_2)))
            })
        }
        ("normpdf", 1) => apply_elem(&args[0], |x| {
            (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
        }),
        ("normpdf", 3) => {
            let mu = scalar_arg(&args[1], name, 2)?;
            let s = scalar_arg(&args[2], name, 3)?;
            if s <= 0.0 {
                return Err("normpdf: sigma must be positive".to_string());
            }
            apply_elem(&args[0], move |x| {
                let z = (x - mu) / s;
                (-0.5 * z * z).exp() / (s * (2.0 * std::f64::consts::PI).sqrt())
            })
        }
        // --- 2-argument scalar functions ---
        ("atan2", 2) => Ok(Value::Scalar(
            scalar_arg(&args[0], name, 1)?.atan2(scalar_arg(&args[1], name, 2)?),
        )),
        ("mod", 2) => {
            let a = scalar_arg(&args[0], name, 1)?;
            let b = scalar_arg(&args[1], name, 2)?;
            Ok(Value::Scalar(a - b * (a / b).floor()))
        }
        ("rem", 2) => {
            let a = scalar_arg(&args[0], name, 1)?;
            let b = scalar_arg(&args[1], name, 2)?;
            Ok(Value::Scalar(a - b * (a / b).trunc()))
        }
        ("max", 2) => Ok(Value::Scalar(
            scalar_arg(&args[0], name, 1)?.max(scalar_arg(&args[1], name, 2)?),
        )),
        ("min", 2) => Ok(Value::Scalar(
            scalar_arg(&args[0], name, 1)?.min(scalar_arg(&args[1], name, 2)?),
        )),
        ("hypot", 2) => Ok(Value::Scalar(
            scalar_arg(&args[0], name, 1)?.hypot(scalar_arg(&args[1], name, 2)?),
        )),
        ("log", 2) => Ok(Value::Scalar(
            scalar_arg(&args[0], name, 1)?.log(scalar_arg(&args[1], name, 2)?),
        )),
        // --- Matrix constructors ---
        ("zeros", 1) => {
            let n = scalar_arg(&args[0], name, 1)? as usize;
            Ok(Value::Matrix(Array2::zeros((n, n))))
        }
        ("zeros", 2) => {
            let r = scalar_arg(&args[0], name, 1)? as usize;
            let c = scalar_arg(&args[1], name, 2)? as usize;
            Ok(Value::Matrix(Array2::zeros((r, c))))
        }
        ("ones", 1) => {
            let n = scalar_arg(&args[0], name, 1)? as usize;
            Ok(Value::Matrix(Array2::ones((n, n))))
        }
        ("ones", 2) => {
            let r = scalar_arg(&args[0], name, 1)? as usize;
            let c = scalar_arg(&args[1], name, 2)? as usize;
            Ok(Value::Matrix(Array2::ones((r, c))))
        }
        ("eye", 1) => {
            let n = scalar_arg(&args[0], name, 1)? as usize;
            let mut m = Array2::<f64>::zeros((n, n));
            for i in 0..n {
                m[[i, i]] = 1.0;
            }
            Ok(Value::Matrix(m))
        }
        // --- Matrix properties ---
        ("size", 1) => match &args[0] {
            Value::Void => Err("size: not applicable to void".to_string()),
            Value::Scalar(_) | Value::Complex(_, _) | Value::Struct(_) => Ok(Value::Matrix(
                Array2::from_shape_vec((1, 2), vec![1.0, 1.0]).unwrap(),
            )),
            Value::Matrix(m) => Ok(Value::Matrix(
                Array2::from_shape_vec((1, 2), vec![m.nrows() as f64, m.ncols() as f64]).unwrap(),
            )),
            Value::Str(s) => Ok(Value::Matrix(
                Array2::from_shape_vec((1, 2), vec![1.0, s.chars().count() as f64]).unwrap(),
            )),
            Value::StringObj(_) => Ok(Value::Matrix(
                Array2::from_shape_vec((1, 2), vec![1.0, 1.0]).unwrap(),
            )),
            Value::Cell(v) => Ok(Value::Matrix(
                Array2::from_shape_vec((1, 2), vec![1.0, v.len() as f64]).unwrap(),
            )),
            Value::StructArray(arr) => Ok(Value::Matrix(
                Array2::from_shape_vec((1, 2), vec![1.0, arr.len() as f64]).unwrap(),
            )),
            Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => Err("size: not applicable to this type".to_string()),
        },
        ("size", 2) => {
            let dim = scalar_arg(&args[1], name, 2)? as usize;
            match &args[0] {
                Value::Void => Err("size: not applicable to void".to_string()),
                Value::Scalar(_) | Value::Complex(_, _) | Value::Struct(_) => {
                    Ok(Value::Scalar(1.0))
                }
                Value::Matrix(m) => match dim {
                    1 => Ok(Value::Scalar(m.nrows() as f64)),
                    2 => Ok(Value::Scalar(m.ncols() as f64)),
                    _ => Err(format!("size: invalid dimension {dim}, must be 1 or 2")),
                },
                Value::Str(s) => match dim {
                    1 => Ok(Value::Scalar(1.0)),
                    2 => Ok(Value::Scalar(s.chars().count() as f64)),
                    _ => Err(format!("size: invalid dimension {dim}")),
                },
                Value::StringObj(_) => Ok(Value::Scalar(1.0)),
                Value::Cell(v) => match dim {
                    1 => Ok(Value::Scalar(1.0)),
                    2 => Ok(Value::Scalar(v.len() as f64)),
                    _ => Err(format!("size: invalid dimension {dim}")),
                },
                Value::StructArray(arr) => match dim {
                    1 => Ok(Value::Scalar(1.0)),
                    2 => Ok(Value::Scalar(arr.len() as f64)),
                    _ => Err(format!("size: invalid dimension {dim}")),
                },
                Value::Lambda(_)
                | Value::Function { .. }
                | Value::Tuple(_)
                | Value::DateTime(_)
                | Value::Duration(_)
                | Value::DateTimeArray(_)
                | Value::DurationArray(_) => Err("size: not applicable to this type".to_string()),
            }
        }
        ("length", 1) => match &args[0] {
            Value::Void => Err("length: not applicable to void".to_string()),
            Value::Scalar(_) | Value::Complex(_, _) | Value::Struct(_) => Ok(Value::Scalar(1.0)),
            Value::Matrix(m) => Ok(Value::Scalar(m.nrows().max(m.ncols()) as f64)),
            Value::Str(s) => Ok(Value::Scalar(s.chars().count() as f64)),
            Value::StringObj(_) => Ok(Value::Scalar(1.0)),
            Value::Cell(v) => Ok(Value::Scalar(v.len() as f64)),
            Value::StructArray(arr) => Ok(Value::Scalar(arr.len() as f64)),
            Value::DateTimeArray(v) | Value::DurationArray(v) => Ok(Value::Scalar(v.len() as f64)),
            Value::DateTime(_) | Value::Duration(_) => Ok(Value::Scalar(1.0)),
            Value::Lambda(_) | Value::Function { .. } | Value::Tuple(_) => {
                Err("length: not applicable to function values".to_string())
            }
        },
        ("numel", 1) => match &args[0] {
            Value::Void => Err("numel: not applicable to void".to_string()),
            Value::Scalar(_) | Value::Complex(_, _) | Value::Struct(_) => Ok(Value::Scalar(1.0)),
            Value::Matrix(m) => Ok(Value::Scalar(m.len() as f64)),
            Value::Str(s) => Ok(Value::Scalar(s.chars().count() as f64)),
            Value::StringObj(_) => Ok(Value::Scalar(1.0)),
            Value::Cell(v) => Ok(Value::Scalar(v.len() as f64)),
            Value::StructArray(arr) => Ok(Value::Scalar(arr.len() as f64)),
            Value::DateTimeArray(v) | Value::DurationArray(v) => Ok(Value::Scalar(v.len() as f64)),
            Value::DateTime(_) | Value::Duration(_) => Ok(Value::Scalar(1.0)),
            Value::Lambda(_) | Value::Function { .. } | Value::Tuple(_) => {
                Err("numel: not applicable to function values".to_string())
            }
        },
        ("trace", 1) => match &args[0] {
            Value::Void => Err("trace: not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            Value::Complex(re, _) => Ok(Value::Scalar(*re)),
            Value::Matrix(m) => {
                let n = m.nrows().min(m.ncols());
                Ok(Value::Scalar((0..n).map(|i| m[[i, i]]).sum()))
            }
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("trace: not applicable to non-numeric values".to_string())
            }
        },
        ("det", 1) => match &args[0] {
            Value::Void => Err("det: not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            Value::Complex(_, _) => Err("det: not applicable to complex scalars".to_string()),
            Value::Matrix(m) => Ok(Value::Scalar(det_matrix(m)?)),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("det: not applicable to non-numeric values".to_string())
            }
        },
        ("inv", 1) => match &args[0] {
            Value::Void => Err("inv: not applicable to void".to_string()),
            Value::Scalar(n) => {
                if *n == 0.0 {
                    Err("inv: singular (zero scalar)".to_string())
                } else {
                    Ok(Value::Scalar(1.0 / n))
                }
            }
            Value::Complex(re, im) => {
                // 1/(a+bi) = (a-bi)/(a²+b²)
                let denom = re * re + im * im;
                if denom == 0.0 {
                    Err("inv: singular (zero complex)".to_string())
                } else {
                    Ok(make_complex(re / denom, -im / denom))
                }
            }
            Value::Matrix(m) => Ok(Value::Matrix(inv_matrix(m)?)),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("inv: not applicable to non-numeric values".to_string())
            }
        },
        // --- Range / linspace ---
        ("linspace", 3) => {
            let a = scalar_arg(&args[0], name, 1)?;
            let b = scalar_arg(&args[1], name, 2)?;
            let n = scalar_arg(&args[2], name, 3)? as usize;
            if n == 0 {
                return Ok(Value::Matrix(Array2::zeros((1, 0))));
            }
            if n == 1 {
                return Ok(Value::Matrix(
                    Array2::from_shape_vec((1, 1), vec![b]).unwrap(),
                ));
            }
            let vals: Vec<f64> = (0..n)
                .map(|i| a + (b - a) * i as f64 / (n - 1) as f64)
                .collect();
            Ok(Value::Matrix(Array2::from_shape_vec((1, n), vals).unwrap()))
        }
        // --- Bitwise functions ---
        // All operands are truncated to i64. Results are non-negative integers
        // returned as f64.  For bitnot the bit-width defines the mask.
        ("bitand", 2) => {
            let a = to_bits(scalar_arg(&args[0], name, 1)?, name, 1)?;
            let b = to_bits(scalar_arg(&args[1], name, 2)?, name, 2)?;
            Ok(Value::Scalar((a & b) as f64))
        }
        ("bitor", 2) => {
            let a = to_bits(scalar_arg(&args[0], name, 1)?, name, 1)?;
            let b = to_bits(scalar_arg(&args[1], name, 2)?, name, 2)?;
            Ok(Value::Scalar((a | b) as f64))
        }
        ("bitxor", 2) => {
            let a = to_bits(scalar_arg(&args[0], name, 1)?, name, 1)?;
            let b = to_bits(scalar_arg(&args[1], name, 2)?, name, 2)?;
            Ok(Value::Scalar((a ^ b) as f64))
        }
        // bitshift(a, n): n > 0 → left shift; n < 0 → logical right shift.
        // Shifts of 64 or more return 0.
        ("bitshift", 2) => {
            let a = to_bits(scalar_arg(&args[0], name, 1)?, name, 1)?;
            let n = scalar_arg(&args[1], name, 2)?;
            if n.fract() != 0.0 {
                return Err("bitshift: shift amount must be an integer".to_string());
            }
            let n = n as i64;
            let result: u64 = if n >= 64 || n <= -64 {
                0
            } else if n >= 0 {
                a.wrapping_shl(n as u32)
            } else {
                a.wrapping_shr((-n) as u32)
            };
            Ok(Value::Scalar(result as f64))
        }
        // bitnot(a)        — NOT within 32-bit window (Octave uint32 default)
        // bitnot(a, bits)  — NOT within explicit bit-width window (1–53)
        ("bitnot", 1) => {
            let a = to_bits(scalar_arg(&args[0], name, 1)?, name, 1)?;
            let mask: u64 = 0xFFFF_FFFF;
            Ok(Value::Scalar(((a ^ mask) & mask) as f64))
        }
        ("bitnot", 2) => {
            let a = to_bits(scalar_arg(&args[0], name, 1)?, name, 1)?;
            let bits = scalar_arg(&args[1], name, 2)?;
            if bits.fract() != 0.0 || !(1.0..=53.0).contains(&bits) {
                return Err(format!(
                    "bitnot: bit-width must be an integer in [1, 53], got {bits}"
                ));
            }
            let mask: u64 = (1u64 << bits as u32) - 1;
            Ok(Value::Scalar(((a ^ mask) & mask) as f64))
        }
        // --- Special constant predicates (element-wise) ---
        ("isnan", 1) => apply_elem(&args[0], |x| if x.is_nan() { 1.0 } else { 0.0 }),
        ("isinf", 1) => apply_elem(&args[0], |x| if x.is_infinite() { 1.0 } else { 0.0 }),
        ("isfinite", 1) => apply_elem(&args[0], |x| if x.is_finite() { 1.0 } else { 0.0 }),
        // --- NaN matrix constructors ---
        ("nan", 1) => {
            let n = scalar_arg(&args[0], name, 1)? as usize;
            Ok(Value::Matrix(Array2::from_elem((n, n), f64::NAN)))
        }
        ("nan", 2) => {
            let r = scalar_arg(&args[0], name, 1)? as usize;
            let c = scalar_arg(&args[1], name, 2)? as usize;
            Ok(Value::Matrix(Array2::from_elem((r, c), f64::NAN)))
        }
        // --- Random number generation ---
        ("rand", 0) => Ok(Value::Scalar(rand_uniform())),
        ("rand", 1) => {
            let n = scalar_arg(&args[0], name, 1)? as usize;
            let data: Vec<f64> = (0..n * n).map(|_| rand_uniform()).collect();
            Ok(Value::Matrix(Array2::from_shape_vec((n, n), data).unwrap()))
        }
        ("rand", 2) => {
            let r = scalar_arg(&args[0], name, 1)? as usize;
            let c = scalar_arg(&args[1], name, 2)? as usize;
            let data: Vec<f64> = (0..r * c).map(|_| rand_uniform()).collect();
            Ok(Value::Matrix(Array2::from_shape_vec((r, c), data).unwrap()))
        }
        ("randn", 0) => Ok(Value::Scalar(rand_normal())),
        ("randn", 1) => {
            let n = scalar_arg(&args[0], name, 1)? as usize;
            let data: Vec<f64> = (0..n * n).map(|_| rand_normal()).collect();
            Ok(Value::Matrix(Array2::from_shape_vec((n, n), data).unwrap()))
        }
        ("randn", 2) => {
            let r = scalar_arg(&args[0], name, 1)? as usize;
            let c = scalar_arg(&args[1], name, 2)? as usize;
            let data: Vec<f64> = (0..r * c).map(|_| rand_normal()).collect();
            Ok(Value::Matrix(Array2::from_shape_vec((r, c), data).unwrap()))
        }
        ("randi", 1) => {
            let (lo, hi) = randi_range(&args[0])?;
            let v = RNG.with(|r| r.borrow_mut().gen_range(lo..=hi)) as f64;
            Ok(Value::Scalar(v))
        }
        ("randi", 2) => {
            let (lo, hi) = randi_range(&args[0])?;
            let n = scalar_arg(&args[1], name, 2)? as usize;
            let data: Vec<f64> = (0..n * n)
                .map(|_| RNG.with(|r| r.borrow_mut().gen_range(lo..=hi)) as f64)
                .collect();
            Ok(Value::Matrix(Array2::from_shape_vec((n, n), data).unwrap()))
        }
        ("randi", 3) => {
            let (lo, hi) = randi_range(&args[0])?;
            let r = scalar_arg(&args[1], name, 2)? as usize;
            let c = scalar_arg(&args[2], name, 3)? as usize;
            let data: Vec<f64> = (0..r * c)
                .map(|_| RNG.with(|rng| rng.borrow_mut().gen_range(lo..=hi)) as f64)
                .collect();
            Ok(Value::Matrix(Array2::from_shape_vec((r, c), data).unwrap()))
        }
        ("rng", 1) => match &args[0] {
            Value::Scalar(n) => {
                rng_seed(*n as u64);
                Ok(Value::Void)
            }
            Value::Str(s) | Value::StringObj(s) if s == "shuffle" => {
                rng_shuffle();
                Ok(Value::Void)
            }
            _ => Err("rng: argument must be a numeric seed or 'shuffle'".to_string()),
        },
        // --- Vector reductions ---
        // For vectors (1×N or N×1): reduce all elements to scalar.
        // For M×N matrices (M>1, N>1): reduce column-wise, return 1×N row vector.
        ("sum", 1) => apply_reduction(&args[0], |v| v.iter().copied().sum()),
        ("prod", 1) => apply_reduction(&args[0], |v| v.iter().copied().product()),
        ("any", 1) => apply_reduction(&args[0], |v| {
            if v.iter().any(|&x| x != 0.0) {
                1.0
            } else {
                0.0
            }
        }),
        ("all", 1) => apply_reduction(&args[0], |v| {
            if v.iter().all(|&x| x != 0.0) {
                1.0
            } else {
                0.0
            }
        }),
        ("mean", 1) => apply_reduction(&args[0], |v| {
            if v.is_empty() {
                f64::NAN
            } else {
                v.iter().copied().sum::<f64>() / v.len() as f64
            }
        }),
        // 1-arg min/max: reduce to scalar for vectors, column-wise for matrices.
        // 2-arg forms (element-wise scalar min/max) are already handled above.
        ("min", 1) => apply_reduction(&args[0], |v| {
            v.iter().copied().fold(f64::INFINITY, f64::min)
        }),
        ("max", 1) => apply_reduction(&args[0], |v| {
            v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        }),
        // --- Norms ---
        ("norm", 1) => match &args[0] {
            Value::Void => Err("norm: not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(n.abs())),
            Value::Complex(re, im) => Ok(Value::Scalar((re * re + im * im).sqrt())),
            Value::Matrix(m) => {
                if m.nrows() <= 1 || m.ncols() <= 1 {
                    // Vector: L2 norm.
                    Ok(Value::Scalar(m.iter().map(|x| x * x).sum::<f64>().sqrt()))
                } else {
                    // Matrix: 2-norm = largest singular value.
                    let (_, s, _) = svd_compute(m)?;
                    Ok(Value::Scalar(s.first().copied().unwrap_or(0.0)))
                }
            }
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("norm: not applicable to non-numeric values".to_string())
            }
        },
        ("norm", 2) => match &args[1] {
            Value::Str(s) | Value::StringObj(s) => match s.as_str() {
                "fro" => match &args[0] {
                    Value::Scalar(n) => Ok(Value::Scalar(n.abs())),
                    Value::Matrix(m) => {
                        Ok(Value::Scalar(m.iter().map(|x| x * x).sum::<f64>().sqrt()))
                    }
                    _ => Err("norm: first argument must be numeric".to_string()),
                },
                other => Err(format!("norm: unknown norm type '{other}'")),
            },
            _ => {
                let p = scalar_arg(&args[1], name, 2)?;
                match &args[0] {
                    Value::Void => Err("norm: not applicable to void".to_string()),
                    Value::Scalar(n) => Ok(Value::Scalar(n.abs())),
                    Value::Complex(re, im) => Ok(Value::Scalar((re * re + im * im).sqrt().powf(p))),
                    Value::Matrix(m) => {
                        if m.nrows() > 1 && m.ncols() > 1 {
                            // Matrix norms.
                            if (p - 2.0).abs() < 1e-15 {
                                let (_, s, _) = svd_compute(m)?;
                                return Ok(Value::Scalar(s.first().copied().unwrap_or(0.0)));
                            } else if (p - 1.0).abs() < 1e-15 {
                                // Maximum absolute column sum.
                                let v = (0..m.ncols())
                                    .map(|j| m.column(j).iter().map(|&x| x.abs()).sum::<f64>())
                                    .fold(0.0_f64, f64::max);
                                return Ok(Value::Scalar(v));
                            } else if p == f64::INFINITY {
                                // Maximum absolute row sum.
                                let v = (0..m.nrows())
                                    .map(|i| m.row(i).iter().map(|&x| x.abs()).sum::<f64>())
                                    .fold(0.0_f64, f64::max);
                                return Ok(Value::Scalar(v));
                            }
                        }
                        // Vector (or general Lp).
                        if p == f64::INFINITY {
                            Ok(Value::Scalar(
                                m.iter().copied().fold(0.0_f64, |acc, x| acc.max(x.abs())),
                            ))
                        } else {
                            Ok(Value::Scalar(
                                m.iter().map(|x| x.abs().powf(p)).sum::<f64>().powf(1.0 / p),
                            ))
                        }
                    }
                    Value::Str(_)
                    | Value::StringObj(_)
                    | Value::Lambda(_)
                    | Value::Function { .. }
                    | Value::Tuple(_)
                    | Value::Cell(_)
                    | Value::Struct(_)
                    | Value::StructArray(_)
                    | Value::DateTime(_)
                    | Value::Duration(_)
                    | Value::DateTimeArray(_)
                    | Value::DurationArray(_) => {
                        Err("norm: not applicable to non-numeric values".to_string())
                    }
                }
            }
        },
        // --- Cumulative reductions ---
        ("cumsum", 1) => apply_cumulative(&args[0], |acc, x| acc + x),
        ("cumprod", 1) => apply_cumulative(&args[0], |acc, x| acc * x),
        // --- Sort ---
        ("sort", 1) => match &args[0] {
            Value::Void => Err("sort: not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            Value::Complex(_, _) => Err("sort: not applicable to complex values".to_string()),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("sort: not applicable to non-numeric values".to_string())
            }
            Value::Matrix(m) => {
                if m.nrows() > 1 && m.ncols() > 1 {
                    return Err("sort: input must be a vector".to_string());
                }
                let mut vals: Vec<f64> = m.iter().copied().collect();
                vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                Ok(Value::Matrix(
                    Array2::from_shape_vec(m.raw_dim(), vals).unwrap(),
                ))
            }
        },
        // --- Reshape ---
        ("reshape", 3) => {
            let r = scalar_arg(&args[1], name, 2)? as usize;
            let c = scalar_arg(&args[2], name, 3)? as usize;
            match &args[0] {
                Value::Void => Err("reshape: not applicable to void".to_string()),
                Value::Scalar(n) => {
                    if r * c != 1 {
                        return Err(format!("reshape: cannot reshape 1 element into {r}x{c}"));
                    }
                    Ok(Value::Matrix(
                        Array2::from_shape_vec((1, 1), vec![*n]).unwrap(),
                    ))
                }
                Value::Complex(_, _) => {
                    Err("reshape: not applicable to complex values".to_string())
                }
                Value::Str(_)
                | Value::StringObj(_)
                | Value::Lambda(_)
                | Value::Function { .. }
                | Value::Tuple(_)
                | Value::Cell(_)
                | Value::Struct(_)
                | Value::StructArray(_)
                | Value::DateTime(_)
                | Value::Duration(_)
                | Value::DateTimeArray(_)
                | Value::DurationArray(_) => {
                    Err("reshape: not applicable to non-numeric values".to_string())
                }
                Value::Matrix(m) => {
                    let total = m.len();
                    if r * c != total {
                        return Err(format!(
                            "reshape: cannot reshape {total} elements into {r}x{c}"
                        ));
                    }
                    // Column-major order (MATLAB convention)
                    let flat: Vec<f64> = (0..m.ncols())
                        .flat_map(|col| (0..m.nrows()).map(move |row| m[[row, col]]))
                        .collect();
                    let mut result = Array2::<f64>::zeros((r, c));
                    for (i, &v) in flat.iter().enumerate() {
                        result[[i % r, i / r]] = v;
                    }
                    Ok(Value::Matrix(result))
                }
            }
        }
        // --- Flip ---
        ("fliplr", 1) => match &args[0] {
            Value::Void => Err(format!("{name}: not applicable to void")),
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            Value::Complex(re, im) => Ok(Value::Complex(*re, *im)),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err(format!("{name}: not applicable to non-numeric values"))
            }
            Value::Matrix(m) => {
                let (nrows, ncols) = (m.nrows(), m.ncols());
                let mut result = m.clone();
                for r in 0..nrows {
                    for c in 0..ncols / 2 {
                        let tmp = result[[r, c]];
                        result[[r, c]] = result[[r, ncols - 1 - c]];
                        result[[r, ncols - 1 - c]] = tmp;
                    }
                }
                Ok(Value::Matrix(result))
            }
        },
        ("flipud", 1) => match &args[0] {
            Value::Void => Err(format!("{name}: not applicable to void")),
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            Value::Complex(re, im) => Ok(Value::Complex(*re, *im)),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err(format!("{name}: not applicable to non-numeric values"))
            }
            Value::Matrix(m) => {
                let (nrows, ncols) = (m.nrows(), m.ncols());
                let mut result = m.clone();
                for c in 0..ncols {
                    for r in 0..nrows / 2 {
                        let tmp = result[[r, c]];
                        result[[r, c]] = result[[nrows - 1 - r, c]];
                        result[[nrows - 1 - r, c]] = tmp;
                    }
                }
                Ok(Value::Matrix(result))
            }
        },
        // --- Find ---
        ("find", 1) => find_nonzero(&args[0], usize::MAX),
        ("find", 2) => {
            let k = scalar_arg(&args[1], name, 2)?;
            if k < 0.0 {
                return Err("find: k must be non-negative".to_string());
            }
            find_nonzero(&args[0], k as usize)
        }
        // --- Unique ---
        ("unique", 1) => match &args[0] {
            Value::Void => Err("unique: not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            Value::Matrix(m) => {
                let mut vals: Vec<f64> = m.iter().copied().collect();
                vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mut unique: Vec<f64> = Vec::new();
                for v in vals {
                    if unique.last().is_none_or(|&last| last != v) {
                        unique.push(v);
                    }
                }
                let n = unique.len();
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, n), unique).unwrap(),
                ))
            }
            Value::Complex(_, _) => Err("unique: not applicable to complex values".to_string()),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("unique: not applicable to non-numeric values".to_string())
            }
        },
        // --- Descriptive statistics ---
        ("std", 1) => apply_stat(&args[0], |s| stat_var_vec(s, false).sqrt(), "std"),
        ("std", 2) => {
            let w = scalar_arg(&args[1], name, 2)?;
            let population = w != 0.0;
            apply_stat(&args[0], |s| stat_var_vec(s, population).sqrt(), "std")
        }
        ("var", 1) => apply_stat(&args[0], |s| stat_var_vec(s, false), "var"),
        ("var", 2) => {
            let w = scalar_arg(&args[1], name, 2)?;
            let population = w != 0.0;
            apply_stat(&args[0], |s| stat_var_vec(s, population), "var")
        }
        ("cov", 1) => match &args[0] {
            Value::Scalar(_) => Ok(Value::Scalar(0.0)),
            Value::Matrix(m) => {
                if m.nrows() == 1 || m.ncols() == 1 {
                    let vals: Vec<f64> = m.iter().copied().collect();
                    Ok(Value::Scalar(stat_var_vec(&vals, false)))
                } else {
                    let (nobs, nvars) = (m.nrows(), m.ncols());
                    if nobs < 2 {
                        return Err("cov: need at least 2 observations".to_string());
                    }
                    let mut centered = m.clone();
                    for c in 0..nvars {
                        let col_mean: f64 = m.column(c).iter().sum::<f64>() / nobs as f64;
                        for r in 0..nobs {
                            centered[[r, c]] -= col_mean;
                        }
                    }
                    let denom = (nobs - 1) as f64;
                    let mut cov_mat = Array2::<f64>::zeros((nvars, nvars));
                    for i in 0..nvars {
                        for j in 0..nvars {
                            let dot: f64 =
                                (0..nobs).map(|r| centered[[r, i]] * centered[[r, j]]).sum();
                            cov_mat[[i, j]] = dot / denom;
                        }
                    }
                    Ok(Value::Matrix(cov_mat))
                }
            }
            _ => Err("cov: argument must be numeric".to_string()),
        },
        ("median", 1) => apply_stat(
            &args[0],
            |s| {
                if s.is_empty() {
                    return f64::NAN;
                }
                let mut v = s.to_vec();
                v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let n = v.len();
                if n % 2 == 0 {
                    (v[n / 2 - 1] + v[n / 2]) / 2.0
                } else {
                    v[n / 2]
                }
            },
            "median",
        ),
        ("mode", 1) => apply_stat(
            &args[0],
            |s| {
                if s.is_empty() {
                    return f64::NAN;
                }
                let mut counts: std::collections::HashMap<u64, usize> =
                    std::collections::HashMap::new();
                for &x in s {
                    *counts.entry(x.to_bits()).or_insert(0) += 1;
                }
                let max_count = counts.values().copied().max().unwrap_or(0);
                let mut candidates: Vec<f64> = counts
                    .iter()
                    .filter(|&(_, &c)| c == max_count)
                    .map(|(&bits, _)| f64::from_bits(bits))
                    .collect();
                candidates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                candidates[0]
            },
            "mode",
        ),
        ("skewness", 1) => apply_stat(
            &args[0],
            |s| {
                let n = s.len();
                if n == 0 {
                    return f64::NAN;
                }
                if n == 1 {
                    return 0.0;
                }
                let mean = s.iter().sum::<f64>() / n as f64;
                let m2 = s.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;
                if m2 == 0.0 {
                    return 0.0;
                }
                let m3 = s.iter().map(|&x| (x - mean).powi(3)).sum::<f64>() / n as f64;
                m3 / m2.powf(1.5)
            },
            "skewness",
        ),
        ("kurtosis", 1) => apply_stat(
            &args[0],
            |s| {
                let n = s.len();
                if n < 2 {
                    return f64::NAN;
                }
                let mean = s.iter().sum::<f64>() / n as f64;
                let m2 = s.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;
                if m2 == 0.0 {
                    return f64::NAN;
                }
                let m4 = s.iter().map(|&x| (x - mean).powi(4)).sum::<f64>() / n as f64;
                m4 / m2.powi(2)
            },
            "kurtosis",
        ),
        ("hist", n) if n == 1 || n == 2 => {
            let n_bins = if args.len() == 2 {
                scalar_arg(&args[1], name, 2)? as usize
            } else {
                10
            };
            if n_bins == 0 {
                return Err("hist: number of bins must be positive".to_string());
            }
            let vals = numeric_vec(&args[0], name)?;
            if vals.is_empty() {
                return Ok(Value::Void);
            }
            let min_v = vals.iter().cloned().fold(f64::INFINITY, f64::min);
            let max_v = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let range = if max_v > min_v { max_v - min_v } else { 1.0 };
            let mut counts = vec![0usize; n_bins];
            for &v in &vals {
                let b = ((v - min_v) / range * n_bins as f64) as usize;
                counts[b.min(n_bins - 1)] += 1;
            }
            let bar_cols: usize = std::env::var("COLUMNS")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(80)
                .saturating_sub(26)
                .max(10);
            let max_count = *counts.iter().max().unwrap_or(&1).max(&1);
            let mut output = String::new();
            for (i, &c) in counts.iter().enumerate() {
                let lo = min_v + range * (i as f64 / n_bins as f64);
                let hi = min_v + range * ((i + 1) as f64 / n_bins as f64);
                let bar_len = c * bar_cols / max_count;
                output.push_str(&format!(
                    "{lo:8.4} {hi:8.4} |{bar:<bar_cols$}| {c}\n",
                    bar = "#".repeat(bar_len),
                ));
            }
            match io.as_deref_mut() {
                Some(ctx) => ctx.write_to_fd(1, &output)?,
                None => {
                    print!("{output}");
                    std::io::stdout().flush().ok();
                }
            }
            Ok(Value::Void)
        }
        ("histc", 2) => {
            let vals = numeric_vec(&args[0], name)?;
            let edges = numeric_vec(&args[1], name)?;
            if edges.is_empty() {
                return Err("histc: edges must not be empty".to_string());
            }
            let n_edges = edges.len();
            let mut counts = vec![0.0f64; n_edges];
            for &v in &vals {
                // Linear scan — fine for typical edge counts
                let last = n_edges - 1;
                if v == edges[last] {
                    counts[last] += 1.0;
                } else {
                    for i in 0..last {
                        if v >= edges[i] && v < edges[i + 1] {
                            counts[i] += 1.0;
                            break;
                        }
                    }
                }
            }
            Ok(Value::Matrix(
                Array2::from_shape_vec((1, n_edges), counts).unwrap(),
            ))
        }
        // --- Percentiles and distributions ---
        ("prctile", 2) => {
            let p_vals = numeric_vec(&args[1], name)?;
            let n_p = p_vals.len();

            // Sort one column of floats and compute all requested percentiles.
            let compute_col = |vals: &[f64]| -> Vec<f64> {
                let mut s = vals.to_vec();
                s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                p_vals.iter().map(|&p| percentile_sorted(&s, p)).collect()
            };

            match &args[0] {
                Value::Scalar(n) => {
                    let pr = compute_col(&[*n]);
                    if n_p == 1 {
                        Ok(Value::Scalar(pr[0]))
                    } else {
                        Ok(Value::Matrix(Array2::from_shape_vec((1, n_p), pr).unwrap()))
                    }
                }
                Value::Matrix(m) if m.nrows() == 1 || m.ncols() == 1 => {
                    let vals: Vec<f64> = m.iter().copied().collect();
                    let pr = compute_col(&vals);
                    if n_p == 1 {
                        Ok(Value::Scalar(pr[0]))
                    } else {
                        Ok(Value::Matrix(Array2::from_shape_vec((1, n_p), pr).unwrap()))
                    }
                }
                Value::Matrix(m) => {
                    // M×N matrix: column-wise → n_p × ncols result
                    let ncols = m.ncols();
                    let mut result = Array2::<f64>::zeros((n_p, ncols));
                    for j in 0..ncols {
                        let col: Vec<f64> = m.column(j).iter().copied().collect();
                        let pr = compute_col(&col);
                        for (i, &v) in pr.iter().enumerate() {
                            result[[i, j]] = v;
                        }
                    }
                    if n_p == 1 {
                        let row: Vec<f64> = result.row(0).iter().copied().collect();
                        Ok(Value::Matrix(
                            Array2::from_shape_vec((1, ncols), row).unwrap(),
                        ))
                    } else {
                        Ok(Value::Matrix(result))
                    }
                }
                _ => Err("prctile: first argument must be numeric".to_string()),
            }
        }
        ("iqr", 1) => apply_stat(
            &args[0],
            |s| {
                let mut sorted = s.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                percentile_sorted(&sorted, 75.0) - percentile_sorted(&sorted, 25.0)
            },
            "iqr",
        ),
        ("zscore", 1) => match &args[0] {
            Value::Scalar(_) => Ok(Value::Scalar(0.0)),
            Value::Matrix(m) => {
                if m.nrows() == 1 || m.ncols() == 1 {
                    let vals: Vec<f64> = m.iter().copied().collect();
                    let n = vals.len() as f64;
                    let mean = vals.iter().sum::<f64>() / n;
                    let s = stat_var_vec(&vals, false).sqrt();
                    let result: Vec<f64> = vals
                        .iter()
                        .map(|&x| if s == 0.0 { 0.0 } else { (x - mean) / s })
                        .collect();
                    Ok(Value::Matrix(
                        Array2::from_shape_vec(m.raw_dim(), result).unwrap(),
                    ))
                } else {
                    let (nrows, ncols) = (m.nrows(), m.ncols());
                    let mut result = m.clone();
                    for j in 0..ncols {
                        let col: Vec<f64> = m.column(j).iter().copied().collect();
                        let mean = col.iter().sum::<f64>() / col.len() as f64;
                        let s = stat_var_vec(&col, false).sqrt();
                        for i in 0..nrows {
                            result[[i, j]] = if s == 0.0 {
                                0.0
                            } else {
                                (m[[i, j]] - mean) / s
                            };
                        }
                    }
                    Ok(Value::Matrix(result))
                }
            }
            _ => Err("zscore: argument must be numeric".to_string()),
        },
        // diag(v) — vector → diagonal matrix; diag(A) → column vector of main diagonal.
        ("diag", 1) => match &args[0] {
            Value::Scalar(n) => Ok(Value::Matrix(Array2::from_elem((1, 1), *n))),
            Value::Matrix(m) => {
                let (rows, cols) = (m.nrows(), m.ncols());
                if rows == 1 || cols == 1 {
                    // vector → N×N diagonal matrix
                    let v: Vec<f64> = m.iter().copied().collect();
                    let n = v.len();
                    let mut result = Array2::<f64>::zeros((n, n));
                    for (i, &val) in v.iter().enumerate() {
                        result[[i, i]] = val;
                    }
                    Ok(Value::Matrix(result))
                } else {
                    // matrix → extract main diagonal as N×1 column vector
                    let n = rows.min(cols);
                    let d: Vec<f64> = (0..n).map(|i| m[[i, i]]).collect();
                    Ok(Value::Matrix(Array2::from_shape_vec((n, 1), d).unwrap()))
                }
            }
            Value::Void => Err("diag: not applicable to void".to_string()),
            Value::Complex(_, _) => Err("diag: not applicable to complex values".to_string()),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("diag: not applicable to non-numeric values".to_string())
            }
        },

        // --- Complex built-ins ---
        // real(z) — real part; works on scalars too (returns the value unchanged).
        ("real", 1) => match &args[0] {
            Value::Void => Err("real: not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            Value::Complex(re, _) => Ok(Value::Scalar(*re)),
            Value::Matrix(_) => Err("real: not applicable to matrices".to_string()),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("real: not applicable to non-numeric values".to_string())
            }
        },
        // imag(z) — imaginary part; returns 0.0 for real scalars.
        ("imag", 1) => match &args[0] {
            Value::Void => Err("imag: not applicable to void".to_string()),
            Value::Scalar(_) => Ok(Value::Scalar(0.0)),
            Value::Complex(_, im) => Ok(Value::Scalar(*im)),
            Value::Matrix(_) => Err("imag: not applicable to matrices".to_string()),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("imag: not applicable to non-numeric values".to_string())
            }
        },
        // abs(z) — modulus; overloads scalar abs.
        ("abs", 1) => match &args[0] {
            Value::Void => Err("abs: not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(n.abs())),
            Value::Complex(re, im) => Ok(Value::Scalar((re * re + im * im).sqrt())),
            Value::Matrix(m) => Ok(Value::Matrix(m.mapv(|x| x.abs()))),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("abs: not applicable to non-numeric values".to_string())
            }
        },
        // angle(z) — argument in radians; returns 0 for non-negative reals.
        ("angle", 1) => match &args[0] {
            Value::Void => Err("angle: not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(if *n >= 0.0 {
                0.0
            } else {
                std::f64::consts::PI
            })),
            Value::Complex(re, im) => Ok(Value::Scalar(im.atan2(*re))),
            Value::Matrix(_) => Err("angle: not applicable to matrices".to_string()),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("angle: not applicable to non-numeric values".to_string())
            }
        },
        // conj(z) — complex conjugate; scalars are unchanged.
        ("conj", 1) => match &args[0] {
            Value::Void => Err("conj: not applicable to void".to_string()),
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            Value::Complex(re, im) => Ok(make_complex(*re, -*im)),
            Value::Matrix(m) => Ok(Value::Matrix(m.clone())),
            Value::Str(_)
            | Value::StringObj(_)
            | Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => {
                Err("conj: not applicable to non-numeric values".to_string())
            }
        },
        // complex(re, im) — construct complex from two reals.
        ("complex", 2) => {
            let re = scalar_arg(&args[0], name, 1)?;
            let im = scalar_arg(&args[1], name, 2)?;
            Ok(make_complex(re, im))
        }
        // isreal(z) — 1.0 if imaginary part is zero, 0.0 otherwise.
        ("isreal", 1) => match &args[0] {
            Value::Void => Ok(Value::Scalar(0.0)),
            Value::Scalar(_) => Ok(Value::Scalar(1.0)),
            Value::Complex(_, im) => Ok(Value::Scalar(if *im == 0.0 { 1.0 } else { 0.0 })),
            Value::Matrix(_) => Ok(Value::Scalar(1.0)),
            // Strings are not real numbers; functions are not numbers
            Value::Str(_) | Value::StringObj(_) => Ok(Value::Scalar(0.0)),
            Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => Ok(Value::Scalar(0.0)),
        },
        // --- String built-ins ---
        // num2str(x) — convert number to char array string
        ("num2str", 1) => match &args[0] {
            Value::Void => Err("num2str: not applicable to void".to_string()),
            Value::Str(s) => Ok(Value::Str(s.clone())),
            Value::StringObj(s) => Ok(Value::Str(s.clone())),
            Value::Scalar(n) => Ok(Value::Str(fmt_auto_sig(*n, 5))),
            Value::Complex(re, im) => Ok(Value::Str(format_complex(*re, *im, &FormatMode::Short))),
            Value::Matrix(m) => {
                let s = m
                    .iter()
                    .map(|x| fmt_auto_sig(*x, 5))
                    .collect::<Vec<_>>()
                    .join("  ");
                Ok(Value::Str(s))
            }
            Value::Lambda(_)
            | Value::Function { .. }
            | Value::Tuple(_)
            | Value::Cell(_)
            | Value::Struct(_)
            | Value::StructArray(_)
            | Value::DateTime(_)
            | Value::Duration(_)
            | Value::DateTimeArray(_)
            | Value::DurationArray(_) => Err("num2str: not applicable to this type".to_string()),
        },
        // num2str(x, N) — N significant digits
        ("num2str", 2) => {
            let n = scalar_arg(&args[1], name, 2)? as usize;
            match &args[0] {
                Value::Void => Err("num2str: not applicable to void".to_string()),
                Value::Str(s) => Ok(Value::Str(s.clone())),
                Value::StringObj(s) => Ok(Value::Str(s.clone())),
                Value::Scalar(v) => Ok(Value::Str(fmt_auto_sig(*v, n))),
                Value::Complex(re, im) => {
                    Ok(Value::Str(format_complex(*re, *im, &FormatMode::Custom(n))))
                }
                Value::Matrix(m) => {
                    let s = m
                        .iter()
                        .map(|x| fmt_auto_sig(*x, n))
                        .collect::<Vec<_>>()
                        .join("  ");
                    Ok(Value::Str(s))
                }
                Value::Lambda(_)
                | Value::Function { .. }
                | Value::Tuple(_)
                | Value::Cell(_)
                | Value::Struct(_)
                | Value::StructArray(_)
                | Value::DateTime(_)
                | Value::Duration(_)
                | Value::DateTimeArray(_)
                | Value::DurationArray(_) => {
                    Err("num2str: not applicable to this type".to_string())
                }
            }
        }
        // str2double(s) — parse string as f64; return NaN on failure
        ("str2double", 1) => {
            let s = string_arg(&args[0], name, 1)?;
            match s.trim().parse::<f64>() {
                Ok(n) => Ok(Value::Scalar(n)),
                Err(_) => Ok(Value::Scalar(f64::NAN)),
            }
        }
        // str2num(s) — parse string as f64; return error on failure
        ("str2num", 1) => {
            let s = string_arg(&args[0], name, 1)?;
            s.trim()
                .parse::<f64>()
                .map(Value::Scalar)
                .map_err(|_| format!("str2num: cannot convert '{}' to number", s.trim()))
        }
        // strcat(a, b, ...) — concatenate strings
        ("strcat", n) if n >= 2 => {
            let mut result = String::new();
            let mut any_obj = false;
            for (i, arg) in args.iter().enumerate() {
                match arg {
                    Value::Str(s) => result.push_str(s.trim_end()),
                    Value::StringObj(s) => {
                        result.push_str(s);
                        any_obj = true;
                    }
                    _ => return Err(format!("strcat: argument {} must be a string", i + 1)),
                }
            }
            if any_obj {
                Ok(Value::StringObj(result))
            } else {
                Ok(Value::Str(result))
            }
        }
        // ischar(s) — 1.0 if char array, 0.0 otherwise
        ("ischar", 1) => Ok(Value::Scalar(if matches!(&args[0], Value::Str(_)) {
            1.0
        } else {
            0.0
        })),
        // isstring(s) — 1.0 if string object, 0.0 otherwise
        ("isstring", 1) => Ok(Value::Scalar(if matches!(&args[0], Value::StringObj(_)) {
            1.0
        } else {
            0.0
        })),
        // --- Struct built-ins ---
        // struct('k1',v1,'k2',v2,...) — construct a scalar struct from name-value pairs
        ("struct", _) => {
            if !args.len().is_multiple_of(2) {
                return Err(
                    "struct: requires an even number of arguments (name, value, ...)".to_string(),
                );
            }
            let mut map = IndexMap::new();
            for pair in args.chunks(2) {
                let key = match &pair[0] {
                    Value::Str(s) | Value::StringObj(s) => s.clone(),
                    _ => return Err("struct: field names must be strings".to_string()),
                };
                map.insert(key, pair[1].clone());
            }
            Ok(Value::Struct(map))
        }
        // fieldnames(s) — cell array of field names in insertion order
        ("fieldnames", 1) => match &args[0] {
            Value::Struct(map) => {
                let names: Vec<Value> = map.keys().map(|k| Value::Str(k.clone())).collect();
                Ok(Value::Cell(names))
            }
            Value::StructArray(arr) => {
                // Use field names from first element
                let names: Vec<Value> = arr
                    .first()
                    .map(|m| m.keys().map(|k| Value::Str(k.clone())).collect())
                    .unwrap_or_default();
                Ok(Value::Cell(names))
            }
            _ => Err("fieldnames: argument must be a struct".to_string()),
        },
        // isfield(s, 'name') — 1.0 if field exists, 0.0 otherwise
        ("isfield", 2) => {
            let field = match &args[1] {
                Value::Str(s) | Value::StringObj(s) => s.clone(),
                _ => return Err("isfield: second argument must be a string".to_string()),
            };
            Ok(Value::Scalar(match &args[0] {
                Value::Struct(map) if map.contains_key(&field) => 1.0,
                Value::StructArray(arr) if arr.first().is_some_and(|m| m.contains_key(&field)) => {
                    1.0
                }
                _ => 0.0,
            }))
        }
        // rmfield(s, 'name') — copy of struct with field removed
        ("rmfield", 2) => {
            let field = match &args[1] {
                Value::Str(s) | Value::StringObj(s) => s.clone(),
                _ => return Err("rmfield: second argument must be a string".to_string()),
            };
            match &args[0] {
                Value::Struct(map) => {
                    if !map.contains_key(&field) {
                        return Err(format!("rmfield: field '{field}' does not exist"));
                    }
                    let mut updated = map.clone();
                    updated.shift_remove(&field);
                    Ok(Value::Struct(updated))
                }
                Value::StructArray(arr) => {
                    let updated: Result<Vec<_>, _> = arr
                        .iter()
                        .map(|m| {
                            if !m.contains_key(&field) {
                                return Err(format!("rmfield: field '{field}' does not exist"));
                            }
                            let mut m2 = m.clone();
                            m2.shift_remove(&field);
                            Ok(m2)
                        })
                        .collect();
                    Ok(Value::StructArray(updated?))
                }
                _ => Err("rmfield: first argument must be a struct".to_string()),
            }
        }
        // isstruct(v) — 1.0 if v is a struct or struct array, 0.0 otherwise
        ("isstruct", 1) => Ok(Value::Scalar(
            if matches!(&args[0], Value::Struct(_) | Value::StructArray(_)) {
                1.0
            } else {
                0.0
            },
        )),
        // --- Cell array built-ins ---
        // isempty(v) — 1.0 if v has no elements, 0.0 otherwise.
        // Matches MATLAB: empty matrix, empty string, empty cell, or Void are empty.
        ("isempty", 1) => {
            let empty = match &args[0] {
                Value::Matrix(m) => m.is_empty(),
                Value::Str(s) | Value::StringObj(s) => s.is_empty(),
                Value::Cell(v) => v.is_empty(),
                Value::Void => true,
                _ => false,
            };
            Ok(Value::Scalar(if empty { 1.0 } else { 0.0 }))
        }
        // iscell(v) — 1.0 if v is a cell array, 0.0 otherwise
        ("iscell", 1) => Ok(Value::Scalar(if matches!(&args[0], Value::Cell(_)) {
            1.0
        } else {
            0.0
        })),
        // cell(n) — create 1×n cell of Scalar(0.0) slots
        ("cell", 1) => {
            let n = scalar_arg(&args[0], name, 1)? as usize;
            Ok(Value::Cell(vec![Value::Scalar(0.0); n]))
        }
        // cell(m, n) — create 1×(m*n) cell (2-D layout deferred; stored flat)
        ("cell", 2) => {
            let m = scalar_arg(&args[0], name, 1)? as usize;
            let n = scalar_arg(&args[1], name, 2)? as usize;
            Ok(Value::Cell(vec![Value::Scalar(0.0); m * n]))
        }
        // cellfun(f, c) — apply f to each element of cell c.
        // Returns Value::Matrix when all results are scalars; otherwise Value::Cell.
        ("cellfun", 2) => {
            let f = args[0].clone();
            match &args[1] {
                Value::Cell(elems) => {
                    let elems = elems.clone();
                    let mut results = Vec::with_capacity(elems.len());
                    for elem in &elems {
                        let result =
                            call_function_value(&f, std::slice::from_ref(elem), io.as_deref_mut())?;
                        results.push(result);
                    }
                    // Try uniform output (all scalars)
                    let all_scalar = results.iter().all(|v| matches!(v, Value::Scalar(_)));
                    if all_scalar {
                        let vals: Vec<f64> = results
                            .iter()
                            .map(|v| {
                                if let Value::Scalar(n) = v {
                                    *n
                                } else {
                                    unreachable!()
                                }
                            })
                            .collect();
                        let n = vals.len();
                        if n == 0 {
                            Ok(Value::Matrix(Array2::zeros((1, 0))))
                        } else {
                            Ok(Value::Matrix(Array2::from_shape_vec((1, n), vals).unwrap()))
                        }
                    } else {
                        Ok(Value::Cell(results))
                    }
                }
                _ => Err("cellfun: second argument must be a cell array".to_string()),
            }
        }
        // arrayfun(f, v) — apply f element-wise to matrix v.
        // Returns same-shape Value::Matrix (scalar-returning f only).
        ("arrayfun", 2) => {
            let f = args[0].clone();
            match &args[1] {
                Value::Matrix(m) => {
                    let m = m.clone();
                    let mut flat = Vec::with_capacity(m.len());
                    // Iterate in column-major order
                    for col in 0..m.ncols() {
                        for row in 0..m.nrows() {
                            let elem = Value::Scalar(m[[row, col]]);
                            let result = call_function_value(&f, &[elem], io.as_deref_mut())?;
                            match result {
                                Value::Scalar(n) => flat.push(n),
                                _ => {
                                    return Err(
                                        "arrayfun: function must return a scalar".to_string()
                                    );
                                }
                            }
                        }
                    }
                    Ok(Value::Matrix(
                        Array2::from_shape_vec((m.nrows(), m.ncols()), flat).unwrap(),
                    ))
                }
                Value::Scalar(n) => {
                    let elem = Value::Scalar(*n);
                    let result = call_function_value(&f, &[elem], io.as_deref_mut())?;
                    Ok(result)
                }
                _ => {
                    Err("arrayfun: second argument must be a numeric matrix or scalar".to_string())
                }
            }
        }
        // lower(s) — convert to lowercase
        ("lower", 1) => match &args[0] {
            Value::Str(s) => Ok(Value::Str(s.to_lowercase())),
            Value::StringObj(s) => Ok(Value::StringObj(s.to_lowercase())),
            _ => Err("lower: argument must be a string".to_string()),
        },
        // upper(s) — convert to uppercase
        ("upper", 1) => match &args[0] {
            Value::Str(s) => Ok(Value::Str(s.to_uppercase())),
            Value::StringObj(s) => Ok(Value::StringObj(s.to_uppercase())),
            _ => Err("upper: argument must be a string".to_string()),
        },
        // strtrim(s) — trim leading/trailing whitespace
        ("strtrim", 1) => match &args[0] {
            Value::Str(s) => Ok(Value::Str(s.trim().to_string())),
            Value::StringObj(s) => Ok(Value::StringObj(s.trim().to_string())),
            _ => Err("strtrim: argument must be a string".to_string()),
        },
        // strrep(s, old, new) — replace all occurrences
        ("strrep", 3) => {
            let s = string_arg(&args[0], name, 1)?.to_string();
            let old = string_arg(&args[1], name, 2)?;
            let new = string_arg(&args[2], name, 3)?;
            let result = s.replace(old, new);
            match &args[0] {
                Value::StringObj(_) => Ok(Value::StringObj(result)),
                _ => Ok(Value::Str(result)),
            }
        }
        // strcmp(a, b) — case-sensitive string comparison
        ("strcmp", 2) => {
            let a = string_arg(&args[0], name, 1)?;
            let b = string_arg(&args[1], name, 2)?;
            Ok(Value::Scalar(bool_to_f64(a == b)))
        }
        // strcmpi(a, b) — case-insensitive comparison
        ("strcmpi", 2) => {
            let a = string_arg(&args[0], name, 1)?.to_lowercase();
            let b = string_arg(&args[1], name, 2)?.to_lowercase();
            Ok(Value::Scalar(bool_to_f64(a == b)))
        }
        // disp(x) — display value without variable name, like MATLAB disp()
        ("disp", 1) => {
            use std::io::Write;
            let mode = get_display_fmt();
            let output = match &args[0] {
                Value::Str(s) | Value::StringObj(s) => format!("{s}\n"),
                v => match format_value_full(v, &mode) {
                    Some(block) => format!("{block}\n\n"),
                    None => format!("{}\n", format_value(v, get_display_base(), &mode)),
                },
            };
            match io {
                Some(ctx) => ctx.write_to_fd(1, &output)?,
                None => {
                    print!("{output}");
                    std::io::stdout().flush().ok();
                }
            }
            Ok(Value::Void)
        }
        // sprintf(fmt, ...) — format and return as char array
        ("sprintf", n) if n >= 1 => {
            let fmt = string_arg(&args[0], name, 1)?.to_string();
            let result = format_printf(&fmt, &args[1..])?;
            Ok(Value::Str(result))
        }
        // fprintf([fd,] fmt, ...) — format and print; fd defaults to 1 (stdout)
        ("fprintf", n) if n >= 1 => {
            // If first arg is a numeric scalar, treat it as a file descriptor.
            let (fd, fmt_idx) = match &args[0] {
                Value::Scalar(n) => (*n as i32, 1),
                _ => (1, 0),
            };
            if fmt_idx >= args.len() {
                return Err("fprintf: missing format string".to_string());
            }
            let fmt = string_arg(&args[fmt_idx], name, fmt_idx + 1)?.to_string();
            let output = format_printf(&fmt, &args[fmt_idx + 1..])?;
            match io {
                Some(ctx) => ctx.write_to_fd(fd, &output)?,
                None => {
                    // No I/O context: only stdout (fd 1) is allowed
                    if fd == 1 {
                        use std::io::Write;
                        print!("{output}");
                        std::io::stdout().flush().ok();
                    } else {
                        return Err("fprintf: file I/O not available in this context".to_string());
                    }
                }
            }
            Ok(Value::Void)
        }
        // fopen(path, mode) — open a file; returns fd or -1 on failure
        ("fopen", 2) => {
            let path = string_arg(&args[0], name, 1)?;
            let mode = string_arg(&args[1], name, 2)?;
            match io {
                Some(ctx) => Ok(Value::Scalar(ctx.fopen(path, mode) as f64)),
                None => Err("fopen: file I/O not available in this context".to_string()),
            }
        }
        // fclose(fd) or fclose('all')
        ("fclose", 1) => match &args[0] {
            Value::Str(s) if s == "all" => {
                if let Some(ctx) = io {
                    ctx.fclose_all();
                }
                Ok(Value::Scalar(0.0))
            }
            _ => {
                let fd = scalar_arg(&args[0], name, 1)? as i32;
                match io {
                    Some(ctx) => Ok(Value::Scalar(ctx.fclose(fd) as f64)),
                    None => Err("fclose: file I/O not available in this context".to_string()),
                }
            }
        },
        // fgetl(fd) — read line, strip newline; returns Str or Scalar(-1) at EOF
        ("fgetl", 1) => {
            let fd = scalar_arg(&args[0], name, 1)? as i32;
            match io {
                Some(ctx) => match ctx.fgetl(fd) {
                    Some(line) => Ok(Value::Str(line)),
                    None => Ok(Value::Scalar(-1.0)),
                },
                None => Err("fgetl: file I/O not available in this context".to_string()),
            }
        }
        // fgets(fd) — read line, keep newline; returns Str or Scalar(-1) at EOF
        ("fgets", 1) => {
            let fd = scalar_arg(&args[0], name, 1)? as i32;
            match io {
                Some(ctx) => match ctx.fgets(fd) {
                    Some(line) => Ok(Value::Str(line)),
                    None => Ok(Value::Scalar(-1.0)),
                },
                None => Err("fgets: file I/O not available in this context".to_string()),
            }
        }
        // isfile(path) — 1.0 if path exists and is a regular file, else 0.0
        ("isfile", 1) => {
            let path = string_arg(&args[0], name, 1)?;
            let is_file = std::fs::metadata(path)
                .map(|m| m.is_file())
                .unwrap_or(false);
            Ok(Value::Scalar(bool_to_f64(is_file)))
        }
        // isfolder(path) — 1.0 if path exists and is a directory, else 0.0
        ("isfolder", 1) => {
            let path = string_arg(&args[0], name, 1)?;
            let is_dir = std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false);
            Ok(Value::Scalar(bool_to_f64(is_dir)))
        }
        // genpath(dir) — return dir and all subdirectories as a path separator-delimited string
        ("genpath", 1) => {
            let root = string_arg(&args[0], name, 1)?;
            let sep = if cfg!(windows) { ';' } else { ':' };
            let mut dirs: Vec<String> = Vec::new();
            let mut stack = vec![std::path::PathBuf::from(root)];
            while let Some(dir) = stack.pop() {
                if !dir.is_dir() {
                    continue;
                }
                dirs.push(dir.to_string_lossy().into_owned());
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    let mut children: Vec<std::path::PathBuf> = entries
                        .filter_map(|e| e.ok())
                        .map(|e| e.path())
                        .filter(|p| p.is_dir())
                        .collect();
                    children.sort();
                    children.reverse();
                    stack.extend(children);
                }
            }
            Ok(Value::Str(dirs.join(&sep.to_string())))
        }
        // pwd() — current working directory as a char array (parser sends ans as sole arg for empty calls)
        ("pwd", _) => {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            Ok(Value::Str(cwd))
        }
        // exist(name) — check var (1), then file (2), else 0
        ("exist", 1) => {
            let name_arg = string_arg(&args[0], name, 1)?;
            if env.contains_key(name_arg) {
                Ok(Value::Scalar(1.0))
            } else if std::path::Path::new(name_arg).is_file() {
                Ok(Value::Scalar(2.0))
            } else {
                Ok(Value::Scalar(0.0))
            }
        }
        // exist(name, 'var') or exist(name, 'file')
        ("exist", 2) => {
            let name_arg = string_arg(&args[0], name, 1)?;
            let kind = string_arg(&args[1], name, 2)?;
            match kind {
                "var" => Ok(Value::Scalar(if env.contains_key(name_arg) {
                    1.0
                } else {
                    0.0
                })),
                "file" => Ok(Value::Scalar(if std::path::Path::new(name_arg).is_file() {
                    2.0
                } else {
                    0.0
                })),
                other => Err(format!(
                    "exist: unknown type '{other}', expected 'var' or 'file'"
                )),
            }
        }
        // dlmread(path) / dlmread(path, delim)
        ("dlmread", 1) => {
            let path = string_arg(&args[0], name, 1)?.to_string();
            dlmread_impl(&path, None)
        }
        ("dlmread", 2) => {
            let path = string_arg(&args[0], name, 1)?.to_string();
            let delim = interpret_delim(string_arg(&args[1], name, 2)?);
            dlmread_impl(&path, Some(delim))
        }
        // dlmwrite(path, A) / dlmwrite(path, A, delim)
        ("dlmwrite", 2) => {
            let path = string_arg(&args[0], name, 1)?.to_string();
            dlmwrite_impl(&path, &args[1], None)
        }
        ("dlmwrite", 3) => {
            let path = string_arg(&args[0], name, 1)?.to_string();
            let delim = interpret_delim(string_arg(&args[2], name, 3)?);
            dlmwrite_impl(&path, &args[1], Some(delim))
        }
        // readmatrix(path) / readmatrix(path, 'Delimiter', d)
        ("readmatrix", n) if n == 1 || n == 3 => {
            let path = string_arg(&args[0], name, 1)?.to_string();
            let delim = parse_delimiter_opt(name, args, 1)?;
            readmatrix_impl(&path, delim)
        }
        // readtable(path) / readtable(path, 'Delimiter', d)
        ("readtable", n) if n == 1 || n == 3 => {
            let path = string_arg(&args[0], name, 1)?.to_string();
            let delim = parse_delimiter_opt(name, args, 1)?;
            readtable_impl(&path, delim)
        }
        // writetable(T, path) / writetable(T, path, 'Delimiter', d)
        ("writetable", n) if n == 2 || n == 4 => {
            let path = string_arg(&args[1], name, 2)?.to_string();
            let delim = parse_delimiter_opt(name, args, 2)?;
            writetable_impl(&args[0], &path, delim)
        }
        // xor(a, b) — element-wise XOR: (a != 0) XOR (b != 0)
        ("xor", 2) => {
            let a = &args[0];
            let b = &args[1];
            match (a, b) {
                (Value::Scalar(x), Value::Scalar(y)) => {
                    Ok(Value::Scalar(bool_to_f64((*x != 0.0) ^ (*y != 0.0))))
                }
                (Value::Matrix(mx), Value::Matrix(my)) => {
                    if mx.shape() != my.shape() {
                        return Err("xor: matrices must have the same dimensions".to_string());
                    }
                    Ok(Value::Matrix(ndarray::Zip::from(mx).and(my).map_collect(
                        |a, b| bool_to_f64((*a != 0.0) ^ (*b != 0.0)),
                    )))
                }
                (Value::Scalar(s), Value::Matrix(m)) => {
                    let sv = *s != 0.0;
                    Ok(Value::Matrix(m.mapv(|x| bool_to_f64(sv ^ (x != 0.0)))))
                }
                (Value::Matrix(m), Value::Scalar(s)) => {
                    let sv = *s != 0.0;
                    Ok(Value::Matrix(m.mapv(|x| bool_to_f64((x != 0.0) ^ sv))))
                }
                _ => Err("xor: arguments must be numeric".to_string()),
            }
        }
        // not(a) — element-wise NOT (alias for ~a)
        ("not", 1) => apply_elem(&args[0], |x| if x == 0.0 { 1.0 } else { 0.0 }),
        // int2str(x) — round to nearest integer, return as char array
        ("int2str", 1) => match &args[0] {
            Value::Scalar(n) => Ok(Value::Str(format!("{}", n.round() as i64))),
            Value::Matrix(m) => {
                let parts: Vec<String> =
                    m.iter().map(|x| format!("{}", x.round() as i64)).collect();
                Ok(Value::Str(parts.join("  ")))
            }
            _ => Err("int2str: argument must be numeric".to_string()),
        },
        // mat2str(A) — matrix to MATLAB literal syntax string
        ("mat2str", 1) => match &args[0] {
            Value::Scalar(n) => Ok(Value::Str(format!("{n}"))),
            Value::Matrix(m) => {
                if m.nrows() == 0 || m.ncols() == 0 {
                    return Ok(Value::Str("[]".to_string()));
                }
                let mut s = String::from("[");
                for (r, row) in m.rows().into_iter().enumerate() {
                    if r > 0 {
                        s.push(';');
                    }
                    for (c, val) in row.iter().enumerate() {
                        if c > 0 {
                            s.push(' ');
                        }
                        s.push_str(&format!("{val}"));
                    }
                }
                s.push(']');
                Ok(Value::Str(s))
            }
            _ => Err("mat2str: argument must be numeric".to_string()),
        },
        // strsplit(s, delim) — split string by delimiter, return cell array
        ("strsplit", 2) => {
            let s = string_arg(&args[0], name, 1)?.to_string();
            let delim = string_arg(&args[1], name, 2)?.to_string();
            let parts: Vec<Value> = s
                .split(delim.as_str())
                .map(|p| Value::Str(p.to_string()))
                .collect();
            Ok(Value::Cell(parts))
        }
        // strsplit(s) — split on whitespace
        ("strsplit", 1) => {
            let s = string_arg(&args[0], name, 1)?.to_string();
            let parts: Vec<Value> = s
                .split_whitespace()
                .map(|p| Value::Str(p.to_string()))
                .collect();
            Ok(Value::Cell(parts))
        }
        // strjoin(c) / strjoin(c, delim) — join a cell array of strings
        ("strjoin", n) if n == 1 || n == 2 => {
            let cells = match &args[0] {
                Value::Cell(v) => v,
                _ => {
                    return Err(
                        "strjoin: first argument must be a cell array of strings".to_string()
                    );
                }
            };
            let delim = if n == 2 {
                string_arg(&args[1], name, 2)?.to_string()
            } else {
                " ".to_string()
            };
            let mut parts: Vec<String> = Vec::with_capacity(cells.len());
            for (i, v) in cells.iter().enumerate() {
                match v {
                    Value::Str(s) | Value::StringObj(s) => parts.push(s.clone()),
                    _ => return Err(format!("strjoin: element {} must be a string", i + 1)),
                }
            }
            Ok(Value::Str(parts.join(&delim)))
        }
        // contains(s, pat) / contains(s, pat, 'IgnoreCase', tf) — substring check
        ("contains", 2) => {
            let s = string_arg(&args[0], name, 1)?;
            let pat = string_arg(&args[1], name, 2)?;
            Ok(Value::Scalar(bool_to_f64(s.contains(pat))))
        }
        ("contains", 4) => {
            let s = string_arg(&args[0], name, 1)?;
            let pat = string_arg(&args[1], name, 2)?;
            let key = string_arg(&args[2], name, 3)?;
            if key != "IgnoreCase" {
                return Err(format!(
                    "contains: unknown option '{key}'; expected 'IgnoreCase'"
                ));
            }
            let ignore = match &args[3] {
                Value::Scalar(n) => *n != 0.0,
                _ => return Err("contains: 'IgnoreCase' value must be a scalar".to_string()),
            };
            if ignore {
                Ok(Value::Scalar(bool_to_f64(
                    s.to_lowercase().contains(&pat.to_lowercase()),
                )))
            } else {
                Ok(Value::Scalar(bool_to_f64(s.contains(pat))))
            }
        }
        // startsWith(s, pat) — prefix check
        ("startsWith", 2) => {
            let s = string_arg(&args[0], name, 1)?;
            let pat = string_arg(&args[1], name, 2)?;
            Ok(Value::Scalar(bool_to_f64(s.starts_with(pat))))
        }
        // endsWith(s, pat) — suffix check
        ("endsWith", 2) => {
            let s = string_arg(&args[0], name, 1)?;
            let pat = string_arg(&args[1], name, 2)?;
            Ok(Value::Scalar(bool_to_f64(s.ends_with(pat))))
        }
        // regexp(s, pat) / regexp(s, pat, 'match') — regular expression search
        ("regexp", 2) => {
            let s = string_arg(&args[0], name, 1)?.to_string();
            let pat = string_arg(&args[1], name, 2)?.to_string();
            regexp_impl("regexp", &s, &pat, false, false)
        }
        ("regexp", 3) => {
            let s = string_arg(&args[0], name, 1)?.to_string();
            let pat = string_arg(&args[1], name, 2)?.to_string();
            let opt = string_arg(&args[2], name, 3)?;
            if opt != "match" {
                return Err(format!("regexp: unknown option '{opt}'; expected 'match'"));
            }
            regexp_impl("regexp", &s, &pat, false, true)
        }
        // regexpi(s, pat) — case-insensitive regexp
        ("regexpi", 2) => {
            let s = string_arg(&args[0], name, 1)?.to_string();
            let pat = string_arg(&args[1], name, 2)?.to_string();
            regexp_impl("regexpi", &s, &pat, true, false)
        }
        ("regexpi", 3) => {
            let s = string_arg(&args[0], name, 1)?.to_string();
            let pat = string_arg(&args[1], name, 2)?.to_string();
            let opt = string_arg(&args[2], name, 3)?;
            if opt != "match" {
                return Err(format!("regexpi: unknown option '{opt}'; expected 'match'"));
            }
            regexp_impl("regexpi", &s, &pat, true, true)
        }
        // regexprep(s, pat, rep) — replace all matches with literal replacement
        ("regexprep", 3) => {
            let s = string_arg(&args[0], name, 1)?.to_string();
            let pat = string_arg(&args[1], name, 2)?.to_string();
            let rep = string_arg(&args[2], name, 3)?.to_string();
            regexprep_impl(&s, &pat, &rep)
        }
        // error(fmt, args...) — raise a runtime error with a formatted message
        ("error", _) if !args.is_empty() => {
            let fmt_str = match &args[0] {
                Value::Str(s) | Value::StringObj(s) => s.clone(),
                _ => return Err("error: first argument must be a format string".to_string()),
            };
            let msg = format_printf(&fmt_str, &args[1..])?;
            Err(msg)
        }
        // warning(fmt, args...) — print a warning to stderr, continue execution
        ("warning", _) if !args.is_empty() => {
            let fmt_str = match &args[0] {
                Value::Str(s) | Value::StringObj(s) => s.clone(),
                _ => return Err("warning: first argument must be a format string".to_string()),
            };
            let msg = format_printf(&fmt_str, &args[1..])?;
            eprintln!("warning: {msg}");
            Ok(Value::Void)
        }
        // lasterr() — return last error message; lasterr(msg) — set and return previous
        ("lasterr", 0) => Ok(Value::Str(get_last_err())),
        ("lasterr", 1) => {
            let prev = get_last_err();
            let new_msg = match &args[0] {
                Value::Str(s) | Value::StringObj(s) => s.clone(),
                _ => return Err("lasterr: argument must be a string".to_string()),
            };
            set_last_err(&new_msg);
            Ok(Value::Str(prev))
        }
        // pcall(@func, args...) — protected call; returns [ok, result_or_msg]
        ("pcall", _) if !args.is_empty() => {
            let callable = args[0].clone();
            let call_args = &args[1..];
            let result = match &callable {
                Value::Lambda(f) => {
                    let f = f.clone();
                    f.0(call_args, io)
                }
                Value::Function { .. } => match io {
                    Some(io_ref) => FN_CALL_HOOK.with(|c| match c.get() {
                        Some(hook) => hook("<pcall>", &callable, call_args, env, io_ref),
                        None => Err("pcall: function execution not initialized".to_string()),
                    }),
                    None => {
                        let mut tmp_io = IoContext::new();
                        FN_CALL_HOOK.with(|c| match c.get() {
                            Some(hook) => hook("<pcall>", &callable, call_args, env, &mut tmp_io),
                            None => Err("pcall: function execution not initialized".to_string()),
                        })
                    }
                },
                _ => {
                    return Err(
                        "pcall: first argument must be a function handle (@func)".to_string()
                    );
                }
            };
            match result {
                Ok(v) => Ok(Value::Tuple(vec![Value::Scalar(1.0), v])),
                Err(msg) => {
                    set_last_err(&msg);
                    Ok(Value::Tuple(vec![Value::Scalar(0.0), Value::Str(msg)]))
                }
            }
        }
        // ── Phase 18 — Advanced linear algebra ──────────────────────────────

        // eig(A): d = eig(A) → eigenvalue column vector; [V,D] = eig(A) → tuple.
        ("eig", 1) => match &args[0] {
            Value::Scalar(n) => {
                if get_nargout() <= 1 {
                    Ok(Value::Matrix(
                        Array2::from_shape_vec((1, 1), vec![*n]).unwrap(),
                    ))
                } else {
                    Ok(Value::Tuple(vec![
                        Value::Matrix(Array2::eye(1)),
                        Value::Matrix(Array2::from_elem((1, 1), *n)),
                    ]))
                }
            }
            Value::Matrix(m) => {
                let (evals, evecs) = eig_compute(m)?;
                let nn = evals.len();
                if get_nargout() <= 1 {
                    let col: Vec<f64> = evals;
                    Ok(Value::Matrix(Array2::from_shape_vec((nn, 1), col).unwrap()))
                } else {
                    let mut d = Array2::<f64>::zeros((nn, nn));
                    for (i, &e) in evals.iter().enumerate() {
                        d[[i, i]] = e;
                    }
                    Ok(Value::Tuple(vec![Value::Matrix(evecs), Value::Matrix(d)]))
                }
            }
            _ => Err("eig: argument must be a real numeric matrix".to_string()),
        },

        // svd(A): s = svd(A) → singular values; [U,S,V] = svd(A) → full tuple.
        // svd(A, 'econ') → economy tuple.
        ("svd", 1) => match &args[0] {
            Value::Scalar(n) => {
                let sv = n.abs();
                if get_nargout() <= 1 {
                    Ok(Value::Matrix(
                        Array2::from_shape_vec((1, 1), vec![sv]).unwrap(),
                    ))
                } else {
                    Ok(Value::Tuple(vec![
                        Value::Matrix(Array2::eye(1)),
                        Value::Matrix(Array2::from_elem((1, 1), sv)),
                        Value::Matrix(Array2::eye(1)),
                    ]))
                }
            }
            Value::Matrix(m) => {
                let mm = m.nrows();
                let nn = m.ncols();
                let (u_c, s_v, v_c) = svd_compute(m)?;
                let k = s_v.len();
                if get_nargout() <= 1 {
                    let col: Vec<f64> = s_v;
                    Ok(Value::Matrix(Array2::from_shape_vec((k, 1), col).unwrap()))
                } else {
                    // Full SVD: extend U to m×m, S to m×n.
                    let u_full = complete_orthonormal_basis(&u_c);
                    let mut s_mat = Array2::<f64>::zeros((mm, nn));
                    for (i, &sv) in s_v.iter().enumerate() {
                        s_mat[[i, i]] = sv;
                    }
                    Ok(Value::Tuple(vec![
                        Value::Matrix(u_full),
                        Value::Matrix(s_mat),
                        Value::Matrix(v_c),
                    ]))
                }
            }
            _ => Err("svd: argument must be a real numeric matrix".to_string()),
        },
        ("svd", 2) => match (&args[0], &args[1]) {
            (Value::Matrix(m), Value::Str(opt) | Value::StringObj(opt)) if opt == "econ" => {
                let (u_c, s_v, v_c) = svd_compute(m)?;
                let k = s_v.len();
                let mut s_mat = Array2::<f64>::zeros((k, k));
                for (i, &sv) in s_v.iter().enumerate() {
                    s_mat[[i, i]] = sv;
                }
                Ok(Value::Tuple(vec![
                    Value::Matrix(u_c),
                    Value::Matrix(s_mat),
                    Value::Matrix(v_c),
                ]))
            }
            _ => Err("svd: expected svd(A, 'econ')".to_string()),
        },

        // lu(A): R = lu(A) → U factor; [L,U,P] = lu(A) → full tuple (PA=LU).
        ("lu", 1) => match &args[0] {
            Value::Scalar(n) => {
                if get_nargout() <= 1 {
                    Ok(Value::Scalar(*n))
                } else {
                    Ok(Value::Tuple(vec![
                        Value::Matrix(Array2::eye(1)),
                        Value::Matrix(Array2::from_elem((1, 1), *n)),
                        Value::Matrix(Array2::eye(1)),
                    ]))
                }
            }
            Value::Matrix(m) => {
                let (l, u, p) = lu_decompose(m)?;
                if get_nargout() <= 1 {
                    Ok(Value::Matrix(u))
                } else {
                    Ok(Value::Tuple(vec![
                        Value::Matrix(l),
                        Value::Matrix(u),
                        Value::Matrix(p),
                    ]))
                }
            }
            _ => Err("lu: argument must be a real numeric matrix".to_string()),
        },

        // qr(A): R = qr(A) → R factor; [Q,R] = qr(A) → full tuple.
        ("qr", 1) => match &args[0] {
            Value::Scalar(n) => {
                if get_nargout() <= 1 {
                    Ok(Value::Scalar(*n))
                } else {
                    Ok(Value::Tuple(vec![
                        Value::Matrix(Array2::from_elem(
                            (1, 1),
                            if *n >= 0.0 { 1.0 } else { -1.0 },
                        )),
                        Value::Matrix(Array2::from_elem((1, 1), n.abs())),
                    ]))
                }
            }
            Value::Matrix(m) => {
                let (q, r) = qr_decompose(m)?;
                if get_nargout() <= 1 {
                    Ok(Value::Matrix(r))
                } else {
                    Ok(Value::Tuple(vec![Value::Matrix(q), Value::Matrix(r)]))
                }
            }
            _ => Err("qr: argument must be a real numeric matrix".to_string()),
        },

        // chol(A): always returns upper triangular R such that A = R'*R.
        ("chol", 1) => match &args[0] {
            Value::Scalar(n) => {
                if *n < 0.0 {
                    Err("chol: value is not positive definite".to_string())
                } else {
                    Ok(Value::Scalar(n.sqrt()))
                }
            }
            Value::Matrix(m) => Ok(Value::Matrix(chol_decompose(m)?)),
            _ => Err("chol: argument must be a real numeric matrix".to_string()),
        },

        // rank(A): numerical rank via SVD threshold.
        ("rank", 1) => match &args[0] {
            Value::Scalar(x) => Ok(Value::Scalar(if x.abs() > 1e-15 { 1.0 } else { 0.0 })),
            Value::Matrix(m) => {
                let (_, s_v, _) = svd_compute(m)?;
                let tol = (m.nrows().max(m.ncols())) as f64
                    * s_v.first().copied().unwrap_or(0.0)
                    * f64::EPSILON
                    * 2.0;
                let r = s_v.iter().filter(|&&s| s > tol).count();
                Ok(Value::Scalar(r as f64))
            }
            _ => Err("rank: argument must be a real numeric matrix".to_string()),
        },

        // null(A): orthonormal basis for null space of A (columns of V for ~0 singular values).
        ("null", 1) => match &args[0] {
            Value::Scalar(_) => Ok(Value::Matrix(Array2::zeros((1, 0)))),
            Value::Matrix(m) => {
                let nn = m.ncols();
                let (_, s_v, v_c) = svd_compute(m)?;
                let tol = (m.nrows().max(nn)) as f64
                    * s_v.first().copied().unwrap_or(0.0)
                    * f64::EPSILON
                    * 2.0;
                let r = s_v.iter().filter(|&&s| s > tol).count();
                let null_k = nn.saturating_sub(r);
                if null_k == 0 {
                    return Ok(Value::Matrix(Array2::zeros((nn, 0))));
                }
                let mut result = Array2::<f64>::zeros((nn, null_k));
                for j in 0..null_k {
                    let col_idx = r + j;
                    if col_idx < v_c.ncols() {
                        for i in 0..nn {
                            result[[i, j]] = v_c[[i, col_idx]];
                        }
                    }
                }
                Ok(Value::Matrix(result))
            }
            _ => Err("null: argument must be a real numeric matrix".to_string()),
        },

        // orth(A): orthonormal basis for column space of A (columns of U for nonzero singular values).
        ("orth", 1) => match &args[0] {
            Value::Scalar(x) => {
                if x.abs() > 1e-15 {
                    Ok(Value::Matrix(Array2::from_elem((1, 1), 1.0)))
                } else {
                    Ok(Value::Matrix(Array2::zeros((1, 0))))
                }
            }
            Value::Matrix(m) => {
                let mm = m.nrows();
                let (u_c, s_v, _) = svd_compute(m)?;
                let tol = (mm.max(m.ncols())) as f64
                    * s_v.first().copied().unwrap_or(0.0)
                    * f64::EPSILON
                    * 2.0;
                let r = s_v.iter().filter(|&&s| s > tol).count();
                if r == 0 {
                    return Ok(Value::Matrix(Array2::zeros((mm, 0))));
                }
                let mut result = Array2::<f64>::zeros((mm, r));
                for j in 0..r {
                    if j < u_c.ncols() {
                        for i in 0..mm {
                            result[[i, j]] = u_c[[i, j]];
                        }
                    }
                }
                Ok(Value::Matrix(result))
            }
            _ => Err("orth: argument must be a real numeric matrix".to_string()),
        },

        // cond(A): condition number = σ_max / σ_min (2-norm by default).
        ("cond", 1) => match &args[0] {
            Value::Scalar(x) => {
                if x.abs() < 1e-15 {
                    Ok(Value::Scalar(f64::INFINITY))
                } else {
                    Ok(Value::Scalar(1.0))
                }
            }
            Value::Matrix(m) => {
                let (_, s_v, _) = svd_compute(m)?;
                if s_v.is_empty() {
                    return Ok(Value::Scalar(1.0));
                }
                let s_max = s_v[0];
                let s_min = *s_v.last().unwrap();
                Ok(Value::Scalar(if s_min < 1e-15 {
                    f64::INFINITY
                } else {
                    s_max / s_min
                }))
            }
            _ => Err("cond: argument must be a real numeric matrix".to_string()),
        },

        // pinv(A): Moore-Penrose pseudoinverse via SVD.
        ("pinv", 1) => match &args[0] {
            Value::Scalar(x) => Ok(Value::Scalar(if x.abs() < 1e-15 { 0.0 } else { 1.0 / x })),
            Value::Matrix(m) => {
                let mm = m.nrows();
                let nn = m.ncols();
                let (u_c, s_v, v_c) = svd_compute(m)?;
                let k = s_v.len();
                let tol =
                    (mm.max(nn)) as f64 * s_v.first().copied().unwrap_or(0.0) * f64::EPSILON * 2.0;
                // pinv = V * diag(1/σ) * U^T
                let mut result = Array2::<f64>::zeros((nn, mm));
                for j in 0..k {
                    if s_v[j] > tol {
                        let inv_s = 1.0 / s_v[j];
                        for r in 0..nn {
                            for c in 0..mm {
                                result[[r, c]] += v_c[[r, j]] * inv_s * u_c[[c, j]];
                            }
                        }
                    }
                }
                Ok(Value::Matrix(result))
            }
            _ => Err("pinv: argument must be a real numeric matrix".to_string()),
        },

        // jsondecode(str) / jsonencode(val)
        ("jsondecode", 1) => jsondecode_impl(&args[0]),
        ("jsonencode", 1) => jsonencode_impl(&args[0]),

        // load('file.mat') — assignment form: data = load('file.mat')
        ("load", 1) => {
            let path = match &args[0] {
                Value::Str(s) | Value::StringObj(s) => s.clone(),
                _ => return Err("load: argument must be a string path".to_string()),
            };
            if !path.ends_with(".mat") {
                return Err("load: use bare 'load path' syntax for non-.mat files".to_string());
            }
            load_mat_file(&path)
        }

        // assert(cond)
        ("assert", 1) => {
            let truthy = match &args[0] {
                Value::Scalar(n) => *n != 0.0 && !n.is_nan(),
                Value::Matrix(m) => m.iter().all(|&x| x != 0.0 && !x.is_nan()),
                Value::Complex(re, im) => *re != 0.0 || *im != 0.0,
                Value::Str(s) | Value::StringObj(s) => !s.is_empty(),
                _ => false,
            };
            if truthy {
                Ok(Value::Void)
            } else {
                Err("assert: condition is false".to_string())
            }
        }

        // assert(expected, actual)
        ("assert", 2) => assert_values_equal(&args[0], &args[1], None),

        // assert(expected, actual, tol)
        ("assert", 3) => {
            let tol = match &args[2] {
                Value::Scalar(t) => *t,
                _ => return Err("assert: tolerance must be a scalar".to_string()),
            };
            assert_values_equal(&args[0], &args[1], Some(tol))
        }

        // ── datetime() constructor ────────────────────────────────────────────
        ("datetime", 1) => match &args[0] {
            Value::Str(s) | Value::StringObj(s) => {
                let s = s.as_str();
                if s == "now" {
                    return Ok(Value::DateTime(crate::datetime::now_timestamp()));
                }
                if s == "today" {
                    return Ok(Value::DateTime(crate::datetime::today_timestamp()));
                }
                crate::datetime::parse_iso8601(s).map(Value::DateTime)
            }
            _ => Err("datetime: expected a string or numeric constructor arguments".to_string()),
        },
        // datetime(ts, 'ConvertFrom', 'posixtime') — must come before the 3-scalar form
        ("datetime", 3) if matches!(&args[1], Value::Str(_) | Value::StringObj(_)) => {
            let ts = scalar_arg(&args[0], "datetime", 1)?;
            match (&args[1], &args[2]) {
                (Value::Str(k) | Value::StringObj(k), Value::Str(v) | Value::StringObj(v))
                    if k.eq_ignore_ascii_case("convertfrom")
                        && v.eq_ignore_ascii_case("posixtime") =>
                {
                    Ok(Value::DateTime(ts))
                }
                _ => Err("datetime: unsupported arguments".to_string()),
            }
        }
        ("datetime", 3) => {
            let y = scalar_arg(&args[0], "datetime", 1)? as i64;
            let mo = scalar_arg(&args[1], "datetime", 2)? as u32;
            let d = scalar_arg(&args[2], "datetime", 3)? as u32;
            Ok(Value::DateTime(crate::datetime::civil_to_timestamp(
                y, mo, d, 0, 0, 0.0,
            )))
        }
        ("datetime", 6) => {
            let y = scalar_arg(&args[0], "datetime", 1)? as i64;
            let mo = scalar_arg(&args[1], "datetime", 2)? as u32;
            let d = scalar_arg(&args[2], "datetime", 3)? as u32;
            let h = scalar_arg(&args[3], "datetime", 4)? as u32;
            let mi = scalar_arg(&args[4], "datetime", 5)? as u32;
            let s = scalar_arg(&args[5], "datetime", 6)?;
            Ok(Value::DateTime(crate::datetime::civil_to_timestamp(
                y, mo, d, h, mi, s,
            )))
        }

        // ── Component extractors ──────────────────────────────────────────────
        ("year", 1) => match &args[0] {
            Value::DateTime(ts) => {
                let (y, ..) = crate::datetime::timestamp_to_civil(*ts);
                Ok(Value::Scalar(y as f64))
            }
            Value::DateTimeArray(v) => {
                let rows: Vec<f64> = v
                    .iter()
                    .map(|ts| {
                        let (y, ..) = crate::datetime::timestamp_to_civil(*ts);
                        y as f64
                    })
                    .collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => Err("year: argument must be a datetime".to_string()),
        },
        ("month", 1) => match &args[0] {
            Value::DateTime(ts) => {
                let (_, mo, ..) = crate::datetime::timestamp_to_civil(*ts);
                Ok(Value::Scalar(mo as f64))
            }
            Value::DateTimeArray(v) => {
                let rows: Vec<f64> = v
                    .iter()
                    .map(|ts| {
                        let (_, mo, ..) = crate::datetime::timestamp_to_civil(*ts);
                        mo as f64
                    })
                    .collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => Err("month: argument must be a datetime".to_string()),
        },
        ("day", 1) => match &args[0] {
            Value::DateTime(ts) => {
                let (_, _, d, ..) = crate::datetime::timestamp_to_civil(*ts);
                Ok(Value::Scalar(d as f64))
            }
            Value::DateTimeArray(v) => {
                let rows: Vec<f64> = v
                    .iter()
                    .map(|ts| {
                        let (_, _, d, ..) = crate::datetime::timestamp_to_civil(*ts);
                        d as f64
                    })
                    .collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => Err("day: argument must be a datetime".to_string()),
        },
        ("hour", 1) => match &args[0] {
            Value::DateTime(ts) => {
                let (_, _, _, h, ..) = crate::datetime::timestamp_to_civil(*ts);
                Ok(Value::Scalar(h as f64))
            }
            Value::DateTimeArray(v) => {
                let rows: Vec<f64> = v
                    .iter()
                    .map(|ts| {
                        let (_, _, _, h, ..) = crate::datetime::timestamp_to_civil(*ts);
                        h as f64
                    })
                    .collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => Err("hour: argument must be a datetime or duration".to_string()),
        },
        ("minute", 1) => match &args[0] {
            Value::DateTime(ts) => {
                let (_, _, _, _, mi, ..) = crate::datetime::timestamp_to_civil(*ts);
                Ok(Value::Scalar(mi as f64))
            }
            Value::DateTimeArray(v) => {
                let rows: Vec<f64> = v
                    .iter()
                    .map(|ts| {
                        let (_, _, _, _, mi, ..) = crate::datetime::timestamp_to_civil(*ts);
                        mi as f64
                    })
                    .collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => Err("minute: argument must be a datetime or duration".to_string()),
        },
        ("second", 1) => match &args[0] {
            Value::DateTime(ts) => {
                let (_, _, _, _, _, s) = crate::datetime::timestamp_to_civil(*ts);
                Ok(Value::Scalar(s))
            }
            Value::DateTimeArray(v) => {
                let rows: Vec<f64> = v
                    .iter()
                    .map(|ts| {
                        let (_, _, _, _, _, s) = crate::datetime::timestamp_to_civil(*ts);
                        s
                    })
                    .collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => Err("second: argument must be a datetime or duration".to_string()),
        },

        // ── Predicates ────────────────────────────────────────────────────────
        ("isdatetime", 1) => Ok(Value::Scalar(bool_to_f64(matches!(
            &args[0],
            Value::DateTime(_) | Value::DateTimeArray(_)
        )))),
        ("isduration", 1) => Ok(Value::Scalar(bool_to_f64(matches!(
            &args[0],
            Value::Duration(_) | Value::DurationArray(_)
        )))),
        ("isnat", 1) => match &args[0] {
            Value::DateTime(ts) => Ok(Value::Scalar(bool_to_f64(ts.is_nan()))),
            Value::DateTimeArray(v) => {
                let rows: Vec<f64> = v
                    .iter()
                    .map(|ts| if ts.is_nan() { 1.0 } else { 0.0 })
                    .collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => Ok(Value::Scalar(0.0)),
        },

        // ── Duration constructors / extractors (overloaded) ───────────────────
        ("hours", 1) => match &args[0] {
            Value::Duration(s) => Ok(Value::Scalar(*s / 3600.0)),
            Value::DurationArray(v) => {
                let rows: Vec<f64> = v.iter().map(|s| s / 3600.0).collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => {
                let s = scalar_arg(&args[0], "hours", 1)?;
                Ok(Value::Duration(s * 3600.0))
            }
        },
        ("minutes", 1) => match &args[0] {
            Value::Duration(s) => Ok(Value::Scalar(*s / 60.0)),
            Value::DurationArray(v) => {
                let rows: Vec<f64> = v.iter().map(|s| s / 60.0).collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => {
                let s = scalar_arg(&args[0], "minutes", 1)?;
                Ok(Value::Duration(s * 60.0))
            }
        },
        ("seconds", 1) => match &args[0] {
            Value::Duration(s) => Ok(Value::Scalar(*s)),
            Value::DurationArray(v) => {
                let rows = v.to_vec();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => {
                let s = scalar_arg(&args[0], "seconds", 1)?;
                Ok(Value::Duration(s))
            }
        },
        ("days", 1) => match &args[0] {
            Value::Duration(s) => Ok(Value::Scalar(*s / 86400.0)),
            Value::DurationArray(v) => {
                let rows: Vec<f64> = v.iter().map(|s| s / 86400.0).collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => {
                let s = scalar_arg(&args[0], "days", 1)?;
                Ok(Value::Duration(s * 86400.0))
            }
        },
        ("milliseconds", 1) => match &args[0] {
            Value::Duration(s) => Ok(Value::Scalar(*s * 1000.0)),
            Value::DurationArray(v) => {
                let rows: Vec<f64> = v.iter().map(|s| s * 1000.0).collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => {
                let s = scalar_arg(&args[0], "milliseconds", 1)?;
                Ok(Value::Duration(s / 1000.0))
            }
        },
        ("years", 1) => match &args[0] {
            Value::Duration(s) => Ok(Value::Scalar(*s / (365.2425 * 86400.0))),
            Value::DurationArray(v) => {
                let rows: Vec<f64> = v.iter().map(|s| s / (365.2425 * 86400.0)).collect();
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((rows.len(), 1), rows)
                        .map_err(|e| e.to_string())?,
                ))
            }
            _ => {
                let s = scalar_arg(&args[0], "years", 1)?;
                Ok(Value::Duration(s * 365.2425 * 86400.0))
            }
        },
        // duration(H, M, S)
        ("duration", 3) => {
            let h = scalar_arg(&args[0], "duration", 1)?;
            let m = scalar_arg(&args[1], "duration", 2)?;
            let s = scalar_arg(&args[2], "duration", 3)?;
            Ok(Value::Duration(h * 3600.0 + m * 60.0 + s))
        }

        // ── Formatting and conversion ─────────────────────────────────────────
        ("datestr", 1) => match &args[0] {
            Value::DateTime(ts) => {
                let s = crate::datetime::format_datestr(*ts, "dd-MMM-yyyy HH:mm:ss");
                Ok(Value::Str(s))
            }
            Value::DateTimeArray(v) => Ok(Value::Cell(
                v.iter()
                    .map(|ts| {
                        Value::Str(crate::datetime::format_datestr(*ts, "dd-MMM-yyyy HH:mm:ss"))
                    })
                    .collect(),
            )),
            _ => Err("datestr: argument must be a datetime".to_string()),
        },
        ("datestr", 2) => {
            let fmt_str = match &args[1] {
                Value::Str(s) | Value::StringObj(s) => s.clone(),
                _ => return Err("datestr: second argument must be a format string".to_string()),
            };
            match &args[0] {
                Value::DateTime(ts) => {
                    Ok(Value::Str(crate::datetime::format_datestr(*ts, &fmt_str)))
                }
                Value::DateTimeArray(v) => Ok(Value::Cell(
                    v.iter()
                        .map(|ts| Value::Str(crate::datetime::format_datestr(*ts, &fmt_str)))
                        .collect(),
                )),
                _ => Err("datestr: first argument must be a datetime".to_string()),
            }
        }
        ("datevec", 1) => match &args[0] {
            Value::DateTime(ts) => {
                let (y, mo, d, h, mi, s) = crate::datetime::timestamp_to_civil(*ts);
                let sec_i = s.floor() as u32;
                let data = vec![
                    y as f64,
                    mo as f64,
                    d as f64,
                    h as f64,
                    mi as f64,
                    sec_i as f64,
                ];
                Ok(Value::Matrix(
                    ndarray::Array2::from_shape_vec((1, 6), data).map_err(|e| e.to_string())?,
                ))
            }
            _ => Err("datevec: argument must be a datetime".to_string()),
        },
        ("datenum", 1) => match &args[0] {
            Value::DateTime(ts) => Ok(Value::Scalar(crate::datetime::to_datenum(*ts))),
            _ => Err("datenum: argument must be a datetime".to_string()),
        },
        ("datenum", 3) => {
            let y = scalar_arg(&args[0], "datenum", 1)? as i64;
            let mo = scalar_arg(&args[1], "datenum", 2)? as u32;
            let d = scalar_arg(&args[2], "datenum", 3)? as u32;
            let ts = crate::datetime::civil_to_timestamp(y, mo, d, 0, 0, 0.0);
            Ok(Value::Scalar(crate::datetime::to_datenum(ts)))
        }
        ("posixtime", 1) => match &args[0] {
            Value::DateTime(ts) => Ok(Value::Scalar(*ts)),
            _ => Err("posixtime: argument must be a datetime".to_string()),
        },

        // ── diff for datetime/duration arrays ─────────────────────────────────
        ("diff", 1) => match &args[0] {
            Value::DateTimeArray(v) if v.len() >= 2 => {
                let diffs: Vec<f64> = v.windows(2).map(|w| w[1] - w[0]).collect();
                Ok(Value::DurationArray(diffs))
            }
            Value::DurationArray(v) if v.len() >= 2 => {
                let diffs: Vec<f64> = v.windows(2).map(|w| w[1] - w[0]).collect();
                Ok(Value::DurationArray(diffs))
            }
            Value::Matrix(m) => {
                // diff on numeric matrix: successive differences along first non-singleton dim
                let (nrows, ncols) = (m.nrows(), m.ncols());
                if ncols > 1 && nrows == 1 {
                    // Row vector → diff along columns
                    let data: Vec<f64> =
                        (0..ncols - 1).map(|j| m[[0, j + 1]] - m[[0, j]]).collect();
                    Ok(Value::Matrix(
                        ndarray::Array2::from_shape_vec((1, data.len()), data)
                            .map_err(|e| e.to_string())?,
                    ))
                } else if nrows > 1 {
                    // Column vector or matrix → diff along rows
                    let data: Vec<f64> = (0..nrows - 1)
                        .flat_map(|i| (0..ncols).map(move |j| m[[i + 1, j]] - m[[i, j]]))
                        .collect();
                    Ok(Value::Matrix(
                        ndarray::Array2::from_shape_vec((nrows - 1, ncols), data)
                            .map_err(|e| e.to_string())?,
                    ))
                } else {
                    Err("diff: input must have at least 2 elements".to_string())
                }
            }
            _ => Err("diff: unsupported argument type".to_string()),
        },

        // ── Phase 23a — Matrix shape utilities ───────────────────────────────
        ("triu", 1) => match &args[0] {
            Value::Matrix(m) => {
                let mut r = m.clone();
                for i in 0..m.nrows() {
                    for j in 0..m.ncols() {
                        if (j as isize) < (i as isize) {
                            r[[i, j]] = 0.0;
                        }
                    }
                }
                Ok(Value::Matrix(r))
            }
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            _ => Err("triu: argument must be a numeric matrix".to_string()),
        },
        ("triu", 2) => match (&args[0], &args[1]) {
            (Value::Matrix(m), Value::Scalar(k)) => {
                let k = *k as isize;
                let mut r = m.clone();
                for i in 0..m.nrows() {
                    for j in 0..m.ncols() {
                        if (j as isize) - (i as isize) < k {
                            r[[i, j]] = 0.0;
                        }
                    }
                }
                Ok(Value::Matrix(r))
            }
            _ => Err("triu: expects (matrix, scalar)".to_string()),
        },

        ("tril", 1) => match &args[0] {
            Value::Matrix(m) => {
                let mut r = m.clone();
                for i in 0..m.nrows() {
                    for j in 0..m.ncols() {
                        if (j as isize) > (i as isize) {
                            r[[i, j]] = 0.0;
                        }
                    }
                }
                Ok(Value::Matrix(r))
            }
            Value::Scalar(n) => Ok(Value::Scalar(*n)),
            _ => Err("tril: argument must be a numeric matrix".to_string()),
        },
        ("tril", 2) => match (&args[0], &args[1]) {
            (Value::Matrix(m), Value::Scalar(k)) => {
                let k = *k as isize;
                let mut r = m.clone();
                for i in 0..m.nrows() {
                    for j in 0..m.ncols() {
                        if (j as isize) - (i as isize) > k {
                            r[[i, j]] = 0.0;
                        }
                    }
                }
                Ok(Value::Matrix(r))
            }
            _ => Err("tril: expects (matrix, scalar)".to_string()),
        },

        ("repmat", 3) => match (&args[0], &args[1], &args[2]) {
            (Value::Matrix(a), Value::Scalar(rm), Value::Scalar(cn)) => {
                let rm = *rm as usize;
                let cn = *cn as usize;
                if rm == 0 || cn == 0 {
                    return Ok(Value::Matrix(Array2::zeros((0, 0))));
                }
                let row_tile: Vec<Array2<f64>> = std::iter::repeat_n(a.view(), cn)
                    .map(|v| v.to_owned())
                    .collect();
                let row_block = ndarray::concatenate(
                    ndarray::Axis(1),
                    &row_tile.iter().map(|m| m.view()).collect::<Vec<_>>(),
                )
                .map_err(|e| e.to_string())?;
                let col_tiles: Vec<Array2<f64>> = std::iter::repeat_n(row_block.view(), rm)
                    .map(|v| v.to_owned())
                    .collect();
                let result = ndarray::concatenate(
                    ndarray::Axis(0),
                    &col_tiles.iter().map(|m| m.view()).collect::<Vec<_>>(),
                )
                .map_err(|e| e.to_string())?;
                Ok(Value::Matrix(result))
            }
            (Value::Scalar(s), Value::Scalar(rm), Value::Scalar(cn)) => {
                let rm = *rm as usize;
                let cn = *cn as usize;
                Ok(Value::Matrix(Array2::from_elem((rm, cn), *s)))
            }
            _ => Err("repmat: expects (matrix, m, n)".to_string()),
        },

        ("kron", 2) => match (&args[0], &args[1]) {
            (Value::Matrix(a), Value::Matrix(b)) => {
                let (ra, ca) = (a.nrows(), a.ncols());
                let (rb, cb) = (b.nrows(), b.ncols());
                let mut result = Array2::<f64>::zeros((ra * rb, ca * cb));
                for i in 0..ra {
                    for j in 0..ca {
                        let aij = a[[i, j]];
                        for p in 0..rb {
                            for q in 0..cb {
                                result[[i * rb + p, j * cb + q]] = aij * b[[p, q]];
                            }
                        }
                    }
                }
                Ok(Value::Matrix(result))
            }
            (Value::Scalar(s), Value::Matrix(b)) => Ok(Value::Matrix(b.mapv(|x| x * s))),
            (Value::Matrix(a), Value::Scalar(s)) => Ok(Value::Matrix(a.mapv(|x| x * s))),
            (Value::Scalar(a), Value::Scalar(b)) => Ok(Value::Scalar(a * b)),
            _ => Err("kron: arguments must be numeric matrices".to_string()),
        },

        // ── Phase 23b — Vector products ──────────────────────────────────────
        ("cross", 2) => {
            fn to_vec3(v: &Value, argn: usize) -> Result<[f64; 3], String> {
                match v {
                    Value::Matrix(m) => {
                        let flat: Vec<f64> = m.iter().copied().collect();
                        if flat.len() != 3 {
                            Err(format!(
                                "cross: argument {} must have exactly 3 elements",
                                argn
                            ))
                        } else {
                            Ok([flat[0], flat[1], flat[2]])
                        }
                    }
                    _ => Err(format!(
                        "cross: argument {} must be a 3-element vector",
                        argn
                    )),
                }
            }
            let a = to_vec3(&args[0], 1)?;
            let b = to_vec3(&args[1], 2)?;
            let cx = a[1] * b[2] - a[2] * b[1];
            let cy = a[2] * b[0] - a[0] * b[2];
            let cz = a[0] * b[1] - a[1] * b[0];
            // Result orientation follows first argument
            let result = match &args[0] {
                Value::Matrix(m) if m.nrows() == 1 => {
                    Array2::from_shape_vec((1, 3), vec![cx, cy, cz]).unwrap()
                }
                _ => Array2::from_shape_vec((3, 1), vec![cx, cy, cz]).unwrap(),
            };
            Ok(Value::Matrix(result))
        }

        ("dot", 2) => {
            fn to_flat(v: &Value, argn: usize) -> Result<Vec<f64>, String> {
                match v {
                    Value::Matrix(m) => Ok(m.iter().copied().collect()),
                    Value::Scalar(s) => Ok(vec![*s]),
                    _ => Err(format!("dot: argument {} must be a numeric vector", argn)),
                }
            }
            let a = to_flat(&args[0], 1)?;
            let b = to_flat(&args[1], 2)?;
            if a.len() != b.len() {
                return Err(format!(
                    "dot: vectors must have the same length ({} vs {})",
                    a.len(),
                    b.len()
                ));
            }
            let s: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            Ok(Value::Scalar(s))
        }

        // ── Phase 23c — Set operations ────────────────────────────────────────
        ("intersect", 2) => {
            fn to_sorted_vec(v: &Value, fname: &str) -> Result<Vec<f64>, String> {
                match v {
                    Value::Matrix(m) => {
                        let mut vals: Vec<f64> = m.iter().copied().collect();
                        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        Ok(vals)
                    }
                    Value::Scalar(s) => Ok(vec![*s]),
                    _ => Err(format!("{fname}: arguments must be numeric vectors")),
                }
            }
            let a = to_sorted_vec(&args[0], "intersect")?;
            let b = to_sorted_vec(&args[1], "intersect")?;
            let b_set: std::collections::HashSet<u64> = b
                .iter()
                .filter(|x| !x.is_nan())
                .map(|x| x.to_bits())
                .collect();
            let mut result: Vec<f64> = Vec::new();
            for x in &a {
                if !x.is_nan()
                    && b_set.contains(&x.to_bits())
                    && result.last().is_none_or(|&last| last != *x)
                {
                    result.push(*x);
                }
            }
            let n = result.len();
            if n == 0 {
                Ok(Value::Matrix(Array2::zeros((1, 0))))
            } else {
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, n), result).unwrap(),
                ))
            }
        }

        ("union", 2) => {
            fn collect_vals(v: &Value, fname: &str) -> Result<Vec<f64>, String> {
                match v {
                    Value::Matrix(m) => Ok(m.iter().copied().collect()),
                    Value::Scalar(s) => Ok(vec![*s]),
                    _ => Err(format!("{fname}: arguments must be numeric vectors")),
                }
            }
            let mut combined = collect_vals(&args[0], "union")?;
            combined.extend(collect_vals(&args[1], "union")?);
            combined.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mut result: Vec<f64> = Vec::new();
            for x in combined {
                if result.last().is_none_or(|&last| last != x) {
                    result.push(x);
                }
            }
            let n = result.len();
            if n == 0 {
                Ok(Value::Matrix(Array2::zeros((1, 0))))
            } else {
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, n), result).unwrap(),
                ))
            }
        }

        ("setdiff", 2) => {
            fn collect_vals2(v: &Value, fname: &str) -> Result<Vec<f64>, String> {
                match v {
                    Value::Matrix(m) => Ok(m.iter().copied().collect()),
                    Value::Scalar(s) => Ok(vec![*s]),
                    _ => Err(format!("{fname}: arguments must be numeric vectors")),
                }
            }
            let a = collect_vals2(&args[0], "setdiff")?;
            let b = collect_vals2(&args[1], "setdiff")?;
            let b_set: std::collections::HashSet<u64> = b
                .iter()
                .filter(|x| !x.is_nan())
                .map(|x| x.to_bits())
                .collect();
            let mut a_sorted = a.clone();
            a_sorted.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
            let mut result: Vec<f64> = Vec::new();
            for x in a_sorted {
                if !x.is_nan()
                    && !b_set.contains(&x.to_bits())
                    && result.last().is_none_or(|&last| last != x)
                {
                    result.push(x);
                }
            }
            let n = result.len();
            if n == 0 {
                Ok(Value::Matrix(Array2::zeros((1, 0))))
            } else {
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, n), result).unwrap(),
                ))
            }
        }

        ("ismember", 2) => {
            fn collect_vals3(v: &Value, fname: &str) -> Result<Vec<f64>, String> {
                match v {
                    Value::Matrix(m) => Ok(m.iter().copied().collect()),
                    Value::Scalar(s) => Ok(vec![*s]),
                    _ => Err(format!("{fname}: arguments must be numeric")),
                }
            }
            let set: std::collections::HashSet<u64> = collect_vals3(&args[1], "ismember")?
                .into_iter()
                .filter(|x| !x.is_nan())
                .map(|x| x.to_bits())
                .collect();
            match &args[0] {
                Value::Scalar(s) => {
                    let found = !s.is_nan() && set.contains(&s.to_bits());
                    Ok(Value::Scalar(if found { 1.0 } else { 0.0 }))
                }
                Value::Matrix(m) => {
                    let result: Vec<f64> = m
                        .iter()
                        .map(|x| {
                            if !x.is_nan() && set.contains(&x.to_bits()) {
                                1.0
                            } else {
                                0.0
                            }
                        })
                        .collect();
                    let shape = m.raw_dim();
                    Ok(Value::Matrix(
                        Array2::from_shape_vec(shape, result).unwrap(),
                    ))
                }
                _ => Err("ismember: first argument must be numeric".to_string()),
            }
        }

        // ── Phase 23d — Index utilities and element repetition ────────────────
        ("sub2ind", 3) => {
            let sz = match &args[0] {
                Value::Matrix(m) if m.len() == 2 => (m[[0, 0]] as usize, m[[0, 1]] as usize),
                _ => return Err("sub2ind: first argument must be [rows cols]".to_string()),
            };
            let rows = sz.0;
            fn idx_vals(v: &Value, argn: usize) -> Result<Vec<f64>, String> {
                match v {
                    Value::Scalar(s) => Ok(vec![*s]),
                    Value::Matrix(m) => Ok(m.iter().copied().collect()),
                    _ => Err(format!("sub2ind: argument {} must be numeric", argn)),
                }
            }
            let r = idx_vals(&args[1], 2)?;
            let c = idx_vals(&args[2], 3)?;
            if r.len() != c.len() {
                return Err(
                    "sub2ind: row and column index vectors must have the same length".to_string(),
                );
            }
            if r.len() == 1 {
                let idx = (c[0] as usize - 1) * rows + r[0] as usize;
                Ok(Value::Scalar(idx as f64))
            } else {
                let vals: Vec<f64> = r
                    .iter()
                    .zip(c.iter())
                    .map(|(&ri, &ci)| ((ci as usize - 1) * rows + ri as usize) as f64)
                    .collect();
                let n = vals.len();
                Ok(Value::Matrix(Array2::from_shape_vec((1, n), vals).unwrap()))
            }
        }

        ("ind2sub", 2) => {
            let sz = match &args[0] {
                Value::Matrix(m) if m.len() == 2 => (m[[0, 0]] as usize, m[[0, 1]] as usize),
                _ => return Err("ind2sub: first argument must be [rows cols]".to_string()),
            };
            let rows = sz.0;
            fn idx_vals2(v: &Value, argn: usize) -> Result<Vec<f64>, String> {
                match v {
                    Value::Scalar(s) => Ok(vec![*s]),
                    Value::Matrix(m) => Ok(m.iter().copied().collect()),
                    _ => Err(format!("ind2sub: argument {} must be numeric", argn)),
                }
            }
            let indices = idx_vals2(&args[1], 2)?;
            if indices.len() == 1 {
                let idx = indices[0] as usize;
                let r = ((idx - 1) % rows + 1) as f64;
                let c = ((idx - 1) / rows + 1) as f64;
                Ok(Value::Tuple(vec![Value::Scalar(r), Value::Scalar(c)]))
            } else {
                let n = indices.len();
                let rs: Vec<f64> = indices
                    .iter()
                    .map(|&idx| ((idx as usize - 1) % rows + 1) as f64)
                    .collect();
                let cs: Vec<f64> = indices
                    .iter()
                    .map(|&idx| ((idx as usize - 1) / rows + 1) as f64)
                    .collect();
                let rm = Value::Matrix(Array2::from_shape_vec((1, n), rs).unwrap());
                let cm = Value::Matrix(Array2::from_shape_vec((1, n), cs).unwrap());
                Ok(Value::Tuple(vec![rm, cm]))
            }
        }

        ("repelem", 2) => match (&args[0], &args[1]) {
            (Value::Matrix(a), Value::Scalar(n)) => {
                let n = *n as usize;
                let flat: Vec<f64> = a.iter().flat_map(|&x| std::iter::repeat_n(x, n)).collect();
                let total = flat.len();
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, total), flat).unwrap(),
                ))
            }
            (Value::Matrix(a), Value::Matrix(ns)) => {
                let av: Vec<f64> = a.iter().copied().collect();
                let nv: Vec<f64> = ns.iter().copied().collect();
                if av.len() != nv.len() {
                    return Err(
                        "repelem: element count vector must match source vector length".to_string(),
                    );
                }
                let flat: Vec<f64> = av
                    .iter()
                    .zip(nv.iter())
                    .flat_map(|(&x, &n)| std::iter::repeat_n(x, n as usize))
                    .collect();
                let total = flat.len();
                Ok(Value::Matrix(
                    Array2::from_shape_vec((1, total), flat).unwrap(),
                ))
            }
            (Value::Scalar(s), Value::Scalar(n)) => {
                let n = *n as usize;
                Ok(Value::Matrix(Array2::from_elem((1, n), *s)))
            }
            _ => Err("repelem: unsupported argument types".to_string()),
        },
        ("repelem", 3) => match (&args[0], &args[1], &args[2]) {
            (Value::Matrix(a), Value::Scalar(rm), Value::Scalar(cn)) => {
                let rm = *rm as usize;
                let cn = *cn as usize;
                let (nrows, ncols) = (a.nrows(), a.ncols());
                let mut result = Array2::<f64>::zeros((nrows * rm, ncols * cn));
                for i in 0..nrows {
                    for j in 0..ncols {
                        let v = a[[i, j]];
                        for di in 0..rm {
                            for dj in 0..cn {
                                result[[i * rm + di, j * cn + dj]] = v;
                            }
                        }
                    }
                }
                Ok(Value::Matrix(result))
            }
            (Value::Scalar(s), Value::Scalar(rm), Value::Scalar(cn)) => Ok(Value::Matrix(
                Array2::from_elem((*rm as usize, *cn as usize), *s),
            )),
            _ => Err("repelem: expects (matrix, m, n) for 2D repetition".to_string()),
        },

        // ── Phase 24a — Polynomial evaluation, fitting, and roots ────────────
        ("polyval", 2) => {
            let coeffs = poly_coeffs(&args[0], "polyval")?;
            if coeffs.is_empty() {
                return Err("polyval: polynomial vector is empty".to_string());
            }
            match &args[1] {
                Value::Scalar(x) => Ok(Value::Scalar(horner(&coeffs, *x))),
                Value::Matrix(m) => Ok(Value::Matrix(m.mapv(|x| horner(&coeffs, x)))),
                _ => Err("polyval: second argument must be a real numeric value".to_string()),
            }
        }

        ("polyfit", 3) => {
            let xv = poly_coeffs(&args[0], "polyfit")?;
            let yv = poly_coeffs(&args[1], "polyfit")?;
            let deg = match &args[2] {
                Value::Scalar(n) => {
                    let d = *n as usize;
                    if *n < 0.0 || (*n - d as f64).abs() > 1e-9 {
                        return Err("polyfit: degree must be a non-negative integer".to_string());
                    }
                    d
                }
                _ => return Err("polyfit: degree must be a scalar".to_string()),
            };
            if xv.len() != yv.len() {
                return Err("polyfit: x and y must have the same length".to_string());
            }
            let m = xv.len();
            let ncols = deg + 1;
            if ncols > m {
                return Err(format!(
                    "polyfit: not enough data points ({m}) for degree-{deg} fit"
                ));
            }
            // Build Vandermonde matrix (m × ncols), highest power first
            let mut vander = Array2::<f64>::zeros((m, ncols));
            for (i, &xi) in xv.iter().enumerate() {
                for j in 0..ncols {
                    vander[[i, j]] = xi.powi((deg - j) as i32);
                }
            }
            // Solve via QR: V*c = y  →  Q*R*c = y  →  R*c = Q^T*y
            let (q, r) = qr_decompose(&vander)?;
            let qty: Vec<f64> = (0..ncols)
                .map(|i| (0..m).map(|k| q[[k, i]] * yv[k]).sum())
                .collect();
            // Extract upper-left ncols×ncols block of R
            let mut r_sq = Array2::<f64>::zeros((ncols, ncols));
            for i in 0..ncols {
                for j in 0..ncols {
                    r_sq[[i, j]] = r[[i, j]];
                }
            }
            let coeffs = poly_back_sub(&r_sq, &qty)?;
            let result = Array2::from_shape_vec((1, ncols), coeffs)
                .map_err(|e| format!("polyfit: internal error: {e}"))?;
            Ok(Value::Matrix(result))
        }

        ("roots", 1) => {
            let raw = poly_coeffs(&args[0], "roots")?;
            // Strip leading zeros
            let start = raw.iter().position(|&c| c != 0.0).unwrap_or(raw.len());
            let coeffs = &raw[start..];
            if coeffs.len() <= 1 {
                return Ok(Value::Matrix(Array2::zeros((0, 1))));
            }
            let roots = durand_kerner(coeffs)?;
            Ok(roots_to_value(&roots))
        }

        ("poly", 1) => match &args[0] {
            Value::Scalar(r) => {
                let data = vec![1.0, -*r];
                Ok(Value::Matrix(Array2::from_shape_vec((1, 2), data).unwrap()))
            }
            Value::Matrix(m) => {
                if m.nrows() == 1 || m.ncols() == 1 {
                    // Root vector: expand (x − r_1)(x − r_2)…
                    let roots: Vec<f64> = if m.nrows() == 1 {
                        m.row(0).iter().copied().collect()
                    } else {
                        m.column(0).iter().copied().collect()
                    };
                    let mut p = vec![1.0_f64];
                    for &r in &roots {
                        p = poly_conv(&p, &[1.0, -r]);
                    }
                    let ncols = p.len();
                    Ok(Value::Matrix(
                        Array2::from_shape_vec((1, ncols), p).unwrap(),
                    ))
                } else {
                    // Square matrix: characteristic polynomial via Faddeev-LeVerrier
                    let coeffs = characteristic_poly(m)?;
                    let ncols = coeffs.len();
                    Ok(Value::Matrix(
                        Array2::from_shape_vec((1, ncols), coeffs).unwrap(),
                    ))
                }
            }
            _ => Err("poly: argument must be a numeric vector or square matrix".to_string()),
        },

        // ── Phase 24b — Convolution, deconvolution, interpolation ────────────
        ("conv", 2) => {
            let a = poly_coeffs(&args[0], "conv")?;
            let b = poly_coeffs(&args[1], "conv")?;
            if a.is_empty() || b.is_empty() {
                return Ok(Value::Matrix(Array2::zeros((1, 0))));
            }
            let c = poly_conv(&a, &b);
            let len = c.len();
            Ok(Value::Matrix(Array2::from_shape_vec((1, len), c).unwrap()))
        }

        ("deconv", 2) => {
            let c = poly_coeffs(&args[0], "deconv")?;
            let b = poly_coeffs(&args[1], "deconv")?;
            let (q, r) = poly_deconv(&c, &b)?;
            let qn = q.len();
            let rn = r.len();
            let q_val = Value::Matrix(Array2::from_shape_vec((1, qn), q).unwrap());
            let r_val = Value::Matrix(Array2::from_shape_vec((1, rn), r).unwrap());
            Ok(Value::Tuple(vec![q_val, r_val]))
        }

        ("interp1", 3) => {
            let xv = poly_coeffs(&args[0], "interp1")?;
            let yv = poly_coeffs(&args[1], "interp1")?;
            if xv.len() != yv.len() {
                return Err("interp1: x and y must have the same length".to_string());
            }
            if xv.len() < 2 {
                return Err("interp1: requires at least two knot points".to_string());
            }
            match &args[2] {
                Value::Scalar(xi) => Ok(Value::Scalar(interp1_at(&xv, &yv, *xi, "linear"))),
                Value::Matrix(xi_m) => Ok(Value::Matrix(
                    xi_m.mapv(|xi| interp1_at(&xv, &yv, xi, "linear")),
                )),
                _ => Err("interp1: query points must be numeric".to_string()),
            }
        }

        ("interp1", 4) => {
            let xv = poly_coeffs(&args[0], "interp1")?;
            let yv = poly_coeffs(&args[1], "interp1")?;
            let method = match &args[3] {
                Value::Str(s) | Value::StringObj(s) => s.clone(),
                _ => return Err("interp1: method argument must be a string".to_string()),
            };
            if !matches!(method.as_str(), "linear" | "nearest" | "previous" | "next") {
                return Err(format!(
                    "interp1: unknown method '{method}'; supported: linear nearest previous next"
                ));
            }
            if xv.len() != yv.len() {
                return Err("interp1: x and y must have the same length".to_string());
            }
            if xv.len() < 2 {
                return Err("interp1: requires at least two knot points".to_string());
            }
            match &args[2] {
                Value::Scalar(xi) => Ok(Value::Scalar(interp1_at(&xv, &yv, *xi, &method))),
                Value::Matrix(xi_m) => {
                    let m_str = method.as_str();
                    Ok(Value::Matrix(
                        xi_m.mapv(|xi| interp1_at(&xv, &yv, xi, m_str)),
                    ))
                }
                _ => Err("interp1: query points must be numeric".to_string()),
            }
        }

        _ => {
            let hint = suggest_similar(name, env);
            match hint {
                Some(s) => Err(format!("Unknown function '{name}'; did you mean '{s}'?")),
                None => Err(format!("Unknown function: '{name}'")),
            }
        }
    }
}

/// Interprets backslash escape sequences in delimiter strings.
/// `\t` → tab, `\n` → newline. Other strings are used as-is.
fn interpret_delim(s: &str) -> String {
    match s {
        r"\t" => "\t".to_string(),
        r"\n" => "\n".to_string(),
        other => other.to_string(),
    }
}

/// Returns true if splitting every line by `delim` gives the same field count > 1.
fn delim_consistent(lines: &[&str], delim: char) -> bool {
    let counts: Vec<usize> = lines.iter().map(|l| l.split(delim).count()).collect();
    counts.iter().all(|&c| c > 1) && counts.windows(2).all(|w| w[0] == w[1])
}

/// Reads a delimiter-separated numeric file and returns a `Value::Matrix`.
fn dlmread_impl(path: &str, explicit_delim: Option<String>) -> Result<Value, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("dlmread: cannot read '{path}': {e}"))?;

    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();

    if lines.is_empty() {
        return Ok(Value::Matrix(Array2::zeros((0, 0))));
    }

    // Determine delimiter: explicit → auto-detect (comma → tab → whitespace)
    let delim: Option<String> = match explicit_delim {
        Some(d) => Some(d),
        None => {
            if delim_consistent(&lines, ',') {
                Some(",".to_string())
            } else if delim_consistent(&lines, '\t') {
                Some("\t".to_string())
            } else {
                None // split by whitespace
            }
        }
    };

    let mut rows: Vec<Vec<f64>> = Vec::new();
    for (line_num, line) in lines.iter().enumerate() {
        let fields: Vec<&str> = match &delim {
            Some(d) => line.split(d.as_str()).collect(),
            None => line.split_whitespace().collect(),
        };
        let mut row_vals: Vec<f64> = Vec::with_capacity(fields.len());
        for field in &fields {
            let trimmed = field.trim();
            if trimmed.is_empty() {
                row_vals.push(0.0);
            } else {
                row_vals.push(trimmed.parse::<f64>().map_err(|_| {
                    format!(
                        "dlmread: non-numeric value '{trimmed}' on line {}",
                        line_num + 1
                    )
                })?);
            }
        }
        if !row_vals.is_empty() {
            rows.push(row_vals);
        }
    }

    if rows.is_empty() {
        return Ok(Value::Matrix(Array2::zeros((0, 0))));
    }

    let ncols = rows[0].len();
    for (i, row) in rows.iter().enumerate() {
        if row.len() != ncols {
            return Err(format!(
                "dlmread: row {} has {} fields, expected {ncols}",
                i + 1,
                row.len()
            ));
        }
    }

    let nrows = rows.len();
    let flat: Vec<f64> = rows.into_iter().flatten().collect();
    Array2::from_shape_vec((nrows, ncols), flat)
        .map_err(|e| format!("dlmread: shape error: {e}"))
        .map(Value::Matrix)
}

/// Formats one f64 value for use in a delimited file.
/// Integers are written without decimal point; floats use full precision.
fn fmt_dlm_number(n: f64) -> String {
    if n.is_finite() && n == n.trunc() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// Writes a scalar or matrix to a delimiter-separated file.
fn dlmwrite_impl(path: &str, val: &Value, explicit_delim: Option<String>) -> Result<Value, String> {
    let delim = explicit_delim.unwrap_or_else(|| ",".to_string());

    let content = match val {
        Value::Scalar(n) => format!("{}\n", fmt_dlm_number(*n)),
        Value::Matrix(m) => {
            let mut out = String::new();
            for row in m.rows() {
                let parts: Vec<String> = row.iter().map(|n| fmt_dlm_number(*n)).collect();
                out.push_str(&parts.join(&delim));
                out.push('\n');
            }
            out
        }
        _ => {
            return Err("dlmwrite: second argument must be a numeric scalar or matrix".to_string());
        }
    };

    std::fs::write(path, content).map_err(|e| format!("dlmwrite: cannot write '{path}': {e}"))?;
    Ok(Value::Void)
}

// --- CSV read/write helpers (readmatrix / readtable / writetable) ---

/// Selects the delimiter shared across all lines; falls back to `None` (whitespace splitting).
///
/// Uses CSV-aware splitting (quoting) when checking for comma consistency.
fn auto_detect_delim(lines: &[&str]) -> Option<String> {
    // Comma: use CSV-aware split so quoted fields with commas don't confuse the count.
    let comma_counts: Vec<usize> = lines.iter().map(|l| split_csv_row(l, ",").len()).collect();
    if comma_counts.iter().all(|&c| c > 1) && comma_counts.windows(2).all(|w| w[0] == w[1]) {
        return Some(",".to_string());
    }
    if delim_consistent(lines, '\t') {
        Some("\t".to_string())
    } else {
        None
    }
}

/// Splits one CSV line by `delim`, respecting RFC 4180 double-quoted fields.
/// `""` inside a quoted field encodes a literal `"`.
/// Falls back to a plain `str::split` for multi-character delimiters.
fn split_csv_row(line: &str, delim: &str) -> Vec<String> {
    if delim.chars().count() != 1 {
        return line.split(delim).map(str::to_string).collect();
    }
    let delim_char = delim.chars().next().unwrap();
    let chars: Vec<char> = line.chars().collect();
    let mut fields: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut i = 0;
    let mut in_quotes = false;
    while i < chars.len() {
        let c = chars[i];
        if in_quotes {
            if c == '"' && i + 1 < chars.len() && chars[i + 1] == '"' {
                field.push('"');
                i += 2;
                continue;
            } else if c == '"' {
                in_quotes = false;
            } else {
                field.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == delim_char {
            fields.push(std::mem::take(&mut field));
        } else {
            field.push(c);
        }
        i += 1;
    }
    fields.push(field);
    fields
}

/// Splits a CSV row with an optional delimiter; `None` splits by whitespace.
fn split_csv_row_opt(line: &str, delim: &Option<String>) -> Vec<String> {
    match delim {
        None => line.split_whitespace().map(str::to_string).collect(),
        Some(d) => split_csv_row(line, d),
    }
}

/// Returns `true` if any non-empty field in `fields` cannot be parsed as `f64`.
fn row_is_header(fields: &[String]) -> bool {
    fields
        .iter()
        .any(|f| !f.trim().is_empty() && f.trim().parse::<f64>().is_err())
}

/// Converts a raw header string to a valid identifier-like name.
/// Runs of non-alphanumeric characters collapse to `_`; a leading digit gets an `x` prefix.
/// Empty results fall back to `x{col}`.
fn sanitize_header(s: &str, col_1based: usize) -> String {
    let s = s.trim();
    if s.is_empty() {
        return format!("x{col_1based}");
    }
    let mut out = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() || c == '_' {
            out.push(c);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    let out = out.trim_end_matches('_').to_string();
    if out.is_empty() {
        return format!("x{col_1based}");
    }
    if out.chars().next().unwrap().is_ascii_digit() {
        format!("x{out}")
    } else {
        out
    }
}

/// Appends `_N` (1-based) suffixes to duplicate entries in a header list.
/// Note: collisions between deduplicated names and pre-existing `_N` names are not resolved.
fn deduplicate_headers(headers: Vec<String>) -> Vec<String> {
    let mut count: HashMap<String, usize> = HashMap::new();
    for h in &headers {
        *count.entry(h.clone()).or_insert(0) += 1;
    }
    let mut seen: HashMap<String, usize> = HashMap::new();
    headers
        .into_iter()
        .map(|h| {
            if *count.get(&h).unwrap() == 1 {
                h
            } else {
                let idx = seen.entry(h.clone()).or_insert(0);
                *idx += 1;
                format!("{h}_{idx}")
            }
        })
        .collect()
}

/// Parses an optional `('Delimiter', d)` argument pair starting at `args[start]`.
/// Returns `Ok(None)` when no extra arguments are present.
fn parse_delimiter_opt(
    fn_name: &str,
    args: &[Value],
    start: usize,
) -> Result<Option<String>, String> {
    if args.len() <= start {
        return Ok(None);
    }
    let key = string_arg(&args[start], fn_name, start + 1)?;
    if !key.eq_ignore_ascii_case("delimiter") {
        return Err(format!(
            "{fn_name}: expected 'Delimiter' option at argument {}, got '{key}'",
            start + 1
        ));
    }
    if args.len() <= start + 1 {
        return Err(format!("{fn_name}: 'Delimiter' option requires a value"));
    }
    let val = interpret_delim(string_arg(&args[start + 1], fn_name, start + 2)?);
    Ok(Some(val))
}

/// Reads a delimiter-separated file and returns a [`Value::Matrix`].
///
/// Auto-detects the delimiter (comma → tab → whitespace). When the first row contains
/// non-numeric text it is treated as a header and skipped. Empty cells become `NaN`
/// (unlike [`dlmread_impl`], which uses `0.0`).
fn readmatrix_impl(path: &str, explicit_delim: Option<String>) -> Result<Value, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("readmatrix: cannot read '{path}': {e}"))?;

    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return Ok(Value::Matrix(Array2::<f64>::zeros((0, 0))));
    }

    let delim = match explicit_delim {
        Some(d) => Some(d),
        None => auto_detect_delim(&lines),
    };

    let first_fields = split_csv_row_opt(lines[0], &delim);
    let skip_header = row_is_header(&first_fields);
    let data_lines = if skip_header { &lines[1..] } else { &lines[..] };

    if data_lines.is_empty() {
        return Ok(Value::Matrix(Array2::<f64>::zeros((0, 0))));
    }

    let mut rows: Vec<Vec<f64>> = Vec::new();
    for (i, line) in data_lines.iter().enumerate() {
        let fields = split_csv_row_opt(line, &delim);
        let mut row: Vec<f64> = Vec::with_capacity(fields.len());
        for f in &fields {
            let t = f.trim();
            if t.is_empty() {
                row.push(f64::NAN);
            } else {
                row.push(t.parse::<f64>().map_err(|_| {
                    format!(
                        "readmatrix: non-numeric value '{t}' on line {}",
                        i + 1 + usize::from(skip_header)
                    )
                })?);
            }
        }
        rows.push(row);
    }

    if rows.is_empty() {
        return Ok(Value::Matrix(Array2::<f64>::zeros((0, 0))));
    }

    let ncols = rows[0].len();
    for (i, row) in rows.iter().enumerate() {
        if row.len() != ncols {
            return Err(format!(
                "readmatrix: row {} has {} fields, expected {ncols}",
                i + 1,
                row.len()
            ));
        }
    }

    let nrows = rows.len();
    let flat: Vec<f64> = rows.into_iter().flatten().collect();
    Array2::from_shape_vec((nrows, ncols), flat)
        .map_err(|e| format!("readmatrix: shape error: {e}"))
        .map(Value::Matrix)
}

/// Reads a delimiter-separated file with a header row and returns a [`Value::Struct`] of columns.
///
/// The first row is always treated as column headers. Numeric columns become `Matrix` (N×1);
/// columns with any non-numeric cell become `Cell` of [`Value::Str`].
/// Whitespace is trimmed from all cell values after CSV unquoting.
fn readtable_impl(path: &str, explicit_delim: Option<String>) -> Result<Value, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("readtable: cannot read '{path}': {e}"))?;

    let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return Ok(Value::Struct(IndexMap::new()));
    }

    let delim = match explicit_delim {
        Some(d) => Some(d),
        None => auto_detect_delim(&lines),
    };

    let raw_headers = split_csv_row_opt(lines[0], &delim);
    let ncols = raw_headers.len();
    let headers: Vec<String> = deduplicate_headers(
        raw_headers
            .iter()
            .enumerate()
            .map(|(i, h)| sanitize_header(h.trim(), i + 1))
            .collect(),
    );

    let data_lines = &lines[1..];
    if data_lines.is_empty() {
        let mut s: IndexMap<String, Value> = IndexMap::new();
        for h in &headers {
            s.insert(h.clone(), Value::Matrix(Array2::<f64>::zeros((0, 1))));
        }
        return Ok(Value::Struct(s));
    }

    let mut all_rows: Vec<Vec<String>> = Vec::new();
    for (i, line) in data_lines.iter().enumerate() {
        let fields = split_csv_row_opt(line, &delim);
        if fields.len() != ncols {
            return Err(format!(
                "readtable: row {} has {} fields, expected {ncols}",
                i + 2,
                fields.len()
            ));
        }
        all_rows.push(fields.into_iter().map(|f| f.trim().to_string()).collect());
    }

    let nrows = all_rows.len();
    let mut s: IndexMap<String, Value> = IndexMap::new();
    for col in 0..ncols {
        let all_numeric = all_rows.iter().all(|row| {
            let t = row[col].as_str();
            t.is_empty() || t.parse::<f64>().is_ok()
        });
        if all_numeric {
            let vals: Vec<f64> = all_rows
                .iter()
                .map(|row| {
                    let t = row[col].as_str();
                    if t.is_empty() {
                        f64::NAN
                    } else {
                        t.parse::<f64>().unwrap()
                    }
                })
                .collect();
            let col_mat = Array2::from_shape_vec((nrows, 1), vals)
                .map_err(|e| format!("readtable: shape error: {e}"))?;
            s.insert(headers[col].clone(), Value::Matrix(col_mat));
        } else {
            let vals: Vec<Value> = all_rows
                .iter()
                .map(|row| Value::Str(row[col].clone()))
                .collect();
            s.insert(headers[col].clone(), Value::Cell(vals));
        }
    }
    Ok(Value::Struct(s))
}

/// Quotes a CSV cell if it contains the delimiter, a double-quote, or a newline (RFC 4180).
fn csv_quote_cell(s: &str, delim: &str) -> String {
    if s.contains('"') || s.contains('\n') || s.contains(delim) {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

/// Returns the number of rows in a struct column value.
///
/// Accepts `Matrix` (N×1), `Cell`, `Scalar`, and `Str`/`StringObj` (1 row each).
/// Returns `None` for unsupported types or non-column matrices.
fn col_nrows(v: &Value) -> Option<usize> {
    match v {
        Value::Matrix(m) if m.ncols() == 1 || m.nrows() == 0 => Some(m.nrows()),
        Value::Cell(c) => Some(c.len()),
        Value::Scalar(_) => Some(1),
        Value::Str(_) | Value::StringObj(_) => Some(1),
        _ => None,
    }
}

/// Returns the formatted CSV cell string for row `row` of a struct column value.
fn col_cell_str(v: &Value, row: usize, delim: &str) -> Result<String, String> {
    match v {
        Value::Matrix(m) => Ok(csv_quote_cell(&fmt_dlm_number(m[[row, 0]]), delim)),
        Value::Cell(c) => match &c[row] {
            Value::Str(s) | Value::StringObj(s) => Ok(csv_quote_cell(s, delim)),
            Value::Scalar(n) => Ok(csv_quote_cell(&fmt_dlm_number(*n), delim)),
            _ => Err(format!(
                "writetable: cell element at row {} has unsupported type",
                row + 1
            )),
        },
        Value::Scalar(n) => Ok(csv_quote_cell(&fmt_dlm_number(*n), delim)),
        Value::Str(s) | Value::StringObj(s) => Ok(csv_quote_cell(s, delim)),
        _ => Err(format!(
            "writetable: unsupported column type at row {}",
            row + 1
        )),
    }
}

/// Writes a struct table to a delimiter-separated file with a header row.
///
/// Each struct field is one column. All fields must have the same number of rows.
/// Accepts `Matrix` (N×1), `Cell`, `Scalar`, and `Str`/`StringObj` columns.
/// Cell values that contain the delimiter, `"`, or newlines are quoted per RFC 4180.
fn writetable_impl(
    tbl: &Value,
    path: &str,
    explicit_delim: Option<String>,
) -> Result<Value, String> {
    let delim = explicit_delim.unwrap_or_else(|| ",".to_string());
    let fields = match tbl {
        Value::Struct(m) => m,
        _ => return Err("writetable: first argument must be a struct".to_string()),
    };
    if fields.is_empty() {
        std::fs::write(path, "").map_err(|e| format!("writetable: cannot write '{path}': {e}"))?;
        return Ok(Value::Void);
    }

    let nrows = {
        let (first_name, first_val) = fields.iter().next().unwrap();
        col_nrows(first_val).ok_or_else(|| {
            format!("writetable: column '{first_name}' must be a Matrix (N×1), Cell, or scalar")
        })?
    };
    for (cname, cval) in fields.iter() {
        let n = col_nrows(cval).ok_or_else(|| {
            format!("writetable: column '{cname}' must be a Matrix (N×1), Cell, or scalar")
        })?;
        if n != nrows {
            return Err(format!(
                "writetable: column '{cname}' has {n} rows, expected {nrows}"
            ));
        }
    }

    let mut out = String::new();
    let header_parts: Vec<String> = fields.keys().map(|k| csv_quote_cell(k, &delim)).collect();
    out.push_str(&header_parts.join(&delim));
    out.push('\n');

    for row in 0..nrows {
        let mut parts: Vec<String> = Vec::with_capacity(fields.len());
        for cval in fields.values() {
            parts.push(col_cell_str(cval, row, &delim)?);
        }
        out.push_str(&parts.join(&delim));
        out.push('\n');
    }

    std::fs::write(path, out).map_err(|e| format!("writetable: cannot write '{path}': {e}"))?;
    Ok(Value::Void)
}

/// Converts an f64 to u64 for bitwise operations.
/// Requires a non-negative integer value; returns an error otherwise.
fn to_bits(v: f64, fname: &str, pos: usize) -> Result<u64, String> {
    if v < 0.0 {
        return Err(format!(
            "{fname}: argument {pos} must be non-negative, got {v}"
        ));
    }
    if v.fract() != 0.0 {
        return Err(format!(
            "{fname}: argument {pos} must be an integer, got {v}"
        ));
    }
    if v > u64::MAX as f64 {
        return Err(format!(
            "{fname}: argument {pos} is too large for bitwise operations"
        ));
    }
    Ok(v as u64)
}

/// Computes determinant of a square matrix via Gaussian elimination.
/// Computes the determinant of a square matrix via Gaussian elimination with
/// partial pivoting (pure Rust, no external dependencies).
fn det_matrix(m: &Array2<f64>) -> Result<f64, String> {
    let n = m.nrows();
    if m.ncols() != n {
        return Err("det: matrix must be square".to_string());
    }
    if n == 0 {
        return Ok(1.0);
    }
    let mut a = m.clone();
    let mut sign: f64 = 1.0;
    for col in 0..n {
        // Partial pivoting: swap in the row with the largest absolute value.
        let pivot = (col..n)
            .max_by(|&r1, &r2| a[[r1, col]].abs().partial_cmp(&a[[r2, col]].abs()).unwrap())
            .unwrap();
        if a[[pivot, col]].abs() < 1e-15 {
            return Ok(0.0); // singular
        }
        if pivot != col {
            for j in 0..n {
                let tmp = a[[pivot, j]];
                a[[pivot, j]] = a[[col, j]];
                a[[col, j]] = tmp;
            }
            sign = -sign;
        }
        let pv = a[[col, col]];
        for row in (col + 1)..n {
            let factor = a[[row, col]] / pv;
            for j in col..n {
                let val = a[[col, j]] * factor;
                a[[row, j]] -= val;
            }
        }
    }
    Ok(sign * (0..n).map(|i| a[[i, i]]).product::<f64>())
}

/// Computes the inverse of a square matrix via Gauss-Jordan elimination with
/// partial pivoting (pure Rust, no external dependencies).
fn inv_matrix(m: &Array2<f64>) -> Result<Array2<f64>, String> {
    let n = m.nrows();
    if m.ncols() != n {
        return Err("inv: matrix must be square".to_string());
    }
    let cols = 2 * n;
    let mut aug = vec![0.0f64; n * cols];
    for i in 0..n {
        for j in 0..n {
            aug[i * cols + j] = m[[i, j]];
        }
        aug[i * cols + n + i] = 1.0;
    }
    for col in 0..n {
        // Partial pivoting: swap in the row with the largest absolute value.
        let pivot = (col..n)
            .max_by(|&r1, &r2| {
                aug[r1 * cols + col]
                    .abs()
                    .partial_cmp(&aug[r2 * cols + col].abs())
                    .unwrap()
            })
            .filter(|&r| aug[r * cols + col].abs() > 1e-12)
            .ok_or_else(|| "inv: matrix is singular".to_string())?;
        if pivot != col {
            for j in 0..cols {
                aug.swap(col * cols + j, pivot * cols + j);
            }
        }
        let pv = aug[col * cols + col];
        for j in 0..cols {
            aug[col * cols + j] /= pv;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row * cols + col];
            for j in 0..cols {
                let val = aug[col * cols + j] * factor;
                aug[row * cols + j] -= val;
            }
        }
    }
    let mut result = Array2::<f64>::zeros((n, n));
    for i in 0..n {
        for j in 0..n {
            result[[i, j]] = aug[i * cols + n + j];
        }
    }
    Ok(result)
}

/// Solves the linear system `A * x = B` using Gaussian elimination with partial pivoting.
///
/// `A` must be square (n×n); `B` must have n rows. Returns x (n × k where k = B.ncols()).
/// This is the engine for the `\` left-division operator.
fn solve_linear(a: &Array2<f64>, b: &Array2<f64>) -> Result<Array2<f64>, String> {
    let n = a.nrows();
    if a.ncols() != n {
        return Err(format!(
            "\\: coefficient matrix must be square, got {}×{}",
            n,
            a.ncols()
        ));
    }
    let k = b.ncols();
    if b.nrows() != n {
        return Err(format!(
            "\\: size mismatch — A is {}×{} but b has {} rows",
            n,
            n,
            b.nrows()
        ));
    }
    if n == 0 {
        return Ok(Array2::zeros((0, k)));
    }
    let cols = n + k;
    let mut aug = vec![0.0f64; n * cols];
    for i in 0..n {
        for j in 0..n {
            aug[i * cols + j] = a[[i, j]];
        }
        for j in 0..k {
            aug[i * cols + n + j] = b[[i, j]];
        }
    }
    for col in 0..n {
        let pivot = (col..n)
            .max_by(|&r1, &r2| {
                aug[r1 * cols + col]
                    .abs()
                    .partial_cmp(&aug[r2 * cols + col].abs())
                    .unwrap()
            })
            .filter(|&r| aug[r * cols + col].abs() > 1e-12)
            .ok_or_else(|| "\\: matrix is singular or nearly singular".to_string())?;
        if pivot != col {
            for j in 0..cols {
                aug.swap(col * cols + j, pivot * cols + j);
            }
        }
        let pv = aug[col * cols + col];
        for j in col..cols {
            aug[col * cols + j] /= pv;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = aug[row * cols + col];
            if factor == 0.0 {
                continue;
            }
            for j in col..cols {
                let val = aug[col * cols + j] * factor;
                aug[row * cols + j] -= val;
            }
        }
    }
    let mut result = Array2::<f64>::zeros((n, k));
    for i in 0..n {
        for j in 0..k {
            result[[i, j]] = aug[i * cols + n + j];
        }
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Advanced linear algebra helpers (Phase 18)
// ---------------------------------------------------------------------------

/// QR decomposition via Householder reflectors.
///
/// For an m×n matrix A returns (Q, R) where Q is m×m orthogonal and R is
/// m×n upper triangular such that A = Q * R.
fn qr_decompose(a: &Array2<f64>) -> Result<(Array2<f64>, Array2<f64>), String> {
    let m = a.nrows();
    let n = a.ncols();
    let k = m.min(n);
    let mut r = a.clone();
    let mut q = Array2::<f64>::eye(m);

    for j in 0..k {
        let col_len = m - j;
        let mut v: Vec<f64> = (j..m).map(|i| r[[i, j]]).collect();

        let norm_x = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
        if norm_x < 1e-14 {
            continue;
        }
        // Householder sign convention avoids cancellation.
        v[0] += if v[0] >= 0.0 { norm_x } else { -norm_x };
        let v_sq: f64 = v.iter().map(|&x| x * x).sum();
        if v_sq < 1e-28 {
            continue;
        }

        // Apply H from left to R: R[j:m, :] -= 2*v*(v^T*R[j:m,:])/v^Tv
        for col in j..n {
            let dot: f64 = (0..col_len).map(|i| v[i] * r[[j + i, col]]).sum();
            let fac = 2.0 * dot / v_sq;
            for i in 0..col_len {
                r[[j + i, col]] -= fac * v[i];
            }
        }
        // Accumulate Q from right: Q[:, j:m] -= (Q[:,j:m]*v) * 2*v^T/v^Tv
        for row in 0..m {
            let dot: f64 = (0..col_len).map(|i| q[[row, j + i]] * v[i]).sum();
            let fac = 2.0 * dot / v_sq;
            for i in 0..col_len {
                q[[row, j + i]] -= fac * v[i];
            }
        }
    }

    Ok((q, r))
}

/// LU decomposition with partial pivoting.
///
/// For an n×n square matrix A returns (L, U, P) where P*A = L*U,
/// L is unit lower triangular, U is upper triangular, and P is a
/// permutation matrix.
type LuResult = Result<(Array2<f64>, Array2<f64>, Array2<f64>), String>;
fn lu_decompose(a: &Array2<f64>) -> LuResult {
    let n = a.nrows();
    if a.ncols() != n {
        return Err("lu: matrix must be square".to_string());
    }
    let mut u = a.clone();
    let mut l = Array2::<f64>::eye(n);
    let mut perm: Vec<usize> = (0..n).collect();

    for j in 0..n {
        let pivot = (j..n)
            .max_by(|&r1, &r2| {
                u[[r1, j]]
                    .abs()
                    .partial_cmp(&u[[r2, j]].abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        if pivot != j {
            for col in 0..n {
                let tmp = u[[j, col]];
                u[[j, col]] = u[[pivot, col]];
                u[[pivot, col]] = tmp;
            }
            for col in 0..j {
                let tmp = l[[j, col]];
                l[[j, col]] = l[[pivot, col]];
                l[[pivot, col]] = tmp;
            }
            perm.swap(j, pivot);
        }

        if u[[j, j]].abs() < 1e-15 {
            continue;
        }
        for i in (j + 1)..n {
            l[[i, j]] = u[[i, j]] / u[[j, j]];
            for k in j..n {
                let val = l[[i, j]] * u[[j, k]];
                u[[i, k]] -= val;
            }
        }
    }

    let mut p = Array2::<f64>::zeros((n, n));
    for (i, &j) in perm.iter().enumerate() {
        p[[i, j]] = 1.0;
    }
    Ok((l, u, p))
}

/// Cholesky decomposition.
///
/// For a symmetric positive-definite n×n matrix A returns the upper triangular
/// factor R such that A = R^T * R (MATLAB convention).
fn chol_decompose(a: &Array2<f64>) -> Result<Array2<f64>, String> {
    let n = a.nrows();
    if a.ncols() != n {
        return Err("chol: matrix must be square".to_string());
    }
    let mut r = Array2::<f64>::zeros((n, n));
    for j in 0..n {
        let mut s = a[[j, j]];
        for k in 0..j {
            s -= r[[k, j]] * r[[k, j]];
        }
        if s <= 0.0 {
            return Err("chol: matrix is not positive definite".to_string());
        }
        r[[j, j]] = s.sqrt();
        for i in (j + 1)..n {
            let mut t = a[[j, i]];
            for k in 0..j {
                t -= r[[k, j]] * r[[k, i]];
            }
            r[[j, i]] = t / r[[j, j]];
        }
    }
    Ok(r)
}

/// One-sided Jacobi SVD (economy form).
///
/// For an m×n matrix A returns (U, s, V) where
/// - U is m×k with orthonormal columns (k = min(m,n))
/// - s is a `Vec<f64>` of singular values in descending order (length k)
/// - V is n×k with orthonormal columns
///
/// For m < n the inputs are transparently transposed and outputs swapped.
type SvdResult = Result<(Array2<f64>, Vec<f64>, Array2<f64>), String>;
fn svd_compute(a: &Array2<f64>) -> SvdResult {
    let m = a.nrows();
    let n = a.ncols();
    if m < n {
        let (v, s, u) = svd_compute(&a.t().to_owned())?;
        return Ok((u, s, v));
    }
    // m >= n from here.
    let k = n;
    let mut b = a.clone();
    let mut v = Array2::<f64>::eye(k);

    const MAX_ITER: usize = 200;
    const EPS: f64 = 1e-14;

    'outer: for _ in 0..MAX_ITER {
        let mut changed = false;
        for p in 0..k {
            for q in (p + 1)..k {
                let alpha: f64 = (0..m).map(|i| b[[i, p]] * b[[i, p]]).sum();
                let beta: f64 = (0..m).map(|i| b[[i, q]] * b[[i, q]]).sum();
                let gamma: f64 = (0..m).map(|i| b[[i, p]] * b[[i, q]]).sum();

                if gamma.abs() <= EPS * (alpha * beta).sqrt() {
                    continue;
                }
                changed = true;

                let zeta = (beta - alpha) / (2.0 * gamma);
                let t = zeta.signum() / (zeta.abs() + (1.0 + zeta * zeta).sqrt());
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = c * t;

                for i in 0..m {
                    let bp = b[[i, p]];
                    let bq = b[[i, q]];
                    b[[i, p]] = c * bp - s * bq;
                    b[[i, q]] = s * bp + c * bq;
                }
                for i in 0..k {
                    let vp = v[[i, p]];
                    let vq = v[[i, q]];
                    v[[i, p]] = c * vp - s * vq;
                    v[[i, q]] = s * vp + c * vq;
                }
            }
        }
        if !changed {
            break 'outer;
        }
    }

    let mut sigma: Vec<f64> = (0..k)
        .map(|j| (0..m).map(|i| b[[i, j]] * b[[i, j]]).sum::<f64>().sqrt())
        .collect();
    let mut u_mat = Array2::<f64>::zeros((m, k));
    for j in 0..k {
        if sigma[j] > EPS {
            for i in 0..m {
                u_mat[[i, j]] = b[[i, j]] / sigma[j];
            }
        }
    }

    // Sort descending by singular value.
    let mut order: Vec<usize> = (0..k).collect();
    order.sort_by(|&a, &b| {
        sigma[b]
            .partial_cmp(&sigma[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let sigma_s: Vec<f64> = order.iter().map(|&i| sigma[i]).collect();
    let mut u_s = Array2::<f64>::zeros((m, k));
    let mut v_s = Array2::<f64>::zeros((n, k));
    for (ni, &oi) in order.iter().enumerate() {
        for r in 0..m {
            u_s[[r, ni]] = u_mat[[r, oi]];
        }
        for r in 0..k {
            v_s[[r, ni]] = v[[r, oi]];
        }
    }
    sigma = sigma_s;

    Ok((u_s, sigma, v_s))
}

/// Extends an m×k matrix with orthonormal columns to a full m×m orthogonal matrix.
///
/// Tries each standard basis vector e_0..e_{m-1} in order; keeps those that
/// have non-negligible component orthogonal to the existing basis.
fn complete_orthonormal_basis(u: &Array2<f64>) -> Array2<f64> {
    let m = u.nrows();
    let k = u.ncols();
    let mut basis: Vec<Vec<f64>> = (0..k).map(|j| u.column(j).to_vec()).collect();

    let mut ei = 0usize;
    while basis.len() < m && ei < m {
        let mut v: Vec<f64> = vec![0.0; m];
        v[ei] = 1.0;
        ei += 1;
        for b in &basis {
            let dot: f64 = v.iter().zip(b.iter()).map(|(&a, &b)| a * b).sum();
            for (vi, &bi) in v.iter_mut().zip(b.iter()) {
                *vi -= dot * bi;
            }
        }
        let norm = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
        if norm > 1e-10 {
            for vi in &mut v {
                *vi /= norm;
            }
            basis.push(v);
        }
    }

    let mut result = Array2::<f64>::zeros((m, m));
    for (j, b) in basis.iter().enumerate() {
        for (i, &val) in b.iter().enumerate() {
            result[[i, j]] = val;
        }
    }
    result
}

/// QR-iteration eigendecomposition for a real square matrix.
///
/// Returns `(eigenvalues, eigenvectors)` where eigenvalues is a `Vec<f64>` of
/// length n and eigenvectors is an n×n matrix whose columns are the eigenvectors.
/// Uses the basic QR iteration with a simple diagonal shift (Wilkinson-style).
/// Convergence is reliable for symmetric matrices; general matrices converge for
/// most well-conditioned inputs within `MAX_ITER` steps.
fn eig_compute(a: &Array2<f64>) -> Result<(Vec<f64>, Array2<f64>), String> {
    let n = a.nrows();
    if a.ncols() != n {
        return Err("eig: matrix must be square".to_string());
    }
    if n == 0 {
        return Ok((vec![], Array2::zeros((0, 0))));
    }
    if n == 1 {
        return Ok((vec![a[[0, 0]]], Array2::eye(1)));
    }

    let mut ak = a.clone();
    let mut evecs = Array2::<f64>::eye(n);

    const MAX_ITER: usize = 2000;
    const EPS: f64 = 1e-12;

    for _ in 0..MAX_ITER {
        // Wilkinson shift: uses the trailing 2×2 submatrix for cubic convergence.
        let mu = {
            let d = ak[[n - 1, n - 1]];
            if n >= 2 {
                let a = ak[[n - 2, n - 2]];
                let b = ak[[n - 2, n - 1]];
                let delta = (a - d) / 2.0;
                if delta.abs() < 1e-30 {
                    d - b.abs()
                } else {
                    d - b * b / (delta + delta.signum() * (delta * delta + b * b).sqrt())
                }
            } else {
                d
            }
        };

        for i in 0..n {
            ak[[i, i]] -= mu;
        }
        let (q, r) = qr_decompose(&ak)?;
        ak = r.dot(&q);
        for i in 0..n {
            ak[[i, i]] += mu;
        }
        evecs = evecs.dot(&q);

        let max_sub = (0..(n - 1))
            .map(|i| ak[[i + 1, i]].abs())
            .fold(0.0_f64, f64::max);
        if max_sub < EPS {
            break;
        }
    }

    let evals: Vec<f64> = (0..n).map(|i| ak[[i, i]]).collect();
    Ok((evals, evecs))
}

// ---------------------------------------------------------------------------
// Indexing
// ---------------------------------------------------------------------------

/// Creates a copy of `env` with `end` set to `dim_size`.
/// Used by `eval_index` so that `end` in index expressions resolves to the correct dimension size.
fn env_with_end(env: &Env, dim_size: usize) -> Env {
    let mut e = env.clone();
    e.insert("end".to_string(), Value::Scalar(dim_size as f64));
    e
}

/// Evaluates `val(args...)` — indexing a variable with one or two index arguments.
///
/// Disambiguation rule (Octave semantics): a name that exists in `Env` is always
/// treated as a variable to be indexed, never as a function call.
fn eval_index(val: &Value, args: &[Expr], env: &Env) -> Result<Value, String> {
    match args.len() {
        0 => Err("Indexing requires at least one index".to_string()),
        1 => {
            // v(i), v(1:3), v(:), v(end), v(end-1:end)
            match val {
                Value::Void => Err("Cannot index into void".to_string()),
                Value::Lambda(_) | Value::Function { .. } | Value::Tuple(_) => {
                    Err("Cannot index into a function value".to_string())
                }
                Value::Cell(_) => Err("Use c{i} to index into a cell array, not c(i)".to_string()),
                Value::Struct(_) => {
                    Err("Use s.field to access struct fields, not s(i)".to_string())
                }
                Value::StructArray(arr) => {
                    let total = arr.len();
                    let env1 = env_with_end(env, total);
                    match resolve_dim(&args[0], total, &env1)? {
                        DimIdx::All => {
                            // s(:) — return all elements as a new struct array
                            Ok(Value::StructArray(arr.clone()))
                        }
                        DimIdx::Indices(idxs) => {
                            if idxs.len() == 1 {
                                let i = idxs[0];
                                if i >= total {
                                    return Err(format!(
                                        "Index {} out of range (1..{})",
                                        i + 1,
                                        total
                                    ));
                                }
                                Ok(Value::Struct(arr[i].clone()))
                            } else {
                                let mut selected = Vec::with_capacity(idxs.len());
                                for &i in &idxs {
                                    if i >= total {
                                        return Err(format!(
                                            "Index {} out of range (1..{})",
                                            i + 1,
                                            total
                                        ));
                                    }
                                    selected.push(arr[i].clone());
                                }
                                Ok(Value::StructArray(selected))
                            }
                        }
                    }
                }
                Value::Scalar(n) => {
                    let env1 = env_with_end(env, 1);
                    match resolve_dim(&args[0], 1, &env1)? {
                        DimIdx::All | DimIdx::Indices(_) => Ok(Value::Scalar(*n)),
                    }
                }
                Value::Complex(re, im) => {
                    let env1 = env_with_end(env, 1);
                    match resolve_dim(&args[0], 1, &env1)? {
                        DimIdx::All | DimIdx::Indices(_) => Ok(Value::Complex(*re, *im)),
                    }
                }
                Value::Matrix(m) => {
                    let total = m.nrows() * m.ncols();
                    let env1 = env_with_end(env, total);
                    match resolve_dim(&args[0], total, &env1)? {
                        DimIdx::All => {
                            // A(:) → column vector, column-major order
                            let mut flat = Vec::with_capacity(total);
                            for col in 0..m.ncols() {
                                for row in 0..m.nrows() {
                                    flat.push(m[[row, col]]);
                                }
                            }
                            Ok(Value::Matrix(
                                Array2::from_shape_vec((total, 1), flat).unwrap(),
                            ))
                        }
                        DimIdx::Indices(idxs) => {
                            // Column-major linear indexing
                            let nrows = m.nrows();
                            let ncols_m = m.ncols();
                            let vals: Result<Vec<f64>, String> = idxs
                                .iter()
                                .map(|&i| {
                                    // i is 0-based, column-major
                                    let row = i % nrows;
                                    let col = i / nrows;
                                    if col >= ncols_m {
                                        Err(format!("Index {} out of range (1..{})", i + 1, total))
                                    } else {
                                        Ok(m[[row, col]])
                                    }
                                })
                                .collect();
                            let vals = vals?;
                            if vals.len() == 1 {
                                Ok(Value::Scalar(vals[0]))
                            } else {
                                let n = vals.len();
                                Ok(Value::Matrix(Array2::from_shape_vec((1, n), vals).unwrap()))
                            }
                        }
                    }
                }
                Value::Str(s) => {
                    // Index into a char array — returns char code(s)
                    let chars: Vec<char> = s.chars().collect();
                    let total = chars.len();
                    let env1 = env_with_end(env, total);
                    match resolve_dim(&args[0], total, &env1)? {
                        DimIdx::All => {
                            let codes: Vec<f64> = chars.iter().map(|&c| c as u32 as f64).collect();
                            if codes.len() == 1 {
                                Ok(Value::Scalar(codes[0]))
                            } else {
                                let n = codes.len();
                                Ok(Value::Matrix(
                                    Array2::from_shape_vec((1, n), codes).unwrap(),
                                ))
                            }
                        }
                        DimIdx::Indices(idxs) => {
                            let mut selected = String::new();
                            for &i in &idxs {
                                if i >= chars.len() {
                                    return Err(format!("Index {} out of range", i + 1));
                                }
                                selected.push(chars[i]);
                            }
                            if selected.chars().count() == 1 {
                                Ok(Value::Scalar(selected.chars().next().unwrap() as u32 as f64))
                            } else {
                                Ok(Value::Str(selected))
                            }
                        }
                    }
                }
                Value::StringObj(s) => {
                    // String object indexing — treat as single element
                    let env1 = env_with_end(env, 1);
                    match resolve_dim(&args[0], 1, &env1)? {
                        DimIdx::All | DimIdx::Indices(_) => Ok(Value::StringObj(s.clone())),
                    }
                }
                Value::DateTimeArray(v) => {
                    let total = v.len();
                    let env1 = env_with_end(env, total);
                    match resolve_dim(&args[0], total, &env1)? {
                        DimIdx::All => Ok(Value::DateTimeArray(v.clone())),
                        DimIdx::Indices(idxs) => {
                            if idxs.len() == 1 {
                                let i = idxs[0];
                                if i >= total {
                                    return Err(format!(
                                        "Index {} out of range (1..{})",
                                        i + 1,
                                        total
                                    ));
                                }
                                Ok(Value::DateTime(v[i]))
                            } else {
                                let mut sel = Vec::with_capacity(idxs.len());
                                for &i in &idxs {
                                    if i >= total {
                                        return Err(format!(
                                            "Index {} out of range (1..{})",
                                            i + 1,
                                            total
                                        ));
                                    }
                                    sel.push(v[i]);
                                }
                                Ok(Value::DateTimeArray(sel))
                            }
                        }
                    }
                }
                Value::DurationArray(v) => {
                    let total = v.len();
                    let env1 = env_with_end(env, total);
                    match resolve_dim(&args[0], total, &env1)? {
                        DimIdx::All => Ok(Value::DurationArray(v.clone())),
                        DimIdx::Indices(idxs) => {
                            if idxs.len() == 1 {
                                let i = idxs[0];
                                if i >= total {
                                    return Err(format!(
                                        "Index {} out of range (1..{})",
                                        i + 1,
                                        total
                                    ));
                                }
                                Ok(Value::Duration(v[i]))
                            } else {
                                let mut sel = Vec::with_capacity(idxs.len());
                                for &i in &idxs {
                                    if i >= total {
                                        return Err(format!(
                                            "Index {} out of range (1..{})",
                                            i + 1,
                                            total
                                        ));
                                    }
                                    sel.push(v[i]);
                                }
                                Ok(Value::DurationArray(sel))
                            }
                        }
                    }
                }
                Value::DateTime(_) | Value::Duration(_) => {
                    // Scalar datetime/duration: indexing with (1) is valid, returns self.
                    let env1 = env_with_end(env, 1);
                    match resolve_dim(&args[0], 1, &env1)? {
                        DimIdx::All | DimIdx::Indices(_) => Ok(val.clone()),
                    }
                }
            }
        }
        2 => {
            // A(i, j), A(:, j), A(i, :), A(:, :), A(end, :), A(1:end, 2)
            if matches!(
                val,
                Value::Void
                    | Value::Str(_)
                    | Value::StringObj(_)
                    | Value::Lambda(_)
                    | Value::Function { .. }
                    | Value::Tuple(_)
                    | Value::Cell(_)
                    | Value::Struct(_)
                    | Value::StructArray(_)
                    | Value::DateTime(_)
                    | Value::Duration(_)
                    | Value::DateTimeArray(_)
                    | Value::DurationArray(_)
            ) {
                return Err("2D indexing not supported for this type".to_string());
            }
            let (nrows, ncols) = match val {
                Value::Scalar(_) | Value::Complex(_, _) => (1, 1),
                Value::Matrix(m) => (m.nrows(), m.ncols()),
                _ => unreachable!(),
            };
            let env_r = env_with_end(env, nrows);
            let env_c = env_with_end(env, ncols);
            let row_idx = resolve_dim(&args[0], nrows, &env_r)?;
            let col_idx = resolve_dim(&args[1], ncols, &env_c)?;

            let rows: Vec<usize> = match row_idx {
                DimIdx::All => (0..nrows).collect(),
                DimIdx::Indices(v) => v,
            };
            let cols: Vec<usize> = match col_idx {
                DimIdx::All => (0..ncols).collect(),
                DimIdx::Indices(v) => v,
            };

            if rows.len() == 1 && cols.len() == 1 {
                match val {
                    Value::Scalar(n) => Ok(Value::Scalar(*n)),
                    Value::Complex(re, im) => Ok(Value::Complex(*re, *im)),
                    Value::Matrix(m) => Ok(Value::Scalar(m[[rows[0], cols[0]]])),
                    _ => unreachable!(),
                }
            } else {
                let out_r = rows.len();
                let out_c = cols.len();
                let flat: Vec<f64> = rows
                    .iter()
                    .flat_map(|&r| {
                        cols.iter().map(move |&c| match val {
                            Value::Scalar(n) => *n,
                            Value::Complex(re, _) => *re,
                            Value::Matrix(m) => m[[r, c]],
                            _ => unreachable!(),
                        })
                    })
                    .collect();
                Ok(Value::Matrix(
                    Array2::from_shape_vec((out_r, out_c), flat).unwrap(),
                ))
            }
        }
        n => Err(format!(
            "Indexing with {n} indices is not supported (max 2)"
        )),
    }
}

/// Resolved index along one dimension. Indices are 0-based.
enum DimIdx {
    All,
    Indices(Vec<usize>),
}

/// Resolves one index argument for a dimension of size `dim_size`.
/// `Expr::Colon` → `DimIdx::All`.
/// Scalar → single 0-based index (validates 1-based bounds).
/// Row/column vector → multiple 0-based indices.
/// Logical mask: a 0/1 vector whose length equals `dim_size` selects positions where value is 1.
fn resolve_dim(expr: &Expr, dim_size: usize, env: &Env) -> Result<DimIdx, String> {
    if matches!(expr, Expr::Colon) {
        return Ok(DimIdx::All);
    }
    let val = eval(expr, env)?;
    let floats: Vec<f64> = match val {
        Value::Void => {
            return Err("Index must be numeric, not void".to_string());
        }
        Value::Scalar(n) => vec![n],
        Value::Complex(re, im) => {
            if im != 0.0 {
                return Err("Index must be real, not complex".to_string());
            }
            vec![re]
        }
        Value::Matrix(m) => {
            // Allow 2-D matrices only when they qualify as a logical mask (same numel as dim_size).
            let total = m.nrows() * m.ncols();
            if m.nrows() > 1 && m.ncols() > 1 && total != dim_size {
                return Err("Index must be a scalar or vector, not a matrix".to_string());
            }
            // Collect in column-major order so mask positions align with linear indexing.
            if m.nrows() > 1 && m.ncols() > 1 {
                let mut v = Vec::with_capacity(total);
                for col in 0..m.ncols() {
                    for row in 0..m.nrows() {
                        v.push(m[[row, col]]);
                    }
                }
                v
            } else {
                m.iter().copied().collect()
            }
        }
        Value::Str(_) | Value::StringObj(_) => {
            return Err("Index must be numeric, not a string".to_string());
        }
        Value::Lambda(_)
        | Value::Function { .. }
        | Value::Tuple(_)
        | Value::Cell(_)
        | Value::Struct(_)
        | Value::StructArray(_)
        | Value::DateTime(_)
        | Value::Duration(_)
        | Value::DateTimeArray(_)
        | Value::DurationArray(_) => {
            return Err("Index must be numeric, not a function or datetime".to_string());
        }
    };
    // Logical mask: a 0/1 array whose element count matches dim_size selects by boolean mask.
    if dim_size > 0 && floats.len() == dim_size && floats.iter().all(|&f| f == 0.0 || f == 1.0) {
        let idxs: Vec<usize> = floats
            .iter()
            .enumerate()
            .filter(|&(_, &f)| f == 1.0)
            .map(|(i, _)| i)
            .collect();
        return Ok(DimIdx::Indices(idxs));
    }
    let mut idxs = Vec::with_capacity(floats.len());
    for n in floats {
        let i = n.round() as i64;
        if i < 1 || i as usize > dim_size {
            return Err(format!("Index {i} out of range (1..{dim_size})"));
        }
        idxs.push(i as usize - 1);
    }
    Ok(DimIdx::Indices(idxs))
}

/// Formats a number for display: integers without decimal point,
/// floats with up to 10 significant fractional digits, trailing zeros trimmed.
/// Always decimal — used for expression re-display, not user-facing output.
pub fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else if n != 0.0 && (n.abs() >= 1e15 || n.abs() < 1e-9) {
        trim_sci(&format!("{:.15e}", n))
    } else {
        let s = format!("{:.10}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Formats a scalar `f64` for user-facing output using the given base and format mode.
pub fn format_scalar(n: f64, base: Base, mode: &FormatMode) -> String {
    // FormatMode::Hex always shows IEEE 754 bits regardless of base.
    if matches!(mode, FormatMode::Hex) {
        return format_decimal(n, mode);
    }
    match base {
        Base::Dec => format_decimal(n, mode),
        _ => format_non_dec(n, base),
    }
}

/// Formats a complex number `re + im*i` for display.
///
/// - `a + 0i` → `a`  (pure real)
/// - `0 + bi` → `bi`
/// - `im == ±1` suppresses the coefficient: `i`, `-i`, `a + i`, `a - i`
pub fn format_complex(re: f64, im: f64, mode: &FormatMode) -> String {
    if im == 0.0 {
        return format_decimal(re, mode);
    }
    let im_abs = im.abs();
    let im_str = if im_abs == 1.0 {
        String::new()
    } else {
        format_decimal(im_abs, mode)
    };
    if re == 0.0 {
        if im < 0.0 {
            format!("-{}i", im_str)
        } else {
            format!("{}i", im_str)
        }
    } else {
        let re_str = format_decimal(re, mode);
        if im < 0.0 {
            format!("{} - {}i", re_str, im_str)
        } else {
            format!("{} + {}i", re_str, im_str)
        }
    }
}

/// Reconstructs a source-like string from an `Expr`.
///
/// Used to populate the display string of lambda values so that
/// `f = @(x) x.^2` shows `f = @(x) x .^ 2` in the REPL.
pub fn expr_to_string(e: &Expr) -> String {
    match e {
        Expr::Number(n) => {
            if n.is_nan() {
                "nan".to_string()
            } else if n.is_infinite() {
                if *n > 0.0 {
                    "inf".to_string()
                } else {
                    "-inf".to_string()
                }
            } else {
                format!("{n}")
            }
        }
        Expr::Var(name) => name.clone(),
        Expr::UnaryMinus(e) => format!("-{}", expr_to_string(e)),
        Expr::UnaryNot(e) => format!("~{}", expr_to_string(e)),
        Expr::BinOp(l, op, r) => {
            let op_str = match op {
                Op::Add => "+",
                Op::Sub => "-",
                Op::Mul => "*",
                Op::Div => "/",
                Op::Pow => "^",
                Op::ElemMul => ".*",
                Op::ElemDiv => "./",
                Op::ElemPow => ".^",
                Op::Eq => "==",
                Op::NotEq => "~=",
                Op::Lt => "<",
                Op::Gt => ">",
                Op::LtEq => "<=",
                Op::GtEq => ">=",
                Op::And => "&&",
                Op::Or => "||",
                Op::ElemAnd => "&",
                Op::ElemOr => "|",
                Op::LDiv => "\\",
            };
            format!("{} {op_str} {}", expr_to_string(l), expr_to_string(r))
        }
        Expr::Call(name, args) => {
            let args_str = args
                .iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}({args_str})")
        }
        Expr::Transpose(e) => format!("{}'", expr_to_string(e)),
        Expr::PlainTranspose(e) => format!("{}.'", expr_to_string(e)),
        Expr::Range(start, step, stop) => {
            if let Some(step) = step {
                format!(
                    "{}:{}:{}",
                    expr_to_string(start),
                    expr_to_string(step),
                    expr_to_string(stop)
                )
            } else {
                format!("{}:{}", expr_to_string(start), expr_to_string(stop))
            }
        }
        Expr::StrLiteral(s) => format!("'{s}'"),
        Expr::StringObjLiteral(s) => format!("\"{s}\""),
        Expr::Lambda { params, body, .. } => {
            format!("@({}) {}", params.join(", "), expr_to_string(body))
        }
        Expr::FuncHandle(name) => format!("@{name}"),
        Expr::Matrix(_) => "[...]".to_string(),
        Expr::CellLiteral(_) => "{...}".to_string(),
        Expr::CellIndex(e, i) => format!("{}{{{}}}", expr_to_string(e), expr_to_string(i)),
        Expr::Colon => ":".to_string(),
        Expr::NaT => "NaT".to_string(),
        Expr::FieldGet(base, field) => format!("{}.{field}", expr_to_string(base)),
        Expr::DotCall(segs, args) => {
            let args_str = args
                .iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({args_str})", segs.join("."))
        }
    }
}

/// Formats a `Value` compactly: scalars as a number string, matrices as `[NxM double]`.
pub fn format_value(v: &Value, base: Base, mode: &FormatMode) -> String {
    match v {
        Value::Void => String::new(),
        Value::Scalar(n) => format_scalar(*n, base, mode),
        Value::Matrix(m) => format!("[{}x{} double]", m.nrows(), m.ncols()),
        Value::Complex(re, im) => format_complex(*re, *im, mode),
        Value::Str(s) => s.clone(),
        Value::StringObj(s) => s.clone(),
        Value::Lambda(lf) => lf.1.clone(),
        Value::Function {
            params, outputs, ..
        } => {
            let params_str = params.join(", ");
            let out_str = match outputs.len() {
                0 => String::new(),
                1 => format!("{} = ", outputs[0]),
                _ => format!("[{}] = ", outputs.join(", ")),
            };
            format!("@function {out_str}f({params_str})")
        }
        Value::Tuple(vals) => {
            let parts: Vec<String> = vals.iter().map(|v| format_value(v, base, mode)).collect();
            format!("({})", parts.join(", "))
        }
        Value::Cell(v) => format!("{{1×{} cell}}", v.len()),
        Value::Struct(_) => "[1×1 struct]".to_string(),
        Value::StructArray(arr) => format!("[1×{} struct]", arr.len()),
        Value::DateTime(ts) => crate::datetime::format_datetime(*ts),
        Value::Duration(s) => crate::datetime::format_duration(*s),
        Value::DateTimeArray(v) => format!("[{}×1 datetime]", v.len()),
        Value::DurationArray(v) => format!("[{}×1 duration]", v.len()),
    }
}

/// Returns `None` for scalars, complex numbers, strings, and void (displayed inline or suppressed);
/// `Some(full_string)` for matrices (MATLAB-style column-aligned display).
pub fn format_value_full(v: &Value, mode: &FormatMode) -> Option<String> {
    match v {
        Value::Void
        | Value::Scalar(_)
        | Value::Complex(_, _)
        | Value::Str(_)
        | Value::StringObj(_)
        | Value::Lambda(_)
        | Value::Function { .. }
        | Value::Tuple(_)
        | Value::DateTime(_)
        | Value::Duration(_) => None,
        Value::Matrix(m) => Some(format_matrix(m, mode)),
        Value::Cell(elems) => Some(format_cell(elems, mode)),
        Value::Struct(map) => Some(format_struct(map, mode)),
        Value::StructArray(arr) => Some(format_struct_array(arr, mode)),
        Value::DateTimeArray(v) => Some(format_datetime_array(v)),
        Value::DurationArray(v) => Some(format_duration_array(v)),
    }
}

/// Formats a cell array in MATLAB-style multi-line display.
fn format_cell(elems: &[Value], mode: &FormatMode) -> String {
    if elems.is_empty() {
        return "  {}".to_string();
    }
    let mut lines = vec!["  {".to_string()];
    for (i, val) in elems.iter().enumerate() {
        let label = format!("    [1,{}]", i + 1);
        match val {
            Value::Matrix(_) => {
                lines.push(format!("{label}:"));
                if let Some(full) = format_value_full(val, mode) {
                    for line in full.lines() {
                        lines.push(format!("   {line}"));
                    }
                }
            }
            Value::Cell(_) => {
                lines.push(format!("{label}: {}", format_value(val, Base::Dec, mode)));
            }
            _ => {
                lines.push(format!("{label}: {}", format_value(val, Base::Dec, mode)));
            }
        }
    }
    lines.push("  }".to_string());
    lines.join("\n")
}

/// Formats a struct in MATLAB 2014b+ multi-line style.
fn format_struct(map: &IndexMap<String, Value>, mode: &FormatMode) -> String {
    let mut lines = vec![
        String::new(),
        "  struct with fields:".to_string(),
        String::new(),
    ];
    for (key, val) in map {
        let val_str = match val {
            Value::Struct(_) => "[1×1 struct]".to_string(),
            Value::StructArray(arr) => format!("[1×{} struct]", arr.len()),
            Value::Matrix(m) => format!("[{}×{} double]", m.nrows(), m.ncols()),
            Value::Cell(v) => format!("{{1×{} cell}}", v.len()),
            _ => format_value(val, Base::Dec, mode),
        };
        lines.push(format!("    {key}: {val_str}"));
    }
    lines.join("\n")
}

/// Formats a 1×N struct array (shows each element's fields).
fn format_struct_array(arr: &[IndexMap<String, Value>], mode: &FormatMode) -> String {
    let n = arr.len();
    let mut lines = vec![
        String::new(),
        format!("  1×{n} struct array with fields:"),
        String::new(),
    ];
    // Collect field names from the first element
    if let Some(first) = arr.first() {
        for key in first.keys() {
            lines.push(format!("    {key}"));
        }
    }
    // Show first element's values if array has exactly 1 element
    if n == 1
        && let Some(first) = arr.first()
    {
        lines.clear();
        lines.push(String::new());
        lines.push("  struct with fields:".to_string());
        lines.push(String::new());
        for (key, val) in first {
            let val_str = match val {
                Value::Struct(_) => "[1×1 struct]".to_string(),
                Value::StructArray(a) => format!("[1×{} struct]", a.len()),
                Value::Matrix(m) => format!("[{}×{} double]", m.nrows(), m.ncols()),
                Value::Cell(v) => format!("{{1×{} cell}}", v.len()),
                _ => format_value(val, Base::Dec, mode),
            };
            lines.push(format!("    {key}: {val_str}"));
        }
    }
    lines.join("\n")
}

fn format_datetime_array(v: &[f64]) -> String {
    let mut lines = Vec::with_capacity(v.len());
    for ts in v {
        lines.push(format!("  {}", crate::datetime::format_datetime(*ts)));
    }
    lines.join("\n")
}

fn format_duration_array(v: &[f64]) -> String {
    let mut lines = Vec::with_capacity(v.len());
    for secs in v {
        lines.push(format!("  {}", crate::datetime::format_duration(*secs)));
    }
    lines.join("\n")
}

/// Formats a matrix with right-aligned columns, 3-space indent, 3 spaces between columns.
/// `FormatMode::Plus` renders a sign grid (`+`, `-`, `0`).
fn format_matrix(m: &Array2<f64>, mode: &FormatMode) -> String {
    if m.nrows() == 0 || m.ncols() == 0 {
        return "   []".to_string();
    }
    // Special rendering for format +
    if matches!(mode, FormatMode::Plus) {
        let lines: Vec<String> = m
            .rows()
            .into_iter()
            .map(|row| {
                let chars: String = row
                    .iter()
                    .map(|&x| {
                        if x > 0.0 {
                            '+'
                        } else if x < 0.0 {
                            '-'
                        } else {
                            '0'
                        }
                    })
                    .collect();
                format!("   {}", chars)
            })
            .collect();
        return lines.join("\n");
    }
    let ncols = m.ncols();
    let cells: Vec<Vec<String>> = m
        .rows()
        .into_iter()
        .map(|row| row.iter().map(|&x| format_decimal(x, mode)).collect())
        .collect();
    let col_widths: Vec<usize> = (0..ncols)
        .map(|c| cells.iter().map(|row| row[c].len()).max().unwrap_or(0))
        .collect();
    let mut lines = Vec::new();
    for row in &cells {
        let mut line = String::from("   ");
        for (c, cell) in row.iter().enumerate() {
            if c > 0 {
                line.push_str("   ");
            }
            let pad = col_widths[c].saturating_sub(cell.len());
            for _ in 0..pad {
                line.push(' ');
            }
            line.push_str(cell);
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// Formats a number in a non-decimal integer base (hex/bin/oct).
/// Rounds to the nearest integer before formatting.
pub fn format_non_dec(n: f64, base: Base) -> String {
    let i = n.round() as i64;
    let u = i.unsigned_abs();
    let sign = if i < 0 { "-" } else { "" };
    match base {
        Base::Hex => format!("{}0x{:X}", sign, u),
        Base::Bin => format!("{}0b{:b}", sign, u),
        Base::Oct => format!("{}0o{:o}", sign, u),
        Base::Dec => format_decimal(n, &FormatMode::default()),
    }
}

// ---------------------------------------------------------------------------
// Internal decimal formatters
// ---------------------------------------------------------------------------

fn format_decimal(n: f64, mode: &FormatMode) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n > 0.0 { "Inf" } else { "-Inf" }.to_string();
    }
    match mode {
        FormatMode::Short | FormatMode::ShortG => fmt_auto_sig(n, 5),
        FormatMode::Long | FormatMode::LongG => fmt_auto_sig(n, 15),
        FormatMode::ShortE => fmt_sci_dp(n, 4),
        FormatMode::LongE => fmt_sci_dp(n, 14),
        FormatMode::Bank => format!("{:.2}", n),
        FormatMode::Rat => fmt_rat(n),
        FormatMode::Hex => fmt_hex_ieee754(n),
        FormatMode::Plus => fmt_plus_sign(n),
        FormatMode::Custom(prec) => fmt_custom_prec(n, *prec),
    }
}

/// Integer shortcut: fits in i64 without fractional part.
#[inline]
fn is_exact_int(n: f64) -> bool {
    n.fract() == 0.0 && n.abs() < 1e15
}

/// Auto fixed/scientific with `sig` significant digits (MATLAB-compatible).
/// Uses fixed notation for exponents in [-3, sig), scientific otherwise.
/// Integers are shown without a decimal point.
fn fmt_auto_sig(n: f64, sig: usize) -> String {
    if is_exact_int(n) {
        return format!("{}", n as i64);
    }
    let abs_n = n.abs();
    let exp = if abs_n == 0.0 {
        0i32
    } else {
        abs_n.log10().floor() as i32
    };
    if exp >= -3 && exp < sig as i32 {
        let dp = (sig as i32 - 1 - exp) as usize;
        let s = format!("{:.prec$}", n, prec = dp);
        // Only strip trailing zeros when there is a decimal point.
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    } else {
        let s = format!("{:.prec$e}", n, prec = sig - 1);
        trim_sci(&s)
    }
}

/// Always scientific notation with `dp` decimal places.
fn fmt_sci_dp(n: f64, dp: usize) -> String {
    let s = format!("{:.prec$e}", n, prec = dp);
    trim_sci(&s)
}

/// Legacy custom-precision: N decimal places, auto fixed/scientific.
fn fmt_custom_prec(n: f64, prec: usize) -> String {
    if is_exact_int(n) {
        return format!("{}", n as i64);
    }
    if n.abs() >= 1e15 || (n != 0.0 && n.abs() < 1e-9) {
        let s = format!("{:.prec$e}", n, prec = prec);
        trim_sci(&s)
    } else {
        let s = format!("{:.prec$}", n, prec = prec);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Rational approximation via continued fractions. Returns `"p/q"` or `"p"` if denominator is 1.
fn fmt_rat(n: f64) -> String {
    if is_exact_int(n) {
        return format!("{}", n as i64);
    }
    let sign = if n < 0.0 { -1i64 } else { 1i64 };
    let x = n.abs();
    let (mut h1, mut h2): (i64, i64) = (1, 0);
    let (mut k1, mut k2): (i64, i64) = (0, 1);
    let mut b = x;
    for _ in 0..64 {
        let a = b.floor() as i64;
        let (nh, nk) = (a * h1 + h2, a * k1 + k2);
        if nk > 10_000 {
            break;
        }
        h2 = h1;
        h1 = nh;
        k2 = k1;
        k1 = nk;
        let frac = b - a as f64;
        if frac < 1e-12 || (h1 as f64 / k1 as f64 - x).abs() < 1e-6 {
            break;
        }
        b = 1.0 / frac;
    }
    let p = sign * h1;
    if k1 == 1 {
        format!("{}", p)
    } else {
        format!("{}/{}", p, k1)
    }
}

/// IEEE 754 double-precision bit pattern as 16 uppercase hex digits.
fn fmt_hex_ieee754(n: f64) -> String {
    format!("{:016X}", n.to_bits())
}

/// Sign indicator: `+`, `-`, or ` ` for zero.
fn fmt_plus_sign(n: f64) -> String {
    if n > 0.0 {
        "+".to_string()
    } else if n < 0.0 {
        "-".to_string()
    } else {
        " ".to_string()
    }
}

fn trim_sci(s: &str) -> String {
    if let Some(e_pos) = s.find('e') {
        let mantissa = s[..e_pos].trim_end_matches('0').trim_end_matches('.');
        let exp_str = &s[e_pos + 1..];
        let (sign, digits) = if let Some(d) = exp_str.strip_prefix('-') {
            ("-", d)
        } else if let Some(d) = exp_str.strip_prefix('+') {
            ("+", d)
        } else {
            ("+", exp_str)
        };
        let exp_num: i32 = digits.parse().unwrap_or(0);
        format!("{}e{}{:02}", mantissa, sign, exp_num)
    } else {
        s.to_string()
    }
}

// --- MAT built-in helpers ---

/// Loads a MATLAB Level 5/7 MAT file and returns a [`Value::Struct`].
///
/// Requires the `mat` Cargo feature; without it, always returns an error.
pub fn load_mat_file(path: &str) -> Result<Value, String> {
    load_mat_file_impl(path)
}

#[cfg(feature = "mat")]
fn load_mat_file_impl(path: &str) -> Result<Value, String> {
    crate::mat::mat_load(path)
}

#[cfg(not(feature = "mat"))]
fn load_mat_file_impl(_path: &str) -> Result<Value, String> {
    Err("load: .mat support not available — rebuild with --features mat".to_string())
}

// --- Regex built-in helpers ---

#[cfg(feature = "regex")]
fn regexp_impl(
    fname: &str,
    s: &str,
    pat: &str,
    ignore_case: bool,
    return_match: bool,
) -> Result<Value, String> {
    use ndarray::Array2;
    let full_pat = if ignore_case {
        format!("(?i){pat}")
    } else {
        pat.to_string()
    };
    let re = regex::Regex::new(&full_pat).map_err(|e| format!("{fname}: invalid pattern: {e}"))?;
    if return_match {
        let matches: Vec<Value> = re
            .find_iter(s)
            .map(|m| Value::Str(m.as_str().to_string()))
            .collect();
        Ok(Value::Cell(matches))
    } else {
        match re.find(s) {
            Some(m) => Ok(Value::Scalar((s[..m.start()].chars().count() + 1) as f64)),
            None => Ok(Value::Matrix(Array2::zeros((0, 0)))),
        }
    }
}

#[cfg(not(feature = "regex"))]
fn regexp_impl(
    fname: &str,
    _s: &str,
    _pat: &str,
    _ignore_case: bool,
    _return_match: bool,
) -> Result<Value, String> {
    Err(format!(
        "{fname}: not available — rebuild with --features regex"
    ))
}

#[cfg(feature = "regex")]
fn regexprep_impl(s: &str, pat: &str, rep: &str) -> Result<Value, String> {
    let re = regex::Regex::new(pat).map_err(|e| format!("regexprep: invalid pattern: {e}"))?;
    let result = re.replace_all(s, regex::NoExpand(rep));
    Ok(Value::Str(result.into_owned()))
}

#[cfg(not(feature = "regex"))]
fn regexprep_impl(_s: &str, _pat: &str, _rep: &str) -> Result<Value, String> {
    Err("regexprep: not available — rebuild with --features regex".to_string())
}

// --- JSON built-in helpers ---

#[cfg(feature = "json")]
fn jsondecode_impl(arg: &Value) -> Result<Value, String> {
    let s = match arg {
        Value::Str(s) | Value::StringObj(s) => s.as_str(),
        _ => return Err("jsondecode: argument must be a string".to_string()),
    };
    let jval: serde_json::Value =
        serde_json::from_str(s).map_err(|e| format!("jsondecode: invalid JSON: {e}"))?;
    Ok(crate::json::json_to_value(&jval))
}

#[cfg(not(feature = "json"))]
fn jsondecode_impl(_arg: &Value) -> Result<Value, String> {
    Err("jsondecode: not available — rebuild with --features json".to_string())
}

#[cfg(feature = "json")]
fn jsonencode_impl(arg: &Value) -> Result<Value, String> {
    let jval = crate::json::value_to_json(arg)?;
    let s = serde_json::to_string(&jval)
        .map_err(|e| format!("jsonencode: serialization error: {e}"))?;
    Ok(Value::Str(s))
}

#[cfg(not(feature = "json"))]
fn jsonencode_impl(_arg: &Value) -> Result<Value, String> {
    Err("jsonencode: not available — rebuild with --features json".to_string())
}

// ---------------------------------------------------------------------------
// Phase 24 — Polynomial helpers
// ---------------------------------------------------------------------------

/// Evaluates a polynomial with real coefficients at a complex point using Horner's method.
fn cpoly_eval(coeffs: &[f64], z: (f64, f64)) -> (f64, f64) {
    let mut acc = (0.0_f64, 0.0_f64);
    for &c in coeffs {
        // acc = acc * z + c
        acc = (acc.0 * z.0 - acc.1 * z.1 + c, acc.0 * z.1 + acc.1 * z.0);
    }
    acc
}

/// Evaluates a polynomial at a real point using Horner's method.
fn horner(coeffs: &[f64], x: f64) -> f64 {
    coeffs.iter().fold(0.0, |acc, &c| acc * x + c)
}

/// Extracts polynomial (or 1-D knot) coefficients from a scalar or row/column vector `Value`.
fn poly_coeffs(v: &Value, fname: &str) -> Result<Vec<f64>, String> {
    match v {
        Value::Scalar(s) => Ok(vec![*s]),
        Value::Matrix(m) => {
            if m.nrows() == 1 {
                Ok(m.row(0).iter().copied().collect())
            } else if m.ncols() == 1 {
                Ok(m.column(0).iter().copied().collect())
            } else {
                Err(format!(
                    "{fname}: argument must be a vector, got {}×{}",
                    m.nrows(),
                    m.ncols()
                ))
            }
        }
        _ => Err(format!("{fname}: argument must be a real numeric vector")),
    }
}

/// Discrete linear convolution of two sequences. Result length = `a.len() + b.len() − 1`.
fn poly_conv(a: &[f64], b: &[f64]) -> Vec<f64> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let mut result = vec![0.0_f64; a.len() + b.len() - 1];
    for (i, &ai) in a.iter().enumerate() {
        for (j, &bj) in b.iter().enumerate() {
            result[i + j] += ai * bj;
        }
    }
    result
}

/// Polynomial long division `c / b` → `(quotient, remainder)`.
///
/// The remainder has the same length as `c` (MATLAB convention), satisfying
/// `conv(b, q) + r == c` element-wise.
fn poly_deconv(c: &[f64], b: &[f64]) -> Result<(Vec<f64>, Vec<f64>), String> {
    if b.is_empty() || b.iter().all(|&x| x == 0.0) {
        return Err("deconv: divisor polynomial must not be zero".to_string());
    }
    let mc = c.len();
    let mb = b.len();
    if mb > mc {
        return Ok((vec![0.0], c.to_vec()));
    }
    let q_len = mc - mb + 1;
    let mut remainder = c.to_vec();
    let mut q = vec![0.0_f64; q_len];
    for i in 0..q_len {
        let coeff = remainder[i] / b[0];
        q[i] = coeff;
        for j in 0..mb {
            remainder[i + j] -= coeff * b[j];
        }
    }
    // Zero out rounding residuals relative to the input scale
    let scale = c.iter().map(|v| v.abs()).fold(0.0_f64, f64::max).max(1.0);
    for x in &mut remainder {
        if x.abs() < 1e-10 * scale {
            *x = 0.0;
        }
    }
    Ok((q, remainder))
}

/// Finds all roots of `coeffs` (degree = `coeffs.len() − 1`) using the
/// Durand–Kerner (Weierstrass) iteration.
///
/// Returns roots as `(re, im)` pairs sorted by descending real part, then
/// descending imaginary part.
fn durand_kerner(coeffs: &[f64]) -> Result<Vec<(f64, f64)>, String> {
    let n = coeffs.len() - 1; // degree
    if n == 0 {
        return Ok(vec![]);
    }
    let lc = coeffs[0];
    if lc == 0.0 {
        return Err("roots: leading coefficient must not be zero".to_string());
    }
    // Normalize to monic polynomial
    let monic: Vec<f64> = coeffs.iter().map(|&c| c / lc).collect();

    // Cauchy root bound: all roots have |z| ≤ r
    let r = 1.0 + monic[1..].iter().map(|c| c.abs()).fold(0.0_f64, f64::max);

    // Initial guesses on a circle, rotated by 0.25/n turns to avoid the real axis
    // (a purely real start can stall for polynomials with purely imaginary roots).
    let mut z: Vec<(f64, f64)> = (0..n)
        .map(|k| {
            let angle = 2.0 * std::f64::consts::PI * (k as f64 + 0.25) / n as f64;
            (r * angle.cos(), r * angle.sin())
        })
        .collect();

    const MAX_ITER: usize = 2000;
    const EPS: f64 = 1e-12;

    for _ in 0..MAX_ITER {
        let z_old = z.clone();
        let mut max_corr = 0.0_f64;
        for i in 0..n {
            let (pre, pim) = cpoly_eval(&monic, z_old[i]);
            // denominator = Π_{j≠i}(z_i − z_j)
            let mut dre = 1.0_f64;
            let mut dim = 0.0_f64;
            for j in 0..n {
                if j == i {
                    continue;
                }
                let (dr, di) = (z_old[i].0 - z_old[j].0, z_old[i].1 - z_old[j].1);
                (dre, dim) = (dre * dr - dim * di, dre * di + dim * dr);
            }
            // correction = p(z_i) / denom
            let d2 = dre * dre + dim * dim;
            let (cre, cim) = if d2 > 0.0 {
                ((pre * dre + pim * dim) / d2, (pim * dre - pre * dim) / d2)
            } else {
                (pre, pim)
            };
            let corr_abs = (cre * cre + cim * cim).sqrt();
            max_corr = max_corr.max(corr_abs);
            z[i] = (z_old[i].0 - cre, z_old[i].1 - cim);
        }
        if max_corr < EPS {
            break;
        }
    }

    // Sort by descending real part, then descending imaginary part
    z.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
    });

    Ok(z)
}

/// Converts a list of complex roots into a `Value`.
///
/// Returns a real `Matrix` (column vector) when all imaginary parts are below
/// `1e-9`; otherwise returns a `Cell` of `Scalar`/`Complex` elements.
fn roots_to_value(roots: &[(f64, f64)]) -> Value {
    const IMAG_TOL: f64 = 1e-9;
    let all_real = roots.iter().all(|(_, im)| im.abs() < IMAG_TOL);
    if all_real {
        let data: Vec<f64> = roots.iter().map(|(re, _)| *re).collect();
        let n = data.len();
        Value::Matrix(Array2::from_shape_vec((n, 1), data).unwrap())
    } else {
        let vals: Vec<Value> = roots
            .iter()
            .map(|&(re, im)| {
                if im.abs() < IMAG_TOL {
                    Value::Scalar(re)
                } else {
                    Value::Complex(re, im)
                }
            })
            .collect();
        Value::Cell(vals)
    }
}

/// Computes the characteristic polynomial of a square matrix using the
/// Faddeev-LeVerrier algorithm.
///
/// Returns coefficients `[1, c_{n-1}, …, c_0]` in descending degree order.
fn characteristic_poly(a: &Array2<f64>) -> Result<Vec<f64>, String> {
    let n = a.nrows();
    if a.ncols() != n {
        return Err("poly: matrix must be square".to_string());
    }
    if n == 0 {
        return Ok(vec![1.0]);
    }
    let mut coeffs = vec![0.0_f64; n + 1];
    coeffs[0] = 1.0;
    let mut nk = Array2::<f64>::eye(n); // N_0 = I
    for (k, coeff) in coeffs.iter_mut().enumerate().skip(1) {
        let ank = a.dot(&nk); // A * N_{k-1}
        let tr: f64 = (0..n).map(|i| ank[[i, i]]).sum();
        let ak = -tr / k as f64;
        *coeff = ak;
        nk = ank; // N_k = A*N_{k-1} + a_k*I
        for i in 0..n {
            nk[[i, i]] += ak;
        }
    }
    Ok(coeffs)
}

/// Back-substitution solver for upper-triangular system `R * x = b`.
fn poly_back_sub(r: &Array2<f64>, b: &[f64]) -> Result<Vec<f64>, String> {
    let n = r.nrows();
    let mut x = vec![0.0_f64; n];
    for i in (0..n).rev() {
        let mut s = b[i];
        for j in (i + 1)..n {
            s -= r[[i, j]] * x[j];
        }
        if r[[i, i]].abs() < 1e-14 {
            return Err(
                "polyfit: Vandermonde matrix is rank-deficient; reduce polynomial degree"
                    .to_string(),
            );
        }
        x[i] = s / r[[i, i]];
    }
    Ok(x)
}

/// Evaluates `interp1` at a single query point `xi` using the given `method`.
///
/// Returns `NaN` for queries outside `[x[0], x[n-1]]`.
fn interp1_at(x: &[f64], y: &[f64], xi: f64, method: &str) -> f64 {
    let n = x.len();
    if xi < x[0] || xi > x[n - 1] {
        return f64::NAN;
    }
    // Index of the leftmost knot ≤ xi (in [0, n-1])
    let lo = x.partition_point(|&xk| xk <= xi).saturating_sub(1);
    // For methods that need a right neighbour, clamp to n-2
    let lo2 = lo.min(n - 2);
    match method {
        "nearest" => {
            if lo == n - 1 {
                return y[n - 1];
            }
            if (xi - x[lo2]) <= (x[lo2 + 1] - xi) {
                y[lo2]
            } else {
                y[lo2 + 1]
            }
        }
        "previous" => y[lo],
        "next" => {
            if lo == n - 1 || xi == x[lo] {
                y[lo]
            } else {
                y[lo2 + 1]
            }
        }
        _ => {
            // "linear" (default)
            if lo == n - 1 {
                return y[n - 1];
            }
            let t = (xi - x[lo2]) / (x[lo2 + 1] - x[lo2]);
            y[lo2] + t * (y[lo2 + 1] - y[lo2])
        }
    }
}

#[cfg(test)]
#[path = "eval_tests.rs"]
mod tests;
