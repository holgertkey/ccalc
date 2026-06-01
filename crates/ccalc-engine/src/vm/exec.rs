//! Bytecode VM executor.
//!
//! [`vm_exec`] runs a compiled [`Chunk`] against a mutable [`Env`], producing
//! identical observable behaviour to the tree-walking interpreter while
//! eliminating per-statement `exec_stmts` call overhead.

use ndarray::Array2;
use num_complex::Complex;

use crate::env::{Env, Value};
use crate::eval::{
    Base, Expr, FormatMode, Op, eval_with_io,
    is_global, is_persistent, global_set, persistent_save, current_func_name,
    set_display_ctx,
};
use crate::io::IoContext;
use crate::exec::{Signal, print_value};
use super::{Chunk, IterState, Opcode};

// ── Entry point ───────────────────────────────────────────────────────────────

/// Execute a compiled [`Chunk`] against `env`.
///
/// Returns `Ok(None)` on normal completion, `Ok(Some(Signal::*))` when a
/// `break`/`continue`/`return` escapes the block, or `Err(msg)` on runtime
/// error.  The result is identical to what `exec_stmts` would have produced for
/// the same statement block.
pub fn vm_exec(
    chunk: &Chunk,
    env:     &mut Env,
    io:      &mut IoContext,
    fmt:     &FormatMode,
    base:    Base,
    compact: bool,
) -> Result<Option<Signal>, String> {
    // Propagate display settings to eval.rs so EvalExpr / fn-call paths work.
    set_display_ctx(fmt, base, compact);

    let mut stack: Vec<Value> = Vec::with_capacity(8);
    let mut iters: Vec<IterState> = Vec::new();
    let mut ip: usize = 0;

    // Annotate a runtime error with a source line number if one is recorded.
    macro_rules! at_line {
        ($result:expr) => {
            $result.map_err(|e: String| {
                let line = chunk.lines.get(ip).copied().unwrap_or(0);
                if line == 0 || e.contains("near line") {
                    e
                } else {
                    format!("{e} near line {line}")
                }
            })?
        };
    }

    while ip < chunk.code.len() {
        let instr = &chunk.code[ip];
        match instr.op {
            // ── Constants & variables ─────────────────────────────────────────
            Opcode::PushConst => {
                let v = chunk.consts[instr.u16_arg() as usize].clone();
                stack.push(v);
                ip += 1;
            }

            Opcode::LoadVar => {
                let name = &chunk.names[instr.u16_arg() as usize];
                // Fast path: variable already in env (covers most iterations).
                let v = if let Some(existing) = env.get(name.as_str()) {
                    existing.clone()
                } else {
                    // Slow path: may be a built-in constant (pi, e, inf, nan, …).
                    at_line!(eval_with_io(&Expr::Var(name.clone()), env, io))
                };
                stack.push(v);
                ip += 1;
            }

            Opcode::StoreVar => {
                let name_idx = instr.u16_at(0) as usize;
                let silent   = instr.u8_at(2) != 0;
                let val = stack.pop().unwrap();
                let name = &chunk.names[name_idx];
                env.insert(name.clone(), val.clone());
                // Mirror to the global store when declared global in this scope.
                if is_global(name) {
                    global_set(name, val.clone());
                }
                // Write-through for persistent vars.
                if is_persistent(name) {
                    persistent_save(&current_func_name(), name, val.clone());
                }
                if !silent && !matches!(val, Value::Void) {
                    print_value(Some(name), &val, fmt, base, compact);
                }
                ip += 1;
            }

            Opcode::UpdateAns => {
                // Peek (no pop) — ans is updated even for silent expr stmts.
                let v = stack.last().unwrap().clone();
                env.insert("ans".to_string(), v);
                ip += 1;
            }

            Opcode::Pop => {
                stack.pop();
                ip += 1;
            }

            Opcode::Print => {
                let v = stack.pop().unwrap();
                if !matches!(v, Value::Void) {
                    print_value(None, &v, fmt, base, compact);
                }
                ip += 1;
            }

            // ── Arithmetic ────────────────────────────────────────────────────
            Opcode::Add => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::Add, b, env, io)));
                ip += 1;
            }
            Opcode::Sub => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::Sub, b, env, io)));
                ip += 1;
            }
            Opcode::Mul => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::Mul, b, env, io)));
                ip += 1;
            }
            Opcode::Div => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::Div, b, env, io)));
                ip += 1;
            }
            Opcode::Pow => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::Pow, b, env, io)));
                ip += 1;
            }
            Opcode::ElemMul => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::ElemMul, b, env, io)));
                ip += 1;
            }
            Opcode::ElemDiv => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::ElemDiv, b, env, io)));
                ip += 1;
            }
            Opcode::ElemPow => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::ElemPow, b, env, io)));
                ip += 1;
            }
            Opcode::Neg => {
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_neg(a, env, io)));
                ip += 1;
            }

            // ── Comparison / logical ──────────────────────────────────────────
            Opcode::Eq => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::Eq, b, env, io)));
                ip += 1;
            }
            Opcode::Ne => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::NotEq, b, env, io)));
                ip += 1;
            }
            Opcode::Lt => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::Lt, b, env, io)));
                ip += 1;
            }
            Opcode::Le => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::LtEq, b, env, io)));
                ip += 1;
            }
            Opcode::Gt => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::Gt, b, env, io)));
                ip += 1;
            }
            Opcode::Ge => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::GtEq, b, env, io)));
                ip += 1;
            }
            Opcode::And => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::And, b, env, io)));
                ip += 1;
            }
            Opcode::Or => {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(at_line!(vm_binop(a, Op::Or, b, env, io)));
                ip += 1;
            }
            Opcode::Not => {
                let a = stack.pop().unwrap();
                let result = match &a {
                    Value::Scalar(n) => Value::Scalar(if *n == 0.0 { 1.0 } else { 0.0 }),
                    _ => {
                        // Fall back via eval_with_io for complex / matrix NOT.
                        let idx = env.len(); // use len as a unique key to avoid collision
                        let tmp_key = format!("__vm_not_{idx}__");
                        env.insert(tmp_key.clone(), a);
                        let result = eval_with_io(
                            &Expr::UnaryNot(Box::new(Expr::Var(tmp_key.clone()))),
                            env, io,
                        )?;
                        env.remove(&tmp_key);
                        result
                    }
                };
                stack.push(result);
                ip += 1;
            }

            // ── Control flow ──────────────────────────────────────────────────
            Opcode::Jump => {
                let off = instr.i32_arg();
                ip = (ip as isize + 1 + off as isize) as usize;
            }
            Opcode::JumpFalsy => {
                let off = instr.i32_arg();
                let v = stack.pop().unwrap();
                if !is_truthy(&v) {
                    ip = (ip as isize + 1 + off as isize) as usize;
                } else {
                    ip += 1;
                }
            }
            Opcode::JumpTruthy => {
                let off = instr.i32_arg();
                let v = stack.pop().unwrap();
                if is_truthy(&v) {
                    ip = (ip as isize + 1 + off as isize) as usize;
                } else {
                    ip += 1;
                }
            }

            // ── For-loop iteration ────────────────────────────────────────────
            Opcode::PushIter => {
                let range_val = stack.pop().unwrap();
                iters.push(at_line!(IterState::from_value(range_val)));
                ip += 1;
            }
            Opcode::IterNext => {
                let var_idx  = instr.u16_at(0) as usize;
                let exit_off = instr.i32_at(2);
                match iters.last_mut().unwrap().next_val() {
                    Some(val) => {
                        env.insert(chunk.names[var_idx].clone(), val);
                        ip += 1;
                    }
                    None => {
                        iters.pop();
                        ip = (ip as isize + 1 + exit_off as isize) as usize;
                    }
                }
            }
            Opcode::PopIter => {
                iters.pop();
                ip += 1;
            }

            // ── Deferred expression evaluation ────────────────────────────────
            Opcode::EvalExpr => {
                let expr_idx = instr.u16_arg() as usize;
                let val = at_line!(eval_with_io(&chunk.exprs[expr_idx], env, io));
                stack.push(val);
                ip += 1;
            }

            // ── Indexed assignment: A(i, j) = v ──────────────────────────────
            Opcode::IndexSetOp => {
                let name_idx = instr.u16_at(0) as usize;
                let iset_idx = instr.u16_at(2) as usize;
                let silent   = instr.u8_at(4) != 0;
                let rhs      = stack.pop().unwrap();
                let name     = &chunk.names[name_idx];
                let indices  = &chunk.index_sets[iset_idx];

                // Refresh persistent var before partial update.
                if is_persistent(name) {
                    let func = current_func_name();
                    if let Some(fresh) = crate::eval::persistent_load(&func, name) {
                        env.insert(name.clone(), fresh);
                    }
                }

                at_line!(crate::exec::exec_index_set(name, indices, rhs, env, io));

                // Write-through for persistent vars.
                if is_persistent(name) {
                    if let Some(val) = env.get(name) {
                        crate::eval::persistent_save(&current_func_name(), name, val.clone());
                    }
                }
                if is_global(name) {
                    if let Some(val) = env.get(name) {
                        global_set(name, val.clone());
                    }
                }
                if !silent {
                    if let Some(val) = env.get(name) {
                        print_value(Some(name), val, fmt, base, compact);
                    }
                }
                ip += 1;
            }

            // ── Function definition ───────────────────────────────────────────
            Opcode::DefineFunc => {
                let name_idx  = instr.u16_at(0) as usize;
                let const_idx = instr.u16_at(2) as usize;
                let func = chunk.consts[const_idx].clone();
                env.insert(chunk.names[name_idx].clone(), func);
                ip += 1;
            }

            // ── Function control ──────────────────────────────────────────────
            Opcode::Return => {
                return Ok(Some(Signal::Return));
            }
        }
    }
    Ok(None)
}

// ── Arithmetic helpers ────────────────────────────────────────────────────────

/// Evaluate a binary operation on two [`Value`]s.
///
/// Fast path for `Scalar op Scalar`; falls back to `eval_with_io` for all
/// other type combinations to reuse the existing semantics without duplication.
fn vm_binop(a: Value, op: Op, b: Value, env: &mut Env, io: &mut IoContext) -> Result<Value, String> {
    // ── Scalar × Scalar fast path ─────────────────────────────────────────────
    if let (Value::Scalar(fa), Value::Scalar(fb)) = (&a, &b) {
        let fa = *fa;
        let fb = *fb;
        let result = match op {
            Op::Add   => fa + fb,
            Op::Sub   => fa - fb,
            Op::Mul   => fa * fb,
            Op::Div   => fa / fb,
            Op::Pow | Op::ElemPow => fa.powf(fb),
            Op::ElemMul => fa * fb,
            Op::ElemDiv => fa / fb,
            Op::Eq    => if fa == fb { 1.0 } else { 0.0 },
            Op::NotEq => if fa != fb { 1.0 } else { 0.0 },
            Op::Lt    => if fa < fb  { 1.0 } else { 0.0 },
            Op::LtEq  => if fa <= fb { 1.0 } else { 0.0 },
            Op::Gt    => if fa > fb  { 1.0 } else { 0.0 },
            Op::GtEq  => if fa >= fb { 1.0 } else { 0.0 },
            Op::And    => if fa != 0.0 && fb != 0.0 { 1.0 } else { 0.0 },
            Op::Or     => if fa != 0.0 || fb != 0.0 { 1.0 } else { 0.0 },
            Op::ElemAnd => if fa != 0.0 && fb != 0.0 { 1.0 } else { 0.0 },
            Op::ElemOr  => if fa != 0.0 || fb != 0.0 { 1.0 } else { 0.0 },
            Op::LDiv    => fb / fa,
        };
        return Ok(Value::Scalar(result));
    }

    // ── Complex × Complex / Scalar×Complex fast path ──────────────────────────
    if matches!((&a, &b), (Value::Complex(..) | Value::Scalar(_), Value::Complex(..) | Value::Scalar(_))) {
        let (are, aim) = to_complex_parts(&a);
        let (bre, bim) = to_complex_parts(&b);
        let result: Option<(f64, f64)> = match op {
            Op::Add    => Some((are + bre, aim + bim)),
            Op::Sub    => Some((are - bre, aim - bim)),
            Op::Mul | Op::ElemMul => Some((are * bre - aim * bim, are * bim + aim * bre)),
            Op::Div | Op::ElemDiv => {
                let denom = bre * bre + bim * bim;
                Some(((are * bre + aim * bim) / denom, (aim * bre - are * bim) / denom))
            }
            Op::Pow | Op::ElemPow => {
                // Use num_complex for correct branch-cut behaviour.
                let base = Complex::new(are, aim);
                let r = if bim == 0.0 {
                    // Real exponent — use powi for integer exponents (exact, no FP error).
                    if bre.fract() == 0.0 && bre.abs() <= i32::MAX as f64 {
                        base.powi(bre as i32)
                    } else {
                        base.powf(bre)
                    }
                } else {
                    base.powc(Complex::new(bre, bim))
                };
                Some((r.re, r.im))
            }
            Op::Eq     => return Ok(Value::Scalar(if are == bre && aim == bim { 1.0 } else { 0.0 })),
            Op::NotEq  => return Ok(Value::Scalar(if are != bre || aim != bim { 1.0 } else { 0.0 })),
            _ => None,
        };
        if let Some((re, im)) = result {
            return Ok(if im == 0.0 { Value::Scalar(re) } else { Value::Complex(re, im) });
        }
    }

    // ── Scalar × Matrix / Matrix × Scalar broadcast ───────────────────────────
    if let (Value::Scalar(s), Value::Matrix(m)) | (Value::Matrix(m), Value::Scalar(s)) = (&a, &b) {
        let s = *s;
        let result_mat = match op {
            Op::Add | Op::ElemMul => {
                // commutative
                if matches!(a, Value::Scalar(_)) {
                    m.mapv(|x| match op { Op::Add => s + x, _ => s * x })
                } else {
                    m.mapv(|x| match op { Op::Add => x + s, _ => x * s })
                }
            }
            Op::Sub => {
                if matches!(a, Value::Scalar(_)) {
                    m.mapv(|x| s - x)
                } else {
                    m.mapv(|x| x - s)
                }
            }
            Op::Mul => {
                // Scalar * Matrix or Matrix * Scalar → broadcast
                m.mapv(|x| x * s)
            }
            Op::Div => {
                if matches!(a, Value::Scalar(_)) {
                    m.mapv(|x| s / x)
                } else {
                    m.mapv(|x| x / s)
                }
            }
            Op::ElemDiv => {
                if matches!(a, Value::Scalar(_)) {
                    m.mapv(|x| s / x)
                } else {
                    m.mapv(|x| x / s)
                }
            }
            Op::Pow | Op::ElemPow => {
                if matches!(a, Value::Scalar(_)) {
                    m.mapv(|x| s.powf(x))
                } else {
                    m.mapv(|x| x.powf(s))
                }
            }
            _ => {
                // Comparison: element-wise
                m.mapv(|x| {
                    let (fa, fb) = if matches!(a, Value::Scalar(_)) { (s, x) } else { (x, s) };
                    match op {
                        Op::Eq   => if fa == fb { 1.0 } else { 0.0 },
                        Op::NotEq=> if fa != fb { 1.0 } else { 0.0 },
                        Op::Lt   => if fa < fb  { 1.0 } else { 0.0 },
                        Op::LtEq => if fa <= fb { 1.0 } else { 0.0 },
                        Op::Gt   => if fa > fb  { 1.0 } else { 0.0 },
                        Op::GtEq => if fa >= fb { 1.0 } else { 0.0 },
                        Op::And  => if fa != 0.0 && fb != 0.0 { 1.0 } else { 0.0 },
                        Op::Or   => if fa != 0.0 || fb != 0.0 { 1.0 } else { 0.0 },
                        _ => unreachable!(),
                    }
                })
            }
        };
        return Ok(Value::Matrix(result_mat));
    }

    // ── Matrix × Matrix ────────────────────────────────────────────────────────
    if let (Value::Matrix(ma), Value::Matrix(mb)) = (&a, &b) {
        match op {
            Op::Add    => return Ok(Value::Matrix(ma + mb)),
            Op::Sub    => return Ok(Value::Matrix(ma - mb)),
            Op::ElemMul => return Ok(Value::Matrix(ma * mb)),
            Op::ElemDiv => return Ok(Value::Matrix(ma / mb)),
            Op::ElemPow => return Ok(Value::Matrix(ma.mapv(|_| 0.0)
                // element-wise pow — mapv over zip
                // ndarray doesn't have zip_mapv, use Zip
            )),
            Op::Mul => {
                // Matrix product
                if ma.ncols() != mb.nrows() {
                    return Err(format!(
                        "Matrix dimensions mismatch for *: {}×{} vs {}×{}",
                        ma.nrows(), ma.ncols(), mb.nrows(), mb.ncols()
                    ));
                }
                return Ok(Value::Matrix(ma.dot(mb)));
            }
            _ => {} // fall through to eval_with_io for other ops
        }
        // Element-wise pow needs special handling (zip).
        if matches!(op, Op::ElemPow) {
            let mut result = Array2::<f64>::zeros(ma.raw_dim());
            for ((r, &av), &bv) in result.iter_mut().zip(ma.iter()).zip(mb.iter()) {
                *r = av.powf(bv);
            }
            return Ok(Value::Matrix(result));
        }
        // Comparison element-wise.
        let result = ndarray::Zip::from(ma).and(mb).map_collect(|&av, &bv| {
            match op {
                Op::Eq    => if av == bv { 1.0 } else { 0.0 },
                Op::NotEq => if av != bv { 1.0 } else { 0.0 },
                Op::Lt    => if av < bv  { 1.0 } else { 0.0 },
                Op::LtEq  => if av <= bv { 1.0 } else { 0.0 },
                Op::Gt    => if av > bv  { 1.0 } else { 0.0 },
                Op::GtEq  => if av >= bv { 1.0 } else { 0.0 },
                Op::And   => if av != 0.0 && bv != 0.0 { 1.0 } else { 0.0 },
                Op::Or    => if av != 0.0 || bv != 0.0 { 1.0 } else { 0.0 },
                _ => 0.0,
            }
        });
        return Ok(Value::Matrix(result));
    }

    // ── String concatenation (`+` on StringObj) ───────────────────────────────
    if let (Op::Add, Value::StringObj(sa), Value::StringObj(sb)) = (&op, &a, &b) {
        return Ok(Value::StringObj(format!("{sa}{sb}")));
    }

    // ── General fallback: delegate to eval_with_io via temporary env entries ───
    // This handles ComplexMatrix, Str arithmetic, and any remaining combinations.
    vm_binop_fallback(a, op, b, env, io)
}

/// Fallback: evaluate `a op b` by inserting values into a temporary scope and
/// calling `eval_with_io` on a constructed `BinOp` expression.
fn vm_binop_fallback(a: Value, op: Op, b: Value, env: &mut Env, io: &mut IoContext) -> Result<Value, String> {
    let key_a = "__vm_op_a__".to_string();
    let key_b = "__vm_op_b__".to_string();
    let saved_a = env.get(&key_a).cloned();
    let saved_b = env.get(&key_b).cloned();
    env.insert(key_a.clone(), a);
    env.insert(key_b.clone(), b);
    let result = eval_with_io(
        &Expr::BinOp(
            Box::new(Expr::Var(key_a.clone())),
            op,
            Box::new(Expr::Var(key_b.clone())),
        ),
        env,
        io,
    );
    // Restore env to its previous state.
    match saved_a {
        Some(v) => { env.insert(key_a, v); }
        None    => { env.remove("__vm_op_a__"); }
    }
    match saved_b {
        Some(v) => { env.insert(key_b, v); }
        None    => { env.remove("__vm_op_b__"); }
    }
    result
}

/// Negate a [`Value`].  Fast path for `Scalar`; fallback via `eval_with_io` for others.
fn vm_neg(a: Value, env: &mut Env, io: &mut IoContext) -> Result<Value, String> {
    match &a {
        Value::Scalar(f) => return Ok(Value::Scalar(-f)),
        Value::Complex(re, im) => return Ok(Value::Complex(-re, -im)),
        Value::Matrix(m) => return Ok(Value::Matrix(m.mapv(|x| -x))),
        _ => {}
    }
    let key = "__vm_neg__".to_string();
    let saved = env.get(&key).cloned();
    env.insert(key.clone(), a);
    let result = eval_with_io(&Expr::UnaryMinus(Box::new(Expr::Var(key.clone()))), env, io);
    match saved {
        Some(v) => { env.insert(key, v); }
        None    => { env.remove("__vm_neg__"); }
    }
    result
}

// ── Truthiness ────────────────────────────────────────────────────────────────

/// Returns `true` if `val` is truthy under MATLAB `if`/`while` semantics.
///
/// Mirrors the `is_truthy` function in `exec.rs`.
fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Scalar(n) => *n != 0.0 && !n.is_nan(),
        Value::Matrix(m) => m.iter().all(|&x| x != 0.0 && !x.is_nan()),
        Value::Complex(re, im) => *re != 0.0 || *im != 0.0,
        Value::ComplexMatrix(m) => m.iter().all(|c: &Complex<f64>| c.re != 0.0 || c.im != 0.0),
        Value::Str(s) | Value::StringObj(s) => !s.is_empty(),
        Value::Void => false,
        Value::Lambda(_) | Value::Function { .. } | Value::Tuple(_) => true,
        Value::Cell(v) => !v.is_empty(),
        Value::Struct(_) | Value::StructArray(_) => true,
        Value::DateTime(ts) => !ts.is_nan(),
        Value::Duration(s) => *s != 0.0,
        Value::DateTimeArray(v) | Value::DurationArray(v) => !v.is_empty(),
        Value::Map(m) => !m.is_empty(),
    }
}

// ── Complex helpers ───────────────────────────────────────────────────────────

fn to_complex_parts(v: &Value) -> (f64, f64) {
    match v {
        Value::Scalar(f)      => (*f, 0.0),
        Value::Complex(re, im) => (*re, *im),
        _ => (f64::NAN, 0.0),
    }
}
