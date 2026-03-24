use crate::eval::format_number;

pub struct Memory {
    cells: [f64; 9],
}

impl Memory {
    pub fn new() -> Self {
        Memory { cells: [0.0; 9] }
    }

    pub fn get(&self, idx: usize) -> f64 {
        self.cells[idx]
    }

    pub fn set(&mut self, idx: usize, val: f64) {
        self.cells[idx] = val;
    }

    pub fn clear_one(&mut self, idx: usize) {
        self.cells[idx] = 0.0;
    }

    pub fn clear_all(&mut self) {
        self.cells = [0.0; 9];
    }

    pub fn display_nonzero(&self, fmt: impl Fn(f64) -> String) {
        for (i, &val) in self.cells.iter().enumerate() {
            if val != 0.0 {
                println!("m{}: {}", i + 1, fmt(val));
            }
        }
    }
}

/// Command that covers the entire input line (no preceding expression).
pub enum StandaloneCmd {
    StoreAcc(usize),  // m[1-9]:  store accumulator → cell
    ClearOne(usize),  // mc[1-9]: clear cell
}

#[derive(Debug, Clone, Copy)]
pub enum CompoundOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

/// Memory directive found at the end of an expression string.
pub enum Directive {
    Store(usize),                  // m[1-9]:   cell = result
    Compound(usize, CompoundOp),   // m[1-9]OP: cell = cell OP result
}

/// Returns a `StandaloneCmd` if the entire input matches a known memory command,
/// or `None` if the input should be treated as an expression.
pub fn parse_standalone_cmd(input: &str) -> Option<StandaloneCmd> {
    let b = input.as_bytes();
    match b {
        [b'm', d] if is_mem_digit(*d) => Some(StandaloneCmd::StoreAcc(mem_idx(*d))),
        [b'm', b'c', d] if is_mem_digit(*d) => Some(StandaloneCmd::ClearOne(mem_idx(*d))),
        _ => None,
    }
}

/// Extracts a trailing memory directive from an expression string.
///
/// The last space-separated token is treated as a directive only when the
/// character directly before it is NOT an operator (`+`, `-`, `*`, `/`, `(`),
/// which would mean the token is an operand, not a command.
///
/// Returns `(expression_part, directive)`.
pub fn extract_directive(input: &str) -> (&str, Option<Directive>) {
    if let Some(last_space) = input.rfind(' ') {
        let last_token = &input[last_space + 1..];
        let before = &input[..last_space];
        let trimmed_before = before.trim_end();

        if !trimmed_before.is_empty() {
            let last_char = trimmed_before.chars().last();
            let after_operator =
                matches!(last_char, Some('+' | '-' | '*' | '/' | '^' | '%' | '('));

            if !after_operator {
                if let Some(directive) = parse_directive_token(last_token) {
                    return (trimmed_before, Some(directive));
                }
            }
        }
    }
    (input, None)
}

/// Replaces `m[1-9]` references in an expression string with their numeric values.
///
/// Returns `(expr_for_parsing, display_str)` where `display_str` is `Some` only
/// when at least one substitution was performed.
pub fn expand_memory_refs(input: &str, memory: &Memory) -> (String, Option<String>) {
    let mut result = String::with_capacity(input.len());
    let mut has_substitutions = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == 'm' {
            if let Some(&next) = chars.peek() {
                if is_mem_digit(next as u8) {
                    chars.next();
                    let idx = mem_idx(next as u8);
                    result.push_str(&format_number(memory.get(idx)));
                    has_substitutions = true;
                    continue;
                }
            }
        }
        result.push(c);
    }

    let display = if has_substitutions {
        Some(result.clone())
    } else {
        None
    };

    (result, display)
}

fn parse_directive_token(token: &str) -> Option<Directive> {
    let b = token.as_bytes();
    match b {
        [b'm', d] if is_mem_digit(*d) => Some(Directive::Store(mem_idx(*d))),
        [b'm', d, op] if is_mem_digit(*d) => {
            let idx = mem_idx(*d);
            let compound_op = match op {
                b'+' => CompoundOp::Add,
                b'-' => CompoundOp::Sub,
                b'*' => CompoundOp::Mul,
                b'/' => CompoundOp::Div,
                b'%' => CompoundOp::Mod,
                b'^' => CompoundOp::Pow,
                _ => return None,
            };
            Some(Directive::Compound(idx, compound_op))
        }
        _ => None,
    }
}

fn is_mem_digit(b: u8) -> bool {
    b.is_ascii_digit() && b != b'0'
}

fn mem_idx(b: u8) -> usize {
    (b - b'1') as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Memory struct ────────────────────────────────────────────────────────

    #[test]
    fn test_memory_new_all_zero() {
        let m = Memory::new();
        for i in 0..9 {
            assert_eq!(m.get(i), 0.0);
        }
    }

    #[test]
    fn test_memory_set_and_get() {
        let mut m = Memory::new();
        m.set(0, 42.0);
        assert_eq!(m.get(0), 42.0);
        m.set(8, -3.5);
        assert_eq!(m.get(8), -3.5);
    }

    #[test]
    fn test_memory_clear_one() {
        let mut m = Memory::new();
        m.set(2, 99.0);
        m.clear_one(2);
        assert_eq!(m.get(2), 0.0);
    }

    #[test]
    fn test_memory_clear_all() {
        let mut m = Memory::new();
        for i in 0..9 {
            m.set(i, (i + 1) as f64);
        }
        m.clear_all();
        for i in 0..9 {
            assert_eq!(m.get(i), 0.0);
        }
    }

    // ── parse_standalone_cmd ─────────────────────────────────────────────────

    #[test]
    fn test_standalone_store_all_cells() {
        for d in b'1'..=b'9' {
            let input = format!("m{}", d as char);
            let cmd = parse_standalone_cmd(&input);
            assert!(matches!(cmd, Some(StandaloneCmd::StoreAcc(i)) if i == (d - b'1') as usize));
        }
    }

    #[test]
    fn test_standalone_clear_one_all_cells() {
        for d in b'1'..=b'9' {
            let input = format!("mc{}", d as char);
            let cmd = parse_standalone_cmd(&input);
            assert!(matches!(cmd, Some(StandaloneCmd::ClearOne(i)) if i == (d - b'1') as usize));
        }
    }

    #[test]
    fn test_standalone_rejects_invalid() {
        assert!(parse_standalone_cmd("m0").is_none());
        assert!(parse_standalone_cmd("m10").is_none());
        assert!(parse_standalone_cmd("mb1").is_none());
        assert!(parse_standalone_cmd("5 m1").is_none());
        assert!(parse_standalone_cmd("m").is_none());
        assert!(parse_standalone_cmd("mc").is_none());
    }

    // ── extract_directive ────────────────────────────────────────────────────

    #[test]
    fn test_extract_directive_store() {
        let (expr, dir) = extract_directive("(1+1)*3 m1");
        assert_eq!(expr, "(1+1)*3");
        assert!(matches!(dir, Some(Directive::Store(0))));
    }

    #[test]
    fn test_extract_directive_compound_add() {
        let (expr, dir) = extract_directive("2 m1+");
        assert_eq!(expr, "2");
        assert!(matches!(dir, Some(Directive::Compound(0, CompoundOp::Add))));
    }

    #[test]
    fn test_extract_directive_compound_sub() {
        let (expr, dir) = extract_directive("5 m3-");
        assert_eq!(expr, "5");
        assert!(matches!(dir, Some(Directive::Compound(2, CompoundOp::Sub))));
    }

    #[test]
    fn test_extract_directive_compound_mul() {
        let (expr, dir) = extract_directive("3 m2*");
        assert_eq!(expr, "3");
        assert!(matches!(dir, Some(Directive::Compound(1, CompoundOp::Mul))));
    }

    #[test]
    fn test_extract_directive_compound_div() {
        let (expr, dir) = extract_directive("4 m1/");
        assert_eq!(expr, "4");
        assert!(matches!(dir, Some(Directive::Compound(0, CompoundOp::Div))));
    }

    #[test]
    fn test_extract_directive_compound_mod() {
        let (expr, dir) = extract_directive("3 m1%");
        assert_eq!(expr, "3");
        assert!(matches!(dir, Some(Directive::Compound(0, CompoundOp::Mod))));
    }

    #[test]
    fn test_extract_directive_compound_pow() {
        let (expr, dir) = extract_directive("2 m1^");
        assert_eq!(expr, "2");
        assert!(matches!(dir, Some(Directive::Compound(0, CompoundOp::Pow))));
    }

    #[test]
    fn test_extract_directive_compound_all_cells() {
        for d in b'1'..=b'9' {
            let input = format!("10 m{}+", d as char);
            let (expr, dir) = extract_directive(&input);
            assert_eq!(expr, "10");
            let idx = (d - b'1') as usize;
            assert!(matches!(dir, Some(Directive::Compound(i, CompoundOp::Add)) if i == idx));
        }
    }

    #[test]
    fn test_extract_directive_no_directive_after_operator() {
        // operator before last token → directive NOT extracted
        let (expr, dir) = extract_directive("5 + m1+");
        assert_eq!(expr, "5 + m1+");
        assert!(dir.is_none());

        let (expr, dir) = extract_directive("5 ^ m1^");
        assert_eq!(expr, "5 ^ m1^");
        assert!(dir.is_none());
    }

    #[test]
    fn test_extract_directive_compound_with_mem_ref_expr() {
        // m2 m1+  →  m1 = m1 + m2
        let (expr, dir) = extract_directive("m2 m1+");
        assert_eq!(expr, "m2");
        assert!(matches!(dir, Some(Directive::Compound(0, CompoundOp::Add))));
    }

    #[test]
    fn test_extract_directive_no_directive_after_plus() {
        let (expr, dir) = extract_directive("5 + m1");
        assert_eq!(expr, "5 + m1");
        assert!(dir.is_none());
    }

    #[test]
    fn test_extract_directive_no_directive_after_minus() {
        let (expr, dir) = extract_directive("10 - m2");
        assert_eq!(expr, "10 - m2");
        assert!(dir.is_none());
    }

    #[test]
    fn test_extract_directive_no_directive_after_star() {
        let (expr, dir) = extract_directive("3 * m1");
        assert_eq!(expr, "3 * m1");
        assert!(dir.is_none());
    }

    #[test]
    fn test_extract_directive_chained_mem_refs() {
        let (expr, dir) = extract_directive("m1 + 8 + m1");
        assert_eq!(expr, "m1 + 8 + m1");
        assert!(dir.is_none());
    }

    #[test]
    fn test_extract_directive_copy_cell() {
        let (expr, dir) = extract_directive("m1 m2");
        assert_eq!(expr, "m1");
        assert!(matches!(dir, Some(Directive::Store(1))));
    }

    #[test]
    fn test_extract_directive_no_space() {
        let (expr, dir) = extract_directive("5+3");
        assert_eq!(expr, "5+3");
        assert!(dir.is_none());
    }

    // ── expand_memory_refs ───────────────────────────────────────────────────

    #[test]
    fn test_expand_no_refs() {
        let m = Memory::new();
        let (expr, display) = expand_memory_refs("5 + 3 * 2", &m);
        assert_eq!(expr, "5 + 3 * 2");
        assert!(display.is_none());
    }

    #[test]
    fn test_expand_single_ref() {
        let mut m = Memory::new();
        m.set(0, 6.0);
        let (expr, display) = expand_memory_refs("m1 + 8", &m);
        assert_eq!(expr, "6 + 8");
        assert_eq!(display, Some("6 + 8".to_string()));
    }

    #[test]
    fn test_expand_multiple_refs() {
        let mut m = Memory::new();
        m.set(0, 6.0);
        let (expr, display) = expand_memory_refs("m1 + 8 + m1", &m);
        assert_eq!(expr, "6 + 8 + 6");
        assert_eq!(display, Some("6 + 8 + 6".to_string()));
    }

    #[test]
    fn test_expand_different_cells() {
        let mut m = Memory::new();
        m.set(0, 10.0);
        m.set(1, 20.0);
        let (expr, display) = expand_memory_refs("m1 + m2", &m);
        assert_eq!(expr, "10 + 20");
        assert_eq!(display, Some("10 + 20".to_string()));
    }

    #[test]
    fn test_expand_zero_cell() {
        let m = Memory::new();
        let (expr, display) = expand_memory_refs("m5 + 1", &m);
        assert_eq!(expr, "0 + 1");
        assert_eq!(display, Some("0 + 1".to_string()));
    }

    #[test]
    fn test_expand_ignores_m_without_digit() {
        let m = Memory::new();
        let (expr, display) = expand_memory_refs("5 + 3", &m);
        assert_eq!(expr, "5 + 3");
        assert!(display.is_none());
    }
}
