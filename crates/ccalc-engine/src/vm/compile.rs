//! Single-pass bytecode compiler.
//!
//! [`compile`] lowers a slice of [`StmtEntry`] nodes to a [`Chunk`].
//! Constructs the compiler cannot yet handle return [`CompileError::Unsupported`];
//! the caller must fall back to the tree-walking interpreter transparently.

use std::collections::{HashMap, HashSet};

use super::{Chunk, CompileError, Instr, Opcode};
use crate::env::Value;
use crate::eval::{Expr, Op, is_global, is_persistent};
use crate::parser::{Stmt, StmtEntry};

// ── Special call names that exec_stmts intercepts and that eval_with_io cannot
// handle (they need direct env mutation / RUN_DEPTH tracking / thread-local updates).
const EXEC_INTERCEPTS: &[&str] = &[
    "run", "source", "addpath", "rmpath", "path", "clear",
    "remove", // containers.Map in-place removal
    "format", // display mode — updates thread-locals via exec_stmts path
    "save", "load", "ws", "wl",
];

/// Builtin function names that can be called natively via [`Opcode::CallBuiltin`].
///
/// A call `f(args)` is compiled to `CallBuiltin` when:
/// 1. `f` is in this list, AND
/// 2. all arguments are recursively pure (no `EvalExpr` needed).
///
/// Exclusions: functions that mutate `env`, require I/O state, inspect `nargout`,
/// or have non-trivial multi-return semantics (`disp`, `fprintf`, `eval`, `clear`, etc.).
///
/// **Note:** if the user shadows one of these names with a variable (e.g. `sum = [1 2 3]`),
/// the builtin is always called — matrix indexing of the variable is not performed.
/// This is an accepted limitation for hot-loop performance.
pub const COMPILABLE_BUILTINS: &[&str] = &[
    // Scalar math
    "abs",
    "sqrt",
    "exp",
    "log",
    "log2",
    "log10",
    "sin",
    "cos",
    "tan",
    "asin",
    "acos",
    "atan",
    "atan2",
    "floor",
    "ceil",
    "round",
    "fix",
    "sign",
    "mod",
    "rem",
    // Complex
    "real",
    "imag",
    "conj",
    "angle",
    "isreal",
    // Predicates
    "isnan",
    "isinf",
    "isfinite",
    // Reductions (1-arg)
    "sum",
    "prod",
    "mean",
    "norm",
    "max",
    "min",
    "any",
    "all",
    "cumsum",
    "cumprod",
    // Shape / construction
    "size",
    "length",
    "numel",
    "zeros",
    "ones",
    "eye",
    "reshape",
    "fliplr",
    "flipud",
    "sort",
    "unique",
    "find",
    // Type conversion / introspection
    "num2str",
    "int2str",
    "str2num",
    "str2double",
    "ischar",
    "iscell",
    "isstruct",
];

// ── Public entry point ────────────────────────────────────────────────────────

/// Return `true` if every statement in `stmts` (recursively) can be compiled
/// to bytecode.
///
/// This is a cheap, allocation-free pre-check that mirrors `compile`'s
/// Unsupported rules.  Call it before `compile` to avoid the cost of partially
/// building a [`Chunk`] — especially relevant inside hot loops where the same
/// statement block is re-entered thousands of times.
pub fn is_compilable(stmts: &[StmtEntry]) -> bool {
    stmts.iter().all(|(stmt, _, _)| stmt_compilable(stmt))
}

fn stmt_compilable(stmt: &Stmt) -> bool {
    match stmt {
        // ── Supported statements ─────────────────────────────────────────────
        Stmt::Break | Stmt::Continue | Stmt::Return => true,
        Stmt::FunctionDef { .. } => true,

        Stmt::Assign(_, expr) => !is_exec_intercepted_call(expr),

        Stmt::Expr(Expr::Call(name, args)) => {
            if EXEC_INTERCEPTS.contains(&name.as_str()) {
                return false;
            }
            if name == "eval" && (args.len() == 1 || args.len() == 2) {
                return false;
            }
            true
        }
        Stmt::Expr(_) => true,

        Stmt::For { body, .. } => is_compilable(body),
        Stmt::While { body, .. } => is_compilable(body),

        Stmt::If {
            body,
            elseif_branches,
            else_body,
            ..
        } => {
            is_compilable(body)
                && elseif_branches.iter().all(|(_, b)| is_compilable(b))
                && else_body.as_ref().is_none_or(|b| is_compilable(b))
        }

        // ── Now supported via IndexSetOp ─────────────────────────────────────
        Stmt::IndexSet { .. } => true,

        // ── Unsupported (will cause CompileError::Unsupported in compile()) ──
        _ => false,
    }
}

/// Compile a statement block to a [`Chunk`].
///
/// Returns [`CompileError::Unsupported`] if any statement (or expression
/// nested within) cannot be lowered.  The caller should then fall back to
/// [`exec_stmts`](crate::exec::exec_stmts).
///
/// **Prefer calling [`is_compilable`] first** when the same statement block
/// is visited repeatedly (e.g. inside a hot loop) to avoid wasted allocation.
pub fn compile(stmts: &[StmtEntry]) -> Result<Chunk, CompileError> {
    // ── Pass 1: collect slot candidates (LHS of Assign + For loop vars) ──────
    let mut candidates: Vec<String> = Vec::new();
    collect_candidates(stmts, &mut candidates);

    // ── Pass 2: find names that must remain in env (env-required) ────────────
    let mut env_required: HashSet<String> = HashSet::new();
    // `ans` is always written via UpdateAns directly to env; never slot it.
    env_required.insert("ans".to_string());
    collect_env_required(stmts, &mut env_required);

    // ── Build slot map (candidates minus env-required) ────────────────────────
    let mut slot_map: HashMap<String, u16> = HashMap::new();
    let mut slot_names: Vec<String> = Vec::new();
    for name in &candidates {
        if !env_required.contains(name) && !is_global(name) && !is_persistent(name) {
            let slot = slot_names.len() as u16;
            slot_map.insert(name.clone(), slot);
            slot_names.push(name.clone());
        }
    }

    let const_map = build_const_map(stmts);

    let mut compiler = Compiler {
        chunk: Chunk::new(),
        loop_stack: Vec::new(),
        current_line: 0,
        slots: slot_map,
        const_map,
    };
    compiler.chunk.slot_names = slot_names;
    compiler.compile_stmts(stmts)?;
    Ok(compiler.chunk)
}

/// Compile a function body to a [`Chunk`] with parameters and outputs pre-slotted.
///
/// Unlike [`compile`], this variant assigns fixed slot indices to `params` (slots
/// `0..n_params`) and `outputs` (next slots) before the body's own locals.  The
/// resulting chunk's [`Chunk::n_params`] field records how many parameters were
/// successfully slotted.
///
/// When `chunk.n_params == params.len()` and [`is_leaf_fn`] returns `true` for
/// the chunk, the VM can execute the body with a pre-built `Vec<Value>` frame
/// instead of allocating a `HashMap` per call — the hot-function fast path.
///
/// Falls back transparently to [`compile`] semantics when a parameter is
/// env-required (appears in a non-pure expression): in that case the param keeps
/// a `StoreVar/LoadVar` round-trip through env, `chunk.names` is non-empty, and
/// `n_params < params.len()` — all of which cause the caller to reject the fast
/// path and use the existing full-`Env` call path.
pub fn compile_fn_body(
    stmts: &[StmtEntry],
    params: &[String],
    outputs: &[String],
) -> Result<Chunk, CompileError> {
    // ── Pass 1: body candidates (LHS of Assign + For loop vars) ──────────────
    let mut candidates: Vec<String> = Vec::new();
    collect_candidates(stmts, &mut candidates);

    // ── Pass 2: env-required names ────────────────────────────────────────────
    let mut env_required: HashSet<String> = HashSet::new();
    env_required.insert("ans".to_string());
    collect_env_required(stmts, &mut env_required);

    // ── Build slot map: params first, then outputs, then body candidates ──────
    let mut slot_map: HashMap<String, u16> = HashMap::new();
    let mut slot_names: Vec<String> = Vec::new();
    let mut n_params: usize = 0;

    // 1. Parameters at slots 0..n_params.
    for p in params {
        if !env_required.contains(p) && !is_global(p) && !is_persistent(p) {
            let slot = slot_names.len() as u16;
            slot_map.insert(p.clone(), slot);
            slot_names.push(p.clone());
            n_params += 1;
        }
        // If a param is env-required, n_params is not incremented.
        // The body will use LoadVar/StoreVar for it → chunk.names non-empty
        // → is_leaf_fn returns false → fast path not used.
    }

    // 2. Outputs at the next slots (unless already slotted as a param).
    for o in outputs {
        if !slot_map.contains_key(o)
            && !env_required.contains(o)
            && !is_global(o)
            && !is_persistent(o)
        {
            let slot = slot_names.len() as u16;
            slot_map.insert(o.clone(), slot);
            slot_names.push(o.clone());
        }
    }

    // 3. Remaining body candidates.
    for name in &candidates {
        if !slot_map.contains_key(name)
            && !env_required.contains(name)
            && !is_global(name)
            && !is_persistent(name)
        {
            let slot = slot_names.len() as u16;
            slot_map.insert(name.clone(), slot);
            slot_names.push(name.clone());
        }
    }

    let const_map = build_const_map(stmts);

    let mut compiler = Compiler {
        chunk: Chunk::new(),
        loop_stack: Vec::new(),
        current_line: 0,
        slots: slot_map,
        const_map,
    };
    compiler.chunk.slot_names = slot_names;
    compiler.chunk.n_params = n_params;
    compiler.compile_stmts(stmts)?;
    Ok(compiler.chunk)
}

/// Returns `true` when a compiled chunk is a *leaf function* — one whose body
/// never accesses `env` during execution.
///
/// The condition `chunk.names.is_empty()` ensures no `LoadVar`, `StoreVar`,
/// `IterNext`, `CallBuiltin`, `CallUser`, `EvalExpr`, or `DefineFunc`
/// instruction references a name from the name pool.  Without those opcodes
/// the VM loop never reads from or writes to `env`, so the function body can
/// run against an empty scratch environment with zero overhead.
///
/// Used by `call_user_function` to select the Vec-frame fast path.
pub fn is_leaf_fn(chunk: &Chunk) -> bool {
    chunk.names.is_empty()
}

// ── Compiler context ──────────────────────────────────────────────────────────

struct LoopFrame {
    /// Instruction index to jump to on `continue`.
    ///
    /// For `for` loops this is the `IterNext` opcode; for `while` loops it is the
    /// first instruction of the condition block.
    continue_target: usize,
    /// Instruction indices that need their i32 payload set to the loop exit offset.
    break_patches: Vec<usize>,
    /// `true` for `for` loops — `break` must emit `PopIter` before the jump.
    is_for: bool,
}

struct Compiler {
    chunk: Chunk,
    loop_stack: Vec<LoopFrame>,
    current_line: usize,
    /// Map from variable name to slot index for pure-local variables.
    slots: HashMap<String, u16>,
    /// Compile-time constant values for variables that are assigned exactly once
    /// at top-level before any loop, with a constant RHS.  Used by `const_eval`.
    const_map: HashMap<String, Value>,
}

impl Compiler {
    // ── Emit helpers ──────────────────────────────────────────────────────────

    /// Push an instruction and record the current source line for it.
    fn emit(&mut self, instr: Instr) {
        self.chunk.lines.push(self.current_line);
        self.chunk.code.push(instr);
    }

    // ── Statement sequence ────────────────────────────────────────────────────

    fn compile_stmts(&mut self, stmts: &[StmtEntry]) -> Result<(), CompileError> {
        for (stmt, silent, line) in stmts {
            self.current_line = *line;
            self.compile_stmt(stmt, *silent)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: &Stmt, silent: bool) -> Result<(), CompileError> {
        match stmt {
            // ── Simple assignment ─────────────────────────────────────────────
            Stmt::Assign(name, expr) => {
                // Reject assignments whose RHS is a call to exec_stmts-intercepted
                // functions: eval_with_io cannot execute them correctly.
                if is_exec_intercepted_call(expr) {
                    return Err(CompileError::Unsupported);
                }
                self.compile_expr_push(expr);
                if let Some(&slot) = self.slots.get(name) {
                    self.emit(Instr::with_u16_u8(
                        Opcode::StoreSlot,
                        slot,
                        u8::from(silent),
                    ));
                } else {
                    let idx = self.chunk.name_idx(name);
                    self.emit(Instr::with_u16_u8(Opcode::StoreVar, idx, u8::from(silent)));
                }
                Ok(())
            }

            // ── Expression statement ──────────────────────────────────────────
            Stmt::Expr(expr) => {
                // Reject calls that exec_stmts intercepts with env mutation.
                if let Expr::Call(name, _) = expr
                    && EXEC_INTERCEPTS.contains(&name.as_str())
                {
                    return Err(CompileError::Unsupported);
                }
                // `eval("...")` as a statement modifies `env` directly through the
                // exec_stmts path and cannot go through eval_with_io.
                if let Expr::Call(name, args) = expr
                    && name == "eval"
                    && (args.len() == 1 || args.len() == 2)
                {
                    return Err(CompileError::Unsupported);
                }
                self.compile_expr_push(expr);
                // ans is always updated, even for silent expr statements.
                self.emit(Instr::no_arg(Opcode::UpdateAns));
                if silent {
                    self.emit(Instr::no_arg(Opcode::Pop));
                } else {
                    self.emit(Instr::no_arg(Opcode::Print));
                }
                Ok(())
            }

            // ── for loop ─────────────────────────────────────────────────────
            Stmt::For {
                var,
                range_expr,
                body,
            } => {
                // Evaluate range and build iterator.
                self.compile_expr_push(range_expr);
                self.emit(Instr::no_arg(Opcode::PushIter));

                let iter_next_pos = self.chunk.code.len();

                if let Some(&slot) = self.slots.get(var) {
                    // Slotted loop variable: emit IterNextSlot with placeholder.
                    self.chunk
                        .code
                        .push(Instr::with_u16_i32(Opcode::IterNextSlot, slot, 0));

                    self.loop_stack.push(LoopFrame {
                        continue_target: iter_next_pos,
                        break_patches: Vec::new(),
                        is_for: true,
                    });

                    self.compile_stmts(body)?;

                    let back_off = iter_next_pos as i32 - self.chunk.code.len() as i32 - 1;
                    self.emit(Instr::with_i32(Opcode::Jump, back_off));

                    let exit_pos = self.chunk.code.len();
                    let exit_off = exit_pos as i32 - iter_next_pos as i32 - 1;
                    self.chunk.code[iter_next_pos].set_u16_i32(slot, exit_off);

                    let frame = self.loop_stack.pop().unwrap();
                    for p in frame.break_patches {
                        let off = exit_pos as i32 - p as i32 - 1;
                        self.chunk.code[p].set_i32(off);
                    }
                } else {
                    // Env-backed loop variable: emit IterNext with placeholder.
                    let var_idx = self.chunk.name_idx(var);
                    self.chunk
                        .code
                        .push(Instr::with_u16_i32(Opcode::IterNext, var_idx, 0));

                    self.loop_stack.push(LoopFrame {
                        continue_target: iter_next_pos,
                        break_patches: Vec::new(),
                        is_for: true,
                    });

                    self.compile_stmts(body)?;

                    let back_off = iter_next_pos as i32 - self.chunk.code.len() as i32 - 1;
                    self.emit(Instr::with_i32(Opcode::Jump, back_off));

                    let exit_pos = self.chunk.code.len();
                    let exit_off = exit_pos as i32 - iter_next_pos as i32 - 1;
                    self.chunk.code[iter_next_pos].set_u16_i32(var_idx, exit_off);

                    let frame = self.loop_stack.pop().unwrap();
                    for p in frame.break_patches {
                        let off = exit_pos as i32 - p as i32 - 1;
                        self.chunk.code[p].set_i32(off);
                    }
                }
                Ok(())
            }

            // ── while loop ───────────────────────────────────────────────────
            Stmt::While { cond, body } => {
                let cond_pos = self.chunk.code.len();
                self.compile_expr_push(cond);

                let jf_idx = self.chunk.code.len();
                self.emit(Instr::with_i32(Opcode::JumpFalsy, 0));

                self.loop_stack.push(LoopFrame {
                    continue_target: cond_pos,
                    break_patches: Vec::new(),
                    is_for: false,
                });

                self.compile_stmts(body)?;

                // Back-edge to condition.
                let back_off = cond_pos as i32 - self.chunk.code.len() as i32 - 1;
                self.emit(Instr::with_i32(Opcode::Jump, back_off));

                let exit_pos = self.chunk.code.len();
                // Patch JumpFalsy.
                let jf_off = exit_pos as i32 - jf_idx as i32 - 1;
                self.chunk.code[jf_idx].set_i32(jf_off);

                // Patch breaks.
                let frame = self.loop_stack.pop().unwrap();
                for p in frame.break_patches {
                    let off = exit_pos as i32 - p as i32 - 1;
                    self.chunk.code[p].set_i32(off);
                }
                Ok(())
            }

            // ── if / elseif / else ────────────────────────────────────────────
            Stmt::If {
                cond,
                body,
                elseif_branches,
                else_body,
            } => {
                // Positions of Jump-to-end instructions that need backpatching.
                let mut end_patches: Vec<usize> = Vec::new();

                // Main condition.
                self.compile_expr_push(cond);
                let jf_idx = self.chunk.code.len();
                self.emit(Instr::with_i32(Opcode::JumpFalsy, 0));

                self.compile_stmts(body)?;

                if elseif_branches.is_empty() && else_body.is_none() {
                    // Simple if — no end-jump needed; patch JumpFalsy directly to exit.
                    let exit_pos = self.chunk.code.len();
                    let off = exit_pos as i32 - jf_idx as i32 - 1;
                    self.chunk.code[jf_idx].set_i32(off);
                } else {
                    // Multi-branch: add Jump-to-end after the body.
                    let ej_idx = self.chunk.code.len();
                    self.emit(Instr::with_i32(Opcode::Jump, 0));
                    end_patches.push(ej_idx);

                    // Patch JumpFalsy to the next branch.
                    let next = self.chunk.code.len();
                    let off = next as i32 - jf_idx as i32 - 1;
                    self.chunk.code[jf_idx].set_i32(off);

                    // Elseif branches.
                    for (ei_cond, ei_body) in elseif_branches {
                        self.compile_expr_push(ei_cond);
                        let ei_jf = self.chunk.code.len();
                        self.emit(Instr::with_i32(Opcode::JumpFalsy, 0));

                        self.compile_stmts(ei_body)?;

                        let ej2 = self.chunk.code.len();
                        self.emit(Instr::with_i32(Opcode::Jump, 0));
                        end_patches.push(ej2);

                        let next2 = self.chunk.code.len();
                        let off2 = next2 as i32 - ei_jf as i32 - 1;
                        self.chunk.code[ei_jf].set_i32(off2);
                    }

                    // Optional else body.
                    if let Some(else_stmts) = else_body {
                        self.compile_stmts(else_stmts)?;
                    }

                    // Patch all end-jumps.
                    let end_pos = self.chunk.code.len();
                    for p in end_patches {
                        let off = end_pos as i32 - p as i32 - 1;
                        self.chunk.code[p].set_i32(off);
                    }
                }
                Ok(())
            }

            // ── break ─────────────────────────────────────────────────────────
            Stmt::Break => {
                let Some(frame) = self.loop_stack.last_mut() else {
                    return Err(CompileError::Unsupported);
                };
                if frame.is_for {
                    self.emit(Instr::no_arg(Opcode::PopIter));
                }
                let j_idx = self.chunk.code.len();
                self.emit(Instr::with_i32(Opcode::Jump, 0));
                // Defer to pop() after the loop is closed — borrow ends here.
                let j_idx_owned = j_idx;
                self.loop_stack
                    .last_mut()
                    .unwrap()
                    .break_patches
                    .push(j_idx_owned);
                Ok(())
            }

            // ── continue ──────────────────────────────────────────────────────
            Stmt::Continue => {
                let Some(frame) = self.loop_stack.last() else {
                    return Err(CompileError::Unsupported);
                };
                let target = frame.continue_target;
                let off = target as i32 - self.chunk.code.len() as i32 - 1;
                self.emit(Instr::with_i32(Opcode::Jump, off));
                Ok(())
            }

            // ── return ────────────────────────────────────────────────────────
            Stmt::Return => {
                self.emit(Instr::no_arg(Opcode::Return));
                Ok(())
            }

            // ── FunctionDef — register function in env at runtime ─────────────
            // exec_script calls hoist_functions first (forward references in scripts),
            // but exec_stmts called directly does not.  The tree-walker also inserts
            // the function in env when it encounters FunctionDef.  Emit DefineFunc
            // so vm_exec replicates that behaviour correctly in both call paths.
            Stmt::FunctionDef {
                name,
                outputs,
                params,
                body_source,
                doc,
            } => {
                use crate::env::{FunctionData, Value};
                use indexmap::IndexMap;
                let func = Value::Function(Box::new(FunctionData {
                    outputs: outputs.clone(),
                    params: params.clone(),
                    body_source: body_source.clone(),
                    locals: IndexMap::new(),
                    doc: doc.clone(),
                }));
                let const_idx = self.chunk.add_const(func);
                let name_idx = self.chunk.name_idx(name);
                self.emit(Instr::with_u16_u16(Opcode::DefineFunc, name_idx, const_idx));
                Ok(())
            }

            // ── Indexed assignment: A(i, j) = v ──────────────────────────────
            Stmt::IndexSet {
                name,
                indices,
                value,
            } => {
                // Push RHS onto the stack.
                self.compile_expr_push(value);
                // Store the index Expr list in the pool.
                let iset_idx = self.chunk.index_sets.len() as u16;
                self.chunk.index_sets.push(indices.clone());
                let name_idx = self.chunk.name_idx(name);
                self.emit(Instr::with_u16_u16_u8(
                    Opcode::IndexSetOp,
                    name_idx,
                    iset_idx,
                    u8::from(silent),
                ));
                Ok(())
            }

            // ── All other statement kinds are not yet supported ────────────────
            _ => Err(CompileError::Unsupported),
        }
    }

    // ── Expression compilation ─────────────────────────────────────────────────

    /// Emit bytecode that pushes the result of `expr` onto the operand stack.
    ///
    /// Simple expressions (numeric literals, variable loads, and arithmetic on
    /// those) compile to native opcodes.  Everything else emits [`Opcode::EvalExpr`]
    /// which calls `eval_with_io` at runtime — correct for all types but without
    /// the loop-overhead elimination benefit.
    fn compile_expr_push(&mut self, expr: &Expr) {
        if Self::is_pure(expr) {
            self.compile_native(expr);
        } else {
            let idx = self.chunk.add_expr(expr.clone());
            self.emit(Instr::with_u16(Opcode::EvalExpr, idx));
        }
    }

    /// Returns `true` if `expr` can be compiled to pure stack opcodes without
    /// any call to `eval_with_io`.
    ///
    /// "Pure" means: numeric literals, variable loads, binary / unary arithmetic,
    /// and calls to [`COMPILABLE_BUILTINS`] whose arguments are recursively pure.
    /// Excludes `ElemAnd`, `ElemOr`, and `LDiv` which are rare enough to
    /// handle via `EvalExpr`.
    fn is_pure(expr: &Expr) -> bool {
        match expr {
            Expr::Number(_) => true,
            // `end` is not a regular env variable — it is injected by eval_index during
            // indexing.  Marking it non-pure prevents v(end) from being compiled to
            // CallUser (where `end` would not be in env and the LoadVar would fail).
            Expr::Var(name) => name != "end",
            Expr::UnaryMinus(e) | Expr::UnaryNot(e) => Self::is_pure(e),
            Expr::BinOp(a, op, b) => {
                !matches!(op, Op::ElemAnd | Op::ElemOr | Op::LDiv)
                    && Self::is_pure(a)
                    && Self::is_pure(b)
            }
            Expr::Call(name, args) => {
                // CallBuiltin: whitelisted math/predicate builtins.
                if COMPILABLE_BUILTINS.contains(&name.as_str()) && args.iter().all(Self::is_pure) {
                    return true;
                }
                // CallUser: non-intercepted user function calls with pure args.
                // The callee name is added to env-required by collect_user_callee_names
                // so the VM can look it up at dispatch time.
                !EXEC_INTERCEPTS.contains(&name.as_str())
                    && name != "eval"
                    && args.iter().all(Self::is_pure)
            }
            _ => false,
        }
    }

    /// Emit native stack opcodes for a pure expression (no `eval_with_io`).
    ///
    /// Tries `const_eval` first; if the entire expression folds to a constant
    /// at compile time, emits a single `PushConst` and returns early.
    /// Panics if `expr` is not pure — callers must check [`is_pure`] first.
    fn compile_native(&mut self, expr: &Expr) {
        if let Some(val) = const_eval(expr, &self.const_map) {
            let idx = self.chunk.add_const(val);
            self.emit(Instr::with_u16(Opcode::PushConst, idx));
            return;
        }
        match expr {
            Expr::Number(f) => {
                let idx = self.chunk.add_const(Value::Scalar(*f));
                self.emit(Instr::with_u16(Opcode::PushConst, idx));
            }
            Expr::Var(name) => {
                if let Some(&slot) = self.slots.get(name) {
                    self.emit(Instr::with_u16(Opcode::LoadSlot, slot));
                } else {
                    let idx = self.chunk.name_idx(name);
                    self.emit(Instr::with_u16(Opcode::LoadVar, idx));
                }
            }
            Expr::UnaryMinus(inner) => {
                self.compile_native(inner);
                self.emit(Instr::no_arg(Opcode::Neg));
            }
            Expr::UnaryNot(inner) => {
                self.compile_native(inner);
                self.emit(Instr::no_arg(Opcode::Not));
            }
            Expr::BinOp(left, op, right) => {
                self.compile_native(left);
                self.compile_native(right);
                let opcode = match op {
                    Op::Add => Opcode::Add,
                    Op::Sub => Opcode::Sub,
                    Op::Mul => Opcode::Mul,
                    Op::Div => Opcode::Div,
                    Op::Pow => Opcode::Pow,
                    Op::ElemMul => Opcode::ElemMul,
                    Op::ElemDiv => Opcode::ElemDiv,
                    Op::ElemPow => Opcode::ElemPow,
                    Op::Eq => Opcode::Eq,
                    Op::NotEq => Opcode::Ne,
                    Op::Lt => Opcode::Lt,
                    Op::LtEq => Opcode::Le,
                    Op::Gt => Opcode::Gt,
                    Op::GtEq => Opcode::Ge,
                    Op::And => Opcode::And,
                    Op::Or => Opcode::Or,
                    // ElemAnd, ElemOr, LDiv are excluded by is_pure(); this arm
                    // is unreachable in correct usage.
                    Op::ElemAnd | Op::ElemOr | Op::LDiv => {
                        unreachable!("compile_native: is_pure should have excluded this op")
                    }
                };
                self.emit(Instr::no_arg(opcode));
            }
            Expr::Call(name, args) => {
                // Push all arguments onto the stack first (left-to-right).
                for arg in args {
                    self.compile_native(arg);
                }
                let name_idx = self.chunk.name_idx(name);
                if COMPILABLE_BUILTINS.contains(&name.as_str()) {
                    // CallBuiltin: direct dispatch to call_builtin.
                    self.emit(Instr::with_u16_u8(
                        Opcode::CallBuiltin,
                        name_idx,
                        args.len() as u8,
                    ));
                } else {
                    // CallUser: dispatch via dispatch_user_call_for_vm.
                    // The callee name is kept in env (not a slot) so the lookup is valid.
                    self.emit(Instr::with_u16_u8(
                        Opcode::CallUser,
                        name_idx,
                        args.len() as u8,
                    ));
                }
            }
            _ => unreachable!("compile_native called on non-pure expression"),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns `true` if `expr` is a top-level call to a function that exec_stmts
/// intercepts with env-mutating semantics that `eval_with_io` cannot replicate.
fn is_exec_intercepted_call(expr: &Expr) -> bool {
    if let Expr::Call(name, _) = expr {
        return EXEC_INTERCEPTS.contains(&name.as_str());
    }
    false
}

// ── Slot analysis passes ──────────────────────────────────────────────────────

/// Pass 1: collect all variable names that are candidates for slotting.
///
/// A name is a candidate if it appears as the LHS of any `Assign` statement
/// or as the loop variable of any `For` statement (recursively).
fn collect_candidates(stmts: &[StmtEntry], out: &mut Vec<String>) {
    for (stmt, _, _) in stmts {
        match stmt {
            Stmt::Assign(name, _) if !out.contains(name) => {
                out.push(name.clone());
            }
            Stmt::Assign(_, _) => {}
            Stmt::For { var, body, .. } => {
                if !out.contains(var) {
                    out.push(var.clone());
                }
                collect_candidates(body, out);
            }
            Stmt::While { body, .. } => collect_candidates(body, out),
            Stmt::If {
                body,
                elseif_branches,
                else_body,
                ..
            } => {
                collect_candidates(body, out);
                for (_, b) in elseif_branches {
                    collect_candidates(b, out);
                }
                if let Some(b) = else_body {
                    collect_candidates(b, out);
                }
            }
            _ => {}
        }
    }
}

/// Pass 2: collect names that must remain in `env` (must NOT be slotted).
///
/// A name is env-required if it appears as a free variable inside any
/// expression that will be evaluated via `EvalExpr` (i.e., not `is_pure`),
/// or inside any `IndexSet` index expression, or is the target of an
/// `IndexSet` operation (because `exec_index_set` reads from env).
fn collect_env_required(stmts: &[StmtEntry], out: &mut HashSet<String>) {
    for (stmt, _, _) in stmts {
        match stmt {
            Stmt::Assign(_, expr) if !Compiler::is_pure(expr) => {
                free_vars_in_expr(expr, out);
            }
            Stmt::Assign(_, expr) => {
                // Pure expr: arg vars can be slots, but callee names for CallUser must stay in env.
                collect_user_callee_names(expr, out);
            }
            Stmt::Expr(expr) if !Compiler::is_pure(expr) => {
                free_vars_in_expr(expr, out);
            }
            Stmt::Expr(expr) => {
                collect_user_callee_names(expr, out);
            }
            Stmt::For {
                range_expr, body, ..
            } => {
                if !Compiler::is_pure(range_expr) {
                    free_vars_in_expr(range_expr, out);
                } else {
                    collect_user_callee_names(range_expr, out);
                }
                collect_env_required(body, out);
            }
            Stmt::While { cond, body } => {
                if !Compiler::is_pure(cond) {
                    free_vars_in_expr(cond, out);
                } else {
                    collect_user_callee_names(cond, out);
                }
                collect_env_required(body, out);
            }
            Stmt::If {
                cond,
                body,
                elseif_branches,
                else_body,
            } => {
                if !Compiler::is_pure(cond) {
                    free_vars_in_expr(cond, out);
                } else {
                    collect_user_callee_names(cond, out);
                }
                collect_env_required(body, out);
                for (ei_cond, ei_body) in elseif_branches {
                    if !Compiler::is_pure(ei_cond) {
                        free_vars_in_expr(ei_cond, out);
                    } else {
                        collect_user_callee_names(ei_cond, out);
                    }
                    collect_env_required(ei_body, out);
                }
                if let Some(b) = else_body {
                    collect_env_required(b, out);
                }
            }
            Stmt::IndexSet {
                name,
                indices,
                value,
            } => {
                // The target variable is always read+written via env by exec_index_set.
                out.insert(name.clone());
                // Index expressions are evaluated against env by exec_index_set.
                for idx in indices {
                    free_vars_in_expr(idx, out);
                }
                // RHS: if not pure, collect free vars; if pure, collect callee names.
                if !Compiler::is_pure(value) {
                    free_vars_in_expr(value, out);
                } else {
                    collect_user_callee_names(value, out);
                }
            }
            _ => {}
        }
    }
}

/// Collect non-builtin callee names from a pure expression.
///
/// For `CallUser` paths: `inc(k)` adds `"inc"` to env-required so the VM can look up
/// the function at dispatch time.  Arg variable names are intentionally NOT added —
/// they remain eligible for slots.  Builtin callee names (`abs`, `sin`, …) are
/// omitted because `CallBuiltin` resolves them without an env lookup.
fn collect_user_callee_names(expr: &Expr, out: &mut HashSet<String>) {
    match expr {
        Expr::Call(name, args) => {
            if !COMPILABLE_BUILTINS.contains(&name.as_str()) {
                // Non-builtin callee: must stay in env for runtime dispatch.
                out.insert(name.clone());
            }
            for arg in args {
                collect_user_callee_names(arg, out);
            }
        }
        Expr::BinOp(a, _, b) => {
            collect_user_callee_names(a, out);
            collect_user_callee_names(b, out);
        }
        Expr::UnaryMinus(e) | Expr::UnaryNot(e) => {
            collect_user_callee_names(e, out);
        }
        // Leaf nodes (Number, Var) and other exprs have no callee names.
        _ => {}
    }
}

// ── Constant folding ──────────────────────────────────────────────────────────

/// Evaluate `expr` at compile time.
///
/// Returns `Some(Value::Scalar(…))` when the entire expression reduces to a
/// scalar constant using only literal numbers and entries in `const_map`.
/// Returns `None` for any sub-expression that is not statically known.
///
/// **Safety invariant:** any `Some(v)` returned must equal the value that
/// `vm_exec` would produce at runtime for the same operands.  Returning `None`
/// is always safe (the compiler falls through to the emit-at-runtime path).
/// Division by zero intentionally returns `None` to stay conservative.
fn const_eval(expr: &Expr, const_map: &HashMap<String, Value>) -> Option<Value> {
    match expr {
        Expr::Number(f) => Some(Value::Scalar(*f)),
        Expr::Var(name) => const_map.get(name.as_str()).cloned(),
        Expr::UnaryMinus(inner) => {
            if let Some(Value::Scalar(f)) = const_eval(inner, const_map) {
                Some(Value::Scalar(-f))
            } else {
                None
            }
        }
        Expr::BinOp(a, op, b) => {
            let va = const_eval(a, const_map)?;
            let vb = const_eval(b, const_map)?;
            if let (Value::Scalar(fa), Value::Scalar(fb)) = (va, vb) {
                match op {
                    // Conservative: don't fold division by zero even though IEEE gives inf.
                    Op::Div if fb == 0.0 => None,
                    Op::ElemDiv if fb == 0.0 => None,
                    Op::Add => Some(Value::Scalar(fa + fb)),
                    Op::Sub => Some(Value::Scalar(fa - fb)),
                    Op::Mul | Op::ElemMul => Some(Value::Scalar(fa * fb)),
                    Op::Div | Op::ElemDiv => Some(Value::Scalar(fa / fb)),
                    Op::Pow | Op::ElemPow => Some(Value::Scalar(fa.powf(fb))),
                    Op::Eq => Some(Value::Scalar(if fa == fb { 1.0 } else { 0.0 })),
                    Op::NotEq => Some(Value::Scalar(if fa != fb { 1.0 } else { 0.0 })),
                    Op::Lt => Some(Value::Scalar(if fa < fb { 1.0 } else { 0.0 })),
                    Op::LtEq => Some(Value::Scalar(if fa <= fb { 1.0 } else { 0.0 })),
                    Op::Gt => Some(Value::Scalar(if fa > fb { 1.0 } else { 0.0 })),
                    Op::GtEq => Some(Value::Scalar(if fa >= fb { 1.0 } else { 0.0 })),
                    Op::And => Some(Value::Scalar(if fa != 0.0 && fb != 0.0 {
                        1.0
                    } else {
                        0.0
                    })),
                    Op::Or => Some(Value::Scalar(if fa != 0.0 || fb != 0.0 {
                        1.0
                    } else {
                        0.0
                    })),
                    _ => None, // ElemAnd, ElemOr, LDiv not folded
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Build a map of compile-time-constant variable names from the top-level
/// statements of a block.
///
/// A variable is a "named constant" when:
/// 1. It is assigned exactly once across the entire block (including nested loops).
/// 2. Its assignment appears at the top level before any loop or conditional.
/// 3. The RHS evaluates to a scalar constant via `const_eval`.
/// 4. It has not been referenced as a free variable in any *earlier* statement
///    (use-before-def guard — prevents folding a name to its eventual value
///    before it has been assigned at runtime).
///
/// These names are folded to their constant values wherever they appear in
/// pure expressions within the compiled chunk.
fn build_const_map(stmts: &[StmtEntry]) -> HashMap<String, Value> {
    // Count total assignments for every name in the block (incl. nested).
    let mut total_counts: HashMap<String, usize> = HashMap::new();
    for (stmt, _, _) in stmts {
        collect_assignment_counts(stmt, &mut total_counts);
    }

    let mut const_map: HashMap<String, Value> = HashMap::new();
    // Track names that appeared as free vars in expressions seen so far.
    // A name in this set was "used before defined" — don't fold it.
    let mut seen: HashSet<String> = HashSet::new();

    for (stmt, _, _) in stmts {
        match stmt {
            // Stop at the first loop or conditional — we can only guarantee
            // constness for names assigned before any branching.
            Stmt::For { .. } | Stmt::While { .. } | Stmt::If { .. } => break,
            Stmt::Assign(name, expr) => {
                if total_counts.get(name).copied().unwrap_or(0) == 1
                    && !seen.contains(name.as_str())
                {
                    // Build the map incrementally so `b = a * 3` folds if `a` is already known.
                    if let Some(val) = const_eval(expr, &const_map) {
                        const_map.insert(name.clone(), val);
                    }
                }
                // Record the RHS free vars: any name used here was seen before
                // any definition that follows.
                free_vars_in_expr(expr, &mut seen);
            }
            Stmt::Expr(expr) => {
                free_vars_in_expr(expr, &mut seen);
            }
            _ => {}
        }
    }
    const_map
}

/// Recursively tally how many times each variable name is the target of an
/// assignment (Assign or IndexSet) or a for-loop variable anywhere in `stmt`
/// and its children.
fn collect_assignment_counts(stmt: &Stmt, counts: &mut HashMap<String, usize>) {
    match stmt {
        Stmt::Assign(name, _) => *counts.entry(name.clone()).or_insert(0) += 1,
        Stmt::IndexSet { name, .. } => *counts.entry(name.clone()).or_insert(0) += 1,
        Stmt::For { var, body, .. } => {
            *counts.entry(var.clone()).or_insert(0) += 1;
            for (s, _, _) in body {
                collect_assignment_counts(s, counts);
            }
        }
        Stmt::While { body, .. } => {
            for (s, _, _) in body {
                collect_assignment_counts(s, counts);
            }
        }
        Stmt::If {
            body,
            elseif_branches,
            else_body,
            ..
        } => {
            for (s, _, _) in body {
                collect_assignment_counts(s, counts);
            }
            for (_, b) in elseif_branches {
                for (s, _, _) in b {
                    collect_assignment_counts(s, counts);
                }
            }
            if let Some(b) = else_body {
                for (s, _, _) in b {
                    collect_assignment_counts(s, counts);
                }
            }
        }
        _ => {}
    }
}

/// Recursively collect all `Expr::Var` names that appear free in `expr`.
fn free_vars_in_expr(expr: &Expr, out: &mut HashSet<String>) {
    match expr {
        Expr::Var(name) => {
            out.insert(name.clone());
        }
        Expr::Number(_)
        | Expr::StrLiteral(_)
        | Expr::StringObjLiteral(_)
        | Expr::Colon
        | Expr::NaT
        | Expr::FuncHandle(_) => {}
        Expr::UnaryMinus(e) | Expr::UnaryNot(e) | Expr::Transpose(e) | Expr::PlainTranspose(e) => {
            free_vars_in_expr(e, out);
        }
        Expr::BinOp(a, _, b) => {
            free_vars_in_expr(a, out);
            free_vars_in_expr(b, out);
        }
        Expr::Call(name, args) => {
            // The callee name may be a variable (e.g. matrix indexing `v(i)`):
            // eval_with_io looks it up from env, so it must not be slotted.
            out.insert(name.clone());
            for a in args {
                free_vars_in_expr(a, out);
            }
        }
        Expr::CellLiteral(args) => {
            for a in args {
                free_vars_in_expr(a, out);
            }
        }
        Expr::Matrix(rows) => {
            for row in rows {
                for e in row {
                    free_vars_in_expr(e, out);
                }
            }
        }
        Expr::Range(a, step, b) => {
            free_vars_in_expr(a, out);
            if let Some(s) = step {
                free_vars_in_expr(s, out);
            }
            free_vars_in_expr(b, out);
        }
        Expr::CellIndex(base, idx) => {
            free_vars_in_expr(base, out);
            free_vars_in_expr(idx, out);
        }
        Expr::FieldGet(base, _) => {
            free_vars_in_expr(base, out);
        }
        Expr::DynFieldGet(base, field) => {
            free_vars_in_expr(base, out);
            free_vars_in_expr(field, out);
        }
        Expr::DotCall(_, args) => {
            for a in args {
                free_vars_in_expr(a, out);
            }
        }
        Expr::Lambda { body, .. } => {
            free_vars_in_expr(body, out);
        }
    }
}
