//! A deliberately small SSA IR.
//!
//! A [`Function`] is one basic block. Values are the instruction indices, so
//! `v0` is `insts[0]`; an SSA producer must emit definitions before uses.
//! There are no phi nodes yet: straight-line code is enough to exercise
//! folding without dragging in dominance bookkeeping.

use std::fmt;

/// Index of a defining instruction, printed as `v<n>`.
pub type ValueId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Shl,
}

impl BinOp {
    /// Parse the mnemonic used in the textual form.
    pub fn parse(token: &str) -> Option<Self> {
        Some(match token {
            "add" => BinOp::Add,
            "sub" => BinOp::Sub,
            "mul" => BinOp::Mul,
            "div" => BinOp::Div,
            "and" => BinOp::And,
            "or" => BinOp::Or,
            "xor" => BinOp::Xor,
            "shl" => BinOp::Shl,
            _ => return None,
        })
    }

    pub fn mnemonic(self) -> &'static str {
        match self {
            BinOp::Add => "add",
            BinOp::Sub => "sub",
            BinOp::Mul => "mul",
            BinOp::Div => "div",
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::Xor => "xor",
            BinOp::Shl => "shl",
        }
    }

    /// Evaluate on two constants.
    ///
    /// `None` means the result is not a defined constant for this pass:
    /// division by zero and shifts of 64 or more bits.
    pub fn eval(self, lhs: u64, rhs: u64) -> Option<u64> {
        Some(match self {
            BinOp::Add => lhs.wrapping_add(rhs),
            BinOp::Sub => lhs.wrapping_sub(rhs),
            BinOp::Mul => lhs.wrapping_mul(rhs),
            BinOp::Div => {
                if rhs == 0 {
                    return None;
                }
                lhs / rhs
            }
            BinOp::And => lhs & rhs,
            BinOp::Or => lhs | rhs,
            BinOp::Xor => lhs ^ rhs,
            BinOp::Shl => {
                if rhs >= 64 {
                    return None;
                }
                lhs << rhs
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inst {
    Const(u64),
    Bin(BinOp, ValueId, ValueId),
    Copy(ValueId),
    Ret(ValueId),
}

impl Inst {
    /// Render one instruction as `v<id> = ...` (or `ret v<id>`).
    pub fn render(&self, id: ValueId) -> String {
        match self {
            Inst::Const(value) => format!("v{id} = const {value}"),
            Inst::Bin(op, lhs, rhs) => format!("v{id} = {} v{lhs} v{rhs}", op.mnemonic()),
            Inst::Copy(src) => format!("v{id} = copy v{src}"),
            Inst::Ret(src) => format!("ret v{src}"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Function {
    pub name: String,
    pub insts: Vec<Inst>,
}

impl Function {
    /// The defining instruction of `id`, if the id is in range.
    pub fn value(&self, id: ValueId) -> Option<&Inst> {
        self.insts.get(id as usize)
    }

    /// Parse the textual form: `fn <name>` header, then one instruction per
    /// line, `;` starts a comment.
    pub fn parse(text: &str) -> Result<Self, ParseError> {
        let mut function = Function::default();
        for (idx, raw) in text.lines().enumerate() {
            let line_no = idx + 1;
            let line = raw.split(';').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if let Some(name) = line.strip_prefix("fn ") {
                function.name = name.trim().to_owned();
                continue;
            }
            let inst = parse_inst(line, function.insts.len() as ValueId)
                .ok_or_else(|| ParseError { line: line_no, text: line.to_owned() })?;
            function.insts.push(inst);
        }
        if function.name.is_empty() {
            return Err(ParseError { line: 1, text: "missing `fn <name>` header".to_owned() });
        }
        Ok(function)
    }

    pub fn to_text(&self) -> String {
        let mut out = format!("fn {}\n", self.name);
        for (id, inst) in self.insts.iter().enumerate() {
            out.push_str(&inst.render(id as ValueId));
            out.push('\n');
        }
        out
    }
}

fn parse_inst(line: &str, id: ValueId) -> Option<Inst> {
    if let Some(rest) = line.strip_prefix("ret ") {
        return Some(Inst::Ret(parse_value(rest)?));
    }
    let (_, rhs) = line.split_once('=')?;
    let tokens: Vec<&str> = rhs.split_whitespace().collect();
    match tokens.as_slice() {
        ["const", value] => Some(Inst::Const(value.parse().ok()?)),
        ["copy", src] => Some(Inst::Copy(parse_value(src)?)),
        [op, lhs, r] => {
            let src = parse_value(lhs)?;
            if src >= id {
                return None;
            }
            let operand = parse_value(r)?;
            if operand >= id {
                return None;
            }
            Some(Inst::Bin(BinOp::parse(op)?, src, operand))
        }
        _ => None,
    }
}

fn parse_value(token: &str) -> Option<ValueId> {
    token.strip_prefix('v')?.parse().ok()
}

#[derive(Clone, Debug)]
pub struct ParseError {
    pub line: usize,
    pub text: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: cannot parse `{}`", self.line, self.text)
    }
}

impl std::error::Error for ParseError {}
