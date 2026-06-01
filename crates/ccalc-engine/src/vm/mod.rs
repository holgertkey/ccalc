//! Bytecode VM: types shared between the compiler and the executor.
//!
//! [`Chunk`] holds a flat bytecode sequence together with constant, name, and
//! expression pools.  [`Instr`] is a fixed-width 8-byte instruction.
//! [`IterState`] drives `for` loops inside the VM.

pub mod compile;
pub mod exec;

use crate::env::Value;
use crate::eval::Expr;
use ndarray::Array2;

// ── Opcode ───────────────────────────────────────────────────────────────────

/// Bytecode opcode — one byte, leaving 7 bytes for the payload.
///
/// Payload encoding per opcode is documented inline.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    // ── Constants & variables ─────────────────── payload ──────────────────
    /// Push `chunk.consts[u16]` onto the operand stack.
    PushConst  = 0,   // [u16 idx, 0, 0, 0, 0]
    /// Push a clone of `env[chunk.names[u16]]` onto the stack.
    LoadVar    = 1,   // [u16 idx, 0, 0, 0, 0]
    /// Pop stack-top → `env[chunk.names[u16]]`; u8 flag: 1 = silent (no print).
    StoreVar   = 2,   // [u16 idx, u8 silent, 0, 0, 0]
    /// Copy stack-top (no pop) into `env["ans"]`.
    UpdateAns  = 3,   // [0, 0, 0, 0, 0, 0, 0]
    /// Pop and discard stack-top.
    Pop        = 4,   // [0, 0, 0, 0, 0, 0, 0]
    /// Print stack-top without variable label, then pop.
    Print      = 5,   // [0, 0, 0, 0, 0, 0, 0]

    // ── Arithmetic ──────────────────────────────────────────────────────────
    /// Pop b, pop a, push a + b.
    Add        = 10,
    /// Pop b, pop a, push a − b.
    Sub        = 11,
    /// Pop b, pop a, push a * b (scalar multiply or matrix product).
    Mul        = 12,
    /// Pop b, pop a, push a / b.
    Div        = 13,
    /// Pop b, pop a, push a ^ b.
    Pow        = 14,
    /// Pop b, pop a, push a .* b (element-wise).
    ElemMul    = 15,
    /// Pop b, pop a, push a ./ b (element-wise).
    ElemDiv    = 16,
    /// Pop b, pop a, push a .^ b (element-wise).
    ElemPow    = 17,
    /// Negate stack-top in place (replace with −top).
    Neg        = 18,

    // ── Comparison / logical ────────────────────────────────────────────────
    /// Logical NOT: replace top with 0.0 if truthy, 1.0 otherwise.
    Not        = 20,
    /// Pop b, a; push 1.0 if a == b, else 0.0.
    Eq         = 21,
    /// Pop b, a; push 1.0 if a ~= b, else 0.0.
    Ne         = 22,
    /// Pop b, a; push 1.0 if a < b, else 0.0.
    Lt         = 23,
    /// Pop b, a; push 1.0 if a <= b, else 0.0.
    Le         = 24,
    /// Pop b, a; push 1.0 if a > b, else 0.0.
    Gt         = 25,
    /// Pop b, a; push 1.0 if a >= b, else 0.0.
    Ge         = 26,
    /// Pop b, a; push 1.0 if both truthy, else 0.0 (element-wise `&&`).
    And        = 27,
    /// Pop b, a; push 1.0 if either truthy, else 0.0 (element-wise `||`).
    Or         = 28,

    // ── Control flow ────────────────────────────────────────────────────────
    /// Unconditional jump: `ip += 1 + i32_payload`.
    Jump       = 30,  // [i32 offset, 0, 0, 0]
    /// Pop top; if falsy, jump by i32 offset, else advance.
    JumpFalsy  = 31,  // [i32 offset, 0, 0, 0]
    /// Pop top; if truthy, jump by i32 offset, else advance.
    JumpTruthy = 32,  // [i32 offset, 0, 0, 0]

    // ── For-loop iteration ───────────────────────────────────────────────────
    /// Pop stack-top; build an [`IterState`] and push onto the iterator stack.
    PushIter   = 40,
    /// Advance the top iterator: if exhausted, pop it and jump by i32 exit_offset;
    /// else store the next column in `chunk.names[u16 var_idx]` and advance ip.
    IterNext   = 41,  // [u16 var_idx, i32 exit_offset, 0]
    /// Pop the top iterator (emitted on `break`).
    PopIter    = 42,

    // ── Deferred expression evaluation ──────────────────────────────────────
    /// Evaluate `chunk.exprs[u16]` via `eval_with_io` and push the result.
    ///
    /// Used for expressions too complex to compile natively (function calls,
    /// ranges, matrix literals, indexing, etc.).
    EvalExpr   = 60,  // [u16 expr_idx, 0, 0, 0, 0]

    // ── Indexed assignment ────────────────────────────────────────────────────
    /// Pop RHS from stack; call `exec_index_set(chunk.names[u16], chunk.index_sets[u16], rhs, …)`.
    /// u8 `silent` flag controls whether the updated variable is printed.
    IndexSetOp = 80,  // [u16 name_idx, u16 iset_idx, u8 silent, 0, 0]

    // ── Function definition ───────────────────────────────────────────────────
    /// Register `chunk.consts[u16 const_idx]` (a `Value::Function`) into
    /// `env[chunk.names[u16 name_idx]]`.  Replicates the tree-walker
    /// `Stmt::FunctionDef` handler.
    DefineFunc = 75,  // [u16 name_idx, u16 const_idx, 0, 0, 0]

    // ── Function control ─────────────────────────────────────────────────────
    /// Return from the current function body (unwinds vm_exec).
    Return     = 70,
}

// ── Instr ────────────────────────────────────────────────────────────────────

/// A single bytecode instruction — always exactly 8 bytes on every platform.
///
/// `op` occupies one byte; `payload` holds up to 7 bytes of arguments encoded
/// in little-endian order.  The exact layout per opcode is described on [`Opcode`].
pub struct Instr {
    /// The instruction opcode.
    pub op: Opcode,
    payload: [u8; 7],
}

// Compile-time size guarantee — catches layout regressions immediately.
const _INSTR_SIZE: () = assert!(std::mem::size_of::<Instr>() == 8);

impl Instr {
    /// Create an instruction with no payload (all payload bytes = 0).
    pub fn no_arg(op: Opcode) -> Self {
        Self { op, payload: [0; 7] }
    }

    /// Create an instruction with a single u16 argument at payload[0..2].
    pub fn with_u16(op: Opcode, v: u16) -> Self {
        let mut p = [0u8; 7];
        p[0..2].copy_from_slice(&v.to_le_bytes());
        Self { op, payload: p }
    }

    /// Create an instruction with a single i32 argument at payload[0..4].
    pub fn with_i32(op: Opcode, v: i32) -> Self {
        let mut p = [0u8; 7];
        p[0..4].copy_from_slice(&v.to_le_bytes());
        Self { op, payload: p }
    }

    /// Create an instruction with a u16 at payload[0..2] and a u8 at payload[2].
    ///
    /// Used by [`Opcode::StoreVar`]: `idx` = name pool index, `flag` = silent bit.
    pub fn with_u16_u8(op: Opcode, idx: u16, flag: u8) -> Self {
        let mut p = [0u8; 7];
        p[0..2].copy_from_slice(&idx.to_le_bytes());
        p[2] = flag;
        Self { op, payload: p }
    }

    /// Create an instruction with u16 at [0..2], u16 at [2..4], and u8 at [4].
    ///
    /// Used by [`Opcode::IndexSetOp`]: `a` = name_idx, `b` = iset_idx, `c` = silent.
    pub fn with_u16_u16_u8(op: Opcode, a: u16, b: u16, c: u8) -> Self {
        let mut p = [0u8; 7];
        p[0..2].copy_from_slice(&a.to_le_bytes());
        p[2..4].copy_from_slice(&b.to_le_bytes());
        p[4] = c;
        Self { op, payload: p }
    }

    /// Create an instruction with two u16 values at payload[0..2] and payload[2..4].
    ///
    /// Used by [`Opcode::DefineFunc`]: `a` = name_idx, `b` = const_idx.
    pub fn with_u16_u16(op: Opcode, a: u16, b: u16) -> Self {
        let mut p = [0u8; 7];
        p[0..2].copy_from_slice(&a.to_le_bytes());
        p[2..4].copy_from_slice(&b.to_le_bytes());
        Self { op, payload: p }
    }

    /// Create an instruction with a u16 at payload[0..2] and an i32 at payload[2..6].
    ///
    /// Used by [`Opcode::IterNext`]: `u` = var_idx, `i` = exit_offset.
    pub fn with_u16_i32(op: Opcode, u: u16, i: i32) -> Self {
        let mut p = [0u8; 7];
        p[0..2].copy_from_slice(&u.to_le_bytes());
        p[2..6].copy_from_slice(&i.to_le_bytes());
        Self { op, payload: p }
    }

    // ── Payload readers (little-endian) ──────────────────────────────────────

    /// Read a u16 from payload[0..2].
    pub fn u16_arg(&self) -> u16 {
        u16::from_le_bytes(self.payload[0..2].try_into().unwrap())
    }

    /// Read an i32 from payload[0..4].
    pub fn i32_arg(&self) -> i32 {
        i32::from_le_bytes(self.payload[0..4].try_into().unwrap())
    }

    /// Read a u8 from `payload[i]`.
    pub fn u8_at(&self, i: usize) -> u8 {
        self.payload[i]
    }

    /// Read a u16 from `payload[i..i+2]`.
    pub fn u16_at(&self, i: usize) -> u16 {
        u16::from_le_bytes(self.payload[i..i + 2].try_into().unwrap())
    }

    /// Read an i32 from `payload[i..i+4]`.
    pub fn i32_at(&self, i: usize) -> i32 {
        i32::from_le_bytes(self.payload[i..i + 4].try_into().unwrap())
    }

    // ── Payload writers (little-endian) — used for backpatching ──────────────

    /// Overwrite payload[0..4] with a new i32 (used to backpatch jump offsets).
    pub fn set_i32(&mut self, v: i32) {
        self.payload[0..4].copy_from_slice(&v.to_le_bytes());
    }

    /// Overwrite a u16 at payload[0..2] and an i32 at payload[2..6].
    ///
    /// Used to backpatch [`Opcode::IterNext`] exit offsets.
    pub fn set_u16_i32(&mut self, u: u16, i: i32) {
        self.payload[0..2].copy_from_slice(&u.to_le_bytes());
        self.payload[2..6].copy_from_slice(&i.to_le_bytes());
    }
}

// ── Chunk ─────────────────────────────────────────────────────────────────────

/// Compiled bytecode for a statement block.
///
/// Produced by [`compile`](crate::vm::compile::compile) and consumed by
/// [`vm_exec`](crate::vm::exec::vm_exec).
pub struct Chunk {
    /// Flat sequence of bytecode instructions.
    pub code:   Vec<Instr>,
    /// Constant pool: scalar literals and other pre-computed values.
    pub consts: Vec<Value>,
    /// Name pool: variable names referenced by `LoadVar`, `StoreVar`, `IterNext`.
    pub names:  Vec<String>,
    /// Expression pool: AST nodes evaluated via `eval_with_io` at runtime.
    ///
    /// Used by [`Opcode::EvalExpr`] for function calls, ranges, matrix literals,
    /// indexing, and other expressions that cannot be compiled to native opcodes.
    pub exprs:  Vec<Expr>,
    /// Index expression pool: lists of index `Expr` nodes for `IndexSetOp`.
    ///
    /// Stores the index expressions of `A(i,j) = v` style statements verbatim
    /// so they can be passed to `exec_index_set` at runtime.
    pub index_sets: Vec<Vec<Expr>>,
    /// Source line number for each instruction (1-based, 0 = unknown).
    ///
    /// Used to annotate runtime errors with `near line N`, matching the
    /// behaviour of the tree-walking interpreter.
    pub lines:  Vec<usize>,
}

impl Chunk {
    /// Create an empty chunk.
    pub fn new() -> Self {
        Self {
            code:       Vec::new(),
            consts:     Vec::new(),
            names:      Vec::new(),
            exprs:      Vec::new(),
            index_sets: Vec::new(),
            lines:      Vec::new(),
        }
    }

    /// Add a constant to the pool (no deduplication) and return its index.
    pub fn add_const(&mut self, v: Value) -> u16 {
        let idx = self.consts.len() as u16;
        self.consts.push(v);
        idx
    }

    /// Return the index of `name` in the name pool, inserting it if absent.
    pub fn name_idx(&mut self, name: &str) -> u16 {
        if let Some(pos) = self.names.iter().position(|n| n == name) {
            return pos as u16;
        }
        let idx = self.names.len() as u16;
        self.names.push(name.to_string());
        idx
    }

    /// Add an expression to the pool and return its index.
    pub fn add_expr(&mut self, e: Expr) -> u16 {
        let idx = self.exprs.len() as u16;
        self.exprs.push(e);
        idx
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

// ── CompileError ──────────────────────────────────────────────────────────────

/// Returned by the compiler when a statement block cannot be lowered to bytecode.
///
/// The caller must fall back to the tree-walking interpreter.
#[derive(Debug)]
pub enum CompileError {
    /// The block contains a construct not yet supported by the compiler.
    Unsupported,
}

// ── IterState ─────────────────────────────────────────────────────────────────

/// Runtime state for a single active `for` loop.
///
/// Holds a pre-materialized vector of column values extracted from the range
/// expression at loop entry, matching the semantics of the tree-walking interpreter.
pub struct IterState {
    vals: Vec<Value>,
    pos:  usize,
}

impl IterState {
    /// Build an [`IterState`] from the evaluated range value.
    ///
    /// - `Scalar(n)` → one-element iterator of `Scalar(n)`.
    /// - `Matrix` → iterator over columns: row vectors yield scalars, taller
    ///   matrices yield M×1 column vectors.
    ///
    /// Returns an error for non-iterable types.
    pub fn from_value(val: Value) -> Result<Self, String> {
        let vals = match val {
            Value::Scalar(n) => vec![Value::Scalar(n)],
            Value::Matrix(m) => {
                let nrows = m.nrows();
                let ncols = m.ncols();
                (0..ncols)
                    .map(|j| {
                        if nrows == 1 {
                            Value::Scalar(m[[0, j]])
                        } else {
                            let mut col = Array2::zeros((nrows, 1));
                            for i in 0..nrows {
                                col[[i, 0]] = m[[i, j]];
                            }
                            Value::Matrix(col)
                        }
                    })
                    .collect()
            }
            _ => return Err("'for': range must evaluate to a scalar or matrix".to_string()),
        };
        Ok(IterState { vals, pos: 0 })
    }

    /// Advance the iterator and return the next value, or `None` when exhausted.
    pub fn next_val(&mut self) -> Option<Value> {
        if self.pos < self.vals.len() {
            let v = self.vals[self.pos].clone();
            self.pos += 1;
            Some(v)
        } else {
            None
        }
    }
}
