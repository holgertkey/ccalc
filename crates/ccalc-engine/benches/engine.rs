//! Criterion benchmark suite for ccalc-engine.
//!
//! Run with:   cargo bench
//! One bench:  cargo bench --bench engine -- scalar_arithmetic
//!
//! HTML report is written to target/criterion/

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

use ccalc_engine::env::{Env, Value};
use ccalc_engine::eval::{Base, FormatMode};
use ccalc_engine::exec::{self, exec_stmts};
use ccalc_engine::io::IoContext;
use ccalc_engine::parser::parse_stmts;

// ── helpers ──────────────────────────────────────────────────────────────────

fn new_env() -> Env {
    let mut env = Env::new();
    env.insert("ans".to_string(), Value::Scalar(0.0));
    env.insert("i".to_string(), Value::Complex(0.0, 1.0));
    env.insert("j".to_string(), Value::Complex(0.0, 1.0));
    env
}

/// Parse and execute a code snippet, panicking on any error.
fn run(code: &str, env: &mut Env) {
    let stmts = parse_stmts(code).unwrap_or_else(|e| panic!("parse failed: {e}\ncode: {code}"));
    let mut io = IoContext::new();
    exec_stmts(&stmts, env, &mut io, &FormatMode::Short, Base::Dec, true)
        .unwrap_or_else(|e| panic!("exec failed: {e}\ncode: {code}"));
}

/// Execute a pre-parsed statement list, panicking on error. Used inside `b.iter()`.
fn exec_checked(stmts: &[(ccalc_engine::parser::Stmt, bool)], env: &mut Env, io: &mut IoContext) {
    exec_stmts(stmts, env, io, &FormatMode::Short, Base::Dec, true)
        .unwrap_or_else(|e| panic!("bench exec failed: {e}"));
}

// ── 1: scalar arithmetic throughput ─────────────────────────────────────────

/// sum(1:1_000_000) — range construction + 1 M additions inside the sum builtin.
fn bench_scalar_ops(c: &mut Criterion) {
    let stmts = parse_stmts("sum(1:1000000)").expect("parse");
    let mut env = new_env();
    let mut io = IoContext::new();

    c.bench_function("scalar_ops_sum_1M", |b| {
        b.iter(|| exec_checked(black_box(&stmts), &mut env, &mut io))
    });
}

// ── 2: recursive fib(30) ─────────────────────────────────────────────────────

/// fib(30) via naive recursive user-defined function — exercises function call
/// overhead and the interpreter body cache.
///
/// fib(30) = 832040, requiring ~2.7 M recursive interpreter calls.
/// Expect ~7 s per iteration on a typical machine; sample_size is set low accordingly.
fn bench_fib(c: &mut Criterion) {
    exec::init(); // register the user-function call hook

    let fib_def = "\
function result = fib(n)
  if n <= 1
    result = n;
  else
    result = fib(n-1) + fib(n-2);
  end
end";

    let mut env = new_env();
    run(fib_def, &mut env);

    let stmts = parse_stmts("fib(30)").expect("parse");
    let mut io = IoContext::new();

    let mut group = c.benchmark_group("fib");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(90));
    group.bench_function("fib_30", |b| {
        b.iter(|| exec_checked(black_box(&stmts), &mut env, &mut io))
    });
    group.finish();
}

// ── 3: interpreter loop throughput ───────────────────────────────────────────

/// for k=1:10000; s+=k; end — measures REPL/exec loop overhead at 10 K iterations.
fn bench_loop_throughput(c: &mut Criterion) {
    let stmts = parse_stmts("s = 0\nfor k = 1:10000\n  s += k\nend").expect("parse");
    let mut env = new_env();
    let mut io = IoContext::new();

    c.bench_function("loop_10k", |b| {
        b.iter(|| exec_checked(black_box(&stmts), &mut env, &mut io))
    });
}

// ── 4: matrix multiply ────────────────────────────────────────────────────────

/// A * A for A = ones(N, N) at N ∈ {100, 500, 1000}.
fn bench_matmul(c: &mut Criterion) {
    let mut group = c.benchmark_group("matmul");
    // Large matrices need more time; keep sample count manageable.
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(20);

    for &n in &[100usize, 500, 1000] {
        let mut env = new_env();
        let mut io = IoContext::new();
        run(&format!("A = ones({n}, {n});"), &mut env);

        let stmts = parse_stmts("A * A").expect("parse");

        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, _| {
            b.iter(|| exec_checked(black_box(&stmts), &mut env, &mut io))
        });
    }

    group.finish();
}

// ── 5: function call overhead ─────────────────────────────────────────────────

/// 1 000 calls to a trivial 1-line function — isolates per-call overhead.
fn bench_fn_calls(c: &mut Criterion) {
    exec::init(); // register the user-function call hook

    let fn_def = "\
function y = inc(x)
  y = x + 1;
end";

    let mut env = new_env();
    run(fn_def, &mut env);

    let stmts =
        parse_stmts("s = 0\nfor k = 1:1000\n  s = inc(k)\nend").expect("parse");
    let mut io = IoContext::new();

    c.bench_function("fn_calls_1000", |b| {
        b.iter(|| exec_checked(black_box(&stmts), &mut env, &mut io))
    });
}

// ── Criterion entry points ────────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_scalar_ops,
    bench_fib,
    bench_loop_throughput,
    bench_matmul,
    bench_fn_calls,
);
criterion_main!(benches);
