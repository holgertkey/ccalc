# ccalc-engine

Core computation engine for [ccalc](https://github.com/holgertkey/ccalc) — a
terminal calculator with Octave/MATLAB syntax.

This crate provides the complete language pipeline: tokenizer → parser → AST →
evaluator, plus the block executor that handles control flow, user-defined
functions, and the session search path. It is designed to be embedded in any
host application that needs a scriptable MATLAB-dialect expression engine.

```text
input string
    └─► tokenizer   (parser::tokenize)
            └─► recursive-descent parser   (parser::parse)  →  Stmt / Expr AST
                        └─► evaluator      (eval::eval)     →  Value
                                └─► block executor  (exec::exec_stmts)
```

---

## Features

| Feature                  | Description                                                                             |
|--------------------------|-----------------------------------------------------------------------------------------|
| Numeric types            | `f64` scalars, 2-D real matrices (via `ndarray`), complex numbers                       |
| MATLAB-compatible syntax | Operators, ranges `1:5`, matrix literals `[1 2; 3 4]`, element-wise `.*`                |
| Control flow             | `if/elseif/else`, `for`, `while`, `do/until`, `switch/case`, `break/continue`, `return` |
| User functions           | Named `function` definitions, multiple return values, `@(x)` lambdas, closures          |
| Scoping                  | `global` and `persistent` variables, `private/` directory isolation, `+pkg/` namespaces |
| String types             | Single-quoted char arrays and double-quoted string objects                              |
| Cell arrays              | `{1, 'hi', [1 2 3]}` heterogeneous containers, `varargin`/`varargout`                   |
| Structs                  | Scalar structs, struct arrays, nested field access `s.a.b`                              |
| File I/O                 | `fopen`/`fclose`/`fgetl`/`fgets`, `dlmread`/`dlmwrite`, `isfile`/`pwd`                  |
| Formatted output         | `fprintf`/`sprintf` with full C `printf` specifier support                              |
| Error handling           | `try`/`catch`, `error()`, `warning()`, `pcall()`, `lasterr()`                           |
| Autoload                 | Calling an unknown name searches `<name>.calc` / `<name>.m` on the path                 |
| Packages                 | `+pkg/` namespace directories; `pkg.func(args)` call syntax                             |
| Number bases             | Decimal, hex, binary, octal output; `0xFF` / `0b1010` / `0o17` literals                 |
| 160+ built-ins           | Math, matrix, string, I/O, filesystem, bitwise, statistics                              |

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ccalc-engine = "0.21"
```

### Optional: BLAS acceleration

Matrix multiplication (`A * B`) can be accelerated via OpenBLAS. `inv`/`det`
always use pure Rust (no BLAS required).

```toml
# Dynamic linkage — requires system-installed OpenBLAS
ccalc-engine = { version = "0.21", features = ["blas"] }

# Static linkage — compiles OpenBLAS from source (requires gfortran + cmake)
ccalc-engine = { version = "0.21", features = ["blas-static"] }
```

---

## Quick start

```rust
use ccalc_engine::{env, eval, exec, parser, io};

fn main() {
    // One-time initialisation — registers the function call and autoload hooks.
    exec::init();

    // Build the variable environment.
    let mut env = env::new_env();
    let mut io  = io::IoContext::new();
    let     fmt = eval::FormatMode::Short;
    let     base = eval::Base::Dec;

    // Parse a multi-statement block and execute it.
    let src = "
        x = 3;
        y = 4;
        hyp = sqrt(x^2 + y^2)
    ";
    let stmts = parser::parse_stmts(src).unwrap();
    exec::exec_stmts(&stmts, &mut env, &mut io, &fmt, base, false).unwrap();

    // Read a result from the environment.
    if let Some(val) = env.get("hyp") {
        println!("hyp = {}", eval::format_value(val, base, &fmt));
        // hyp = 5
    }
}
```

### Single expression

```rust
use ccalc_engine::{env, eval, exec, parser};

exec::init();
let env = env::new_env();

let expr = parser::parse("sin(pi/6)^2 + cos(pi/6)^2").unwrap();
// parser::parse returns a Stmt; extract the Expr from Stmt::Expr
let val = match expr {
    parser::Stmt::Expr(e) => eval::eval(&e, &env).unwrap(),
    _ => panic!("expected expression"),
};
println!("{val:?}"); // Scalar(1.0)
```

### User-defined functions

```rust
use ccalc_engine::{env, eval, exec, parser, io};

exec::init();
let mut env = env::new_env();
let mut io  = io::IoContext::new();

let src = "
    function y = square(x)
      y = x ^ 2;
    end

    result = square(7)
";
let stmts = parser::parse_stmts(src).unwrap();
exec::exec_stmts(&stmts, &mut env, &mut io,
    &eval::FormatMode::Short, eval::Base::Dec, false).unwrap();

// env["result"] == Scalar(49.0)
```

---

## API overview

### `env` module

```rust
pub fn new_env() -> Env   // creates a fresh environment seeded with i, j = 0+1i

pub enum Value {
    Void,
    Scalar(f64),
    Matrix(Array2<f64>),
    Complex(f64, f64),
    Str(String),           // single-quoted char array
    StringObj(String),     // double-quoted string object
    Lambda(LambdaFn),      // @(x) expr closure
    Function { outputs, params, body_source, locals },
    Tuple(Vec<Value>),     // internal multi-return (consumed by MultiAssign)
    Cell(Vec<Value>),      // {1, 'hi', [1 2 3]}
    Struct(IndexMap<String, Value>),
    StructArray(Vec<IndexMap<String, Value>>),
}
```

### `parser` module

```rust
// Parse a single statement (expression or assignment).
pub fn parse(input: &str) -> Result<Stmt, String>

// Parse a full multi-line script into a statement list.
// Each tuple is (statement, is_silent) where is_silent == true means
// the statement ended with ';' and should not print output.
pub fn parse_stmts(input: &str) -> Result<Vec<(Stmt, bool)>, String>

// REPL helper: returns the net block depth delta for one line.
// +1 for 'if'/'for'/'while'/'function'/..., -1 for 'end'/'until'.
pub fn block_depth_delta(line: &str) -> i32

// Returns true if a line is a self-contained single-line block:
// 'if cond; body; end' — no buffering needed in the REPL.
pub fn is_single_line_block(line: &str) -> bool
```

### `eval` module

```rust
// Evaluate a parsed expression.
pub fn eval(expr: &Expr, env: &Env) -> Result<Value, String>

// Evaluate with file I/O support (required for fprintf, fopen, etc.).
pub fn eval_with_io(expr: &Expr, env: &Env, io: &mut IoContext) -> Result<Value, String>

// Number formatting.
pub fn format_value(v: &Value, base: Base, mode: &FormatMode) -> String
pub fn format_value_full(v: &Value, mode: &FormatMode) -> Option<String>
pub fn format_scalar(n: f64, base: Base, mode: &FormatMode) -> String
pub fn format_complex(re: f64, im: f64, mode: &FormatMode) -> String

// C printf-style formatting engine used by fprintf/sprintf.
pub fn format_printf(fmt: &str, args: &[Value]) -> Result<String, String>

#[derive(Clone, Copy, PartialEq)]
pub enum Base { Dec, Hex, Bin, Oct }

#[derive(Clone)]
pub enum FormatMode {
    Short, Long, ShortE, LongE, ShortG, Bank, Rat, Hex, Sign,
    Compact, Loose,
    Decimals(usize),   // format N
}
```

### `exec` module

```rust
// Must be called once at startup before any user function can be invoked.
pub fn exec_stmts(
    stmts: &[(Stmt, bool)],
    env: &mut Env,
    io: &mut IoContext,
    fmt: &FormatMode,
    base: Base,
    compact: bool,
) -> Result<Option<Signal>, String>

pub enum Signal { Break, Continue, Return }

// Search path management (mirrors MATLAB addpath/rmpath).
pub fn session_path_init(paths: Vec<PathBuf>)
pub fn session_path_add(path: PathBuf, append: bool)
pub fn session_path_remove(path: &Path)
pub fn session_path_list() -> Vec<PathBuf>

// Script directory stack — push before run(), pop after.
pub fn script_dir_push(dir: &Path)
pub fn script_dir_pop()

// Resolve a script filename to an existing path (CWD → script dir → session path).
pub fn resolve_script_path(name: &str) -> Option<PathBuf>
```

---

## Architecture

```
ccalc-engine/src/
├── lib.rs       crate root, module re-exports
├── env.rs       Value enum, Env type, workspace save/load
├── eval.rs      Expr/Op AST, eval_inner, call_builtin (160+ built-ins),
│                global/persistent variable stores, display thread-locals,
│                autoload hook, format_* helpers
├── parser.rs    tokenize → recursive-descent parse → Stmt/Expr AST
├── exec.rs      exec_stmts, call_user_function, try_autoload,
│                session path, script dir stack, body parse cache
└── io.rs        IoContext — file descriptor table for fopen/fclose/fgetl/fgets
```

### Thread model

All state is **thread-local**. Each OS thread has its own independent
environment, global/persistent stores, and search path. This makes the engine
safe to embed in multi-threaded applications as long as each thread manages its
own environment.

### Hook pattern

`eval.rs` and `exec.rs` are decoupled via function-pointer hooks to avoid a
circular dependency:

- `FnCallHook` — called by `eval_inner` when a `Value::Function` is invoked;
  implemented by `exec::call_user_function`.
- `AutoloadHook` — called by `eval_inner` when a name is unknown; implemented
  by `exec::try_autoload`.

Both hooks are registered by `exec::init()` which must be called before any
user function or autoload can work.

---

## Scoping

| Mechanism      | Description                                                        |
|----------------|--------------------------------------------------------------------|
| `global x`     | Shared across all functions and the base workspace that declare it |
| `persistent x` | Per-function value that survives between calls; `[]` on first call |
| `private/`     | Functions in `private/` are visible only to the parent directory   |
| `+pkg/`        | Package namespace; `pkg.func(args)` searches `+pkg/func.calc`      |

Persistent variables use write-through semantics: `Stmt::IndexSet` and
`Stmt::Assign` write to `PERSISTENT_STORE` immediately so recursive callers
see updates without waiting for the current frame to return. This is what
makes memoization (e.g. Fibonacci with a persistent cache) correct and O(n)
rather than O(2ⁿ).

---

## Embedding example — minimal REPL

```rust
use ccalc_engine::{env, eval, exec, parser, io};

fn main() {
    exec::init();
    let mut env  = env::new_env();
    let mut io   = io::IoContext::new();
    let     fmt  = eval::FormatMode::Short;
    let     base = eval::Base::Dec;

    let mut input = String::new();
    loop {
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let line = input.trim();
        if line == "exit" { break; }

        match parser::parse_stmts(line) {
            Ok(stmts) => {
                match exec::exec_stmts(&stmts, &mut env, &mut io, &fmt, base, false) {
                    Ok(_) => {}
                    Err(e) => eprintln!("Error: {e}"),
                }
            }
            Err(e) => eprintln!("Parse error: {e}"),
        }
    }
}
```

---

## Running the benchmarks

```bash
cargo bench
```

Benchmarks live in `benches/engine.rs` and use [Criterion](https://github.com/bheisler/criterion.rs):

| Benchmark                    | What it measures                                        |
|------------------------------|---------------------------------------------------------|
| `scalar_ops_sum_1M`          | `sum(1:1000000)` — range + reduction                    |
| `fib/fib_30`                 | Recursive Fibonacci(30) — deep function call overhead   |
| `loop_10k`                   | `for k=1:10000; s+=k; end` — loop + compound assignment |
| `matmul/100` … `matmul/1000` | `ones(N,N) * ones(N,N)` — matrix multiply               |
| `fn_calls_1000`              | 1000 calls to a trivial named function                  |

---

## License

MIT — same as the parent [ccalc](https://github.com/holgertkey/ccalc) workspace.
