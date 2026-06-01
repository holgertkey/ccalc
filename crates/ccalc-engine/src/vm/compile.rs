//! Single-pass bytecode compiler.
//!
//! [`compile`] lowers a slice of [`StmtEntry`] nodes to a [`Chunk`].
//! Constructs the compiler cannot yet handle return [`CompileError::Unsupported`];
//! the caller must fall back to the tree-walking interpreter transparently.

use crate::eval::{Expr, Op};
use crate::env::Value;
use crate::parser::{Stmt, StmtEntry};
use super::{Chunk, CompileError, Instr, Opcode};

// ── Special call names that exec_stmts intercepts and that eval_with_io cannot
// handle (they need direct env mutation / RUN_DEPTH tracking / thread-local updates).
const EXEC_INTERCEPTS: &[&str] = &[
    "run", "source",
    "addpath", "rmpath", "path",
    "clear",
    "remove",  // containers.Map in-place removal
    "format",  // display mode — updates thread-locals via exec_stmts path
    "save", "load", "ws", "wl",
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
                && else_body.as_ref().map_or(true, |b| is_compilable(b))
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
    let mut compiler = Compiler {
        chunk:        Chunk::new(),
        loop_stack:   Vec::new(),
        current_line: 0,
    };
    compiler.compile_stmts(stmts)?;
    Ok(compiler.chunk)
}

// ── Compiler context ──────────────────────────────────────────────────────────

struct LoopFrame {
    /// Instruction index to jump to on `continue`.
    ///
    /// For `for` loops this is the `IterNext` opcode; for `while` loops it is the
    /// first instruction of the condition block.
    continue_target: usize,
    /// Instruction indices that need their i32 payload set to the loop exit offset.
    break_patches:   Vec<usize>,
    /// `true` for `for` loops — `break` must emit `PopIter` before the jump.
    is_for:          bool,
}

struct Compiler {
    chunk:        Chunk,
    loop_stack:   Vec<LoopFrame>,
    current_line: usize,
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
                let idx = self.chunk.name_idx(name);
                self.emit(Instr::with_u16_u8(
                    Opcode::StoreVar,
                    idx,
                    u8::from(silent),
                ));
                Ok(())
            }

            // ── Expression statement ──────────────────────────────────────────
            Stmt::Expr(expr) => {
                // Reject calls that exec_stmts intercepts with env mutation.
                if let Expr::Call(name, _) = expr {
                    if EXEC_INTERCEPTS.contains(&name.as_str()) {
                        return Err(CompileError::Unsupported);
                    }
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
            Stmt::For { var, range_expr, body } => {
                // Evaluate range and build iterator.
                self.compile_expr_push(range_expr);
                self.emit(Instr::no_arg(Opcode::PushIter));

                let var_idx = self.chunk.name_idx(var);
                let iter_next_pos = self.chunk.code.len();

                // Emit IterNext with placeholder exit offset.
                self.chunk
                    .code
                    .push(Instr::with_u16_i32(Opcode::IterNext, var_idx, 0));

                // Push loop frame (continue → IterNext; break → after loop).
                self.loop_stack.push(LoopFrame {
                    continue_target: iter_next_pos,
                    break_patches:   Vec::new(),
                    is_for:          true,
                });

                self.compile_stmts(body)?;

                // Back-edge: jump back to IterNext.
                let back_off = iter_next_pos as i32 - self.chunk.code.len() as i32 - 1;
                self.emit(Instr::with_i32(Opcode::Jump, back_off));

                // Exit position and patch IterNext.
                let exit_pos = self.chunk.code.len();
                let exit_off = exit_pos as i32 - iter_next_pos as i32 - 1;
                self.chunk.code[iter_next_pos].set_u16_i32(var_idx, exit_off);

                // Patch all break jumps.
                let frame = self.loop_stack.pop().unwrap();
                for p in frame.break_patches {
                    let off = exit_pos as i32 - p as i32 - 1;
                    self.chunk.code[p].set_i32(off);
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
                    break_patches:   Vec::new(),
                    is_for:          false,
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
                self.loop_stack.last_mut().unwrap().break_patches.push(j_idx_owned);
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
                use crate::env::Value;
                use indexmap::IndexMap;
                let func = Value::Function {
                    outputs:     outputs.clone(),
                    params:      params.clone(),
                    body_source: body_source.clone(),
                    locals:      IndexMap::new(),
                    doc:         doc.clone(),
                };
                let const_idx = self.chunk.add_const(func);
                let name_idx  = self.chunk.name_idx(name);
                self.emit(Instr::with_u16_u16(Opcode::DefineFunc, name_idx, const_idx));
                Ok(())
            }

            // ── Indexed assignment: A(i, j) = v ──────────────────────────────
            Stmt::IndexSet { name, indices, value } => {
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
    /// "Pure" means: numeric literals, variable loads, and binary / unary
    /// arithmetic on those — excluding `ElemAnd`, `ElemOr`, and `LDiv` which
    /// are rare enough to handle via `EvalExpr`.
    fn is_pure(expr: &Expr) -> bool {
        match expr {
            Expr::Number(_) | Expr::Var(_) => true,
            Expr::UnaryMinus(e) | Expr::UnaryNot(e) => Self::is_pure(e),
            Expr::BinOp(a, op, b) => {
                !matches!(op, Op::ElemAnd | Op::ElemOr | Op::LDiv)
                    && Self::is_pure(a)
                    && Self::is_pure(b)
            }
            _ => false,
        }
    }

    /// Emit native stack opcodes for a pure expression (no `eval_with_io`).
    ///
    /// Panics if `expr` is not pure — callers must check [`is_pure`] first.
    fn compile_native(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(f) => {
                let idx = self.chunk.add_const(Value::Scalar(*f));
                self.emit(Instr::with_u16(Opcode::PushConst, idx));
            }
            Expr::Var(name) => {
                let idx = self.chunk.name_idx(name);
                self.emit(Instr::with_u16(Opcode::LoadVar, idx));
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
                    Op::Add    => Opcode::Add,
                    Op::Sub    => Opcode::Sub,
                    Op::Mul    => Opcode::Mul,
                    Op::Div    => Opcode::Div,
                    Op::Pow    => Opcode::Pow,
                    Op::ElemMul => Opcode::ElemMul,
                    Op::ElemDiv => Opcode::ElemDiv,
                    Op::ElemPow => Opcode::ElemPow,
                    Op::Eq     => Opcode::Eq,
                    Op::NotEq  => Opcode::Ne,
                    Op::Lt     => Opcode::Lt,
                    Op::LtEq   => Opcode::Le,
                    Op::Gt     => Opcode::Gt,
                    Op::GtEq   => Opcode::Ge,
                    Op::And    => Opcode::And,
                    Op::Or     => Opcode::Or,
                    // ElemAnd, ElemOr, LDiv are excluded by is_pure(); this arm
                    // is unreachable in correct usage.
                    Op::ElemAnd | Op::ElemOr | Op::LDiv => unreachable!(
                        "compile_native: is_pure should have excluded this op"
                    ),
                };
                self.emit(Instr::no_arg(opcode));
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
