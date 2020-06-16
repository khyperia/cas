use std::fmt::Display;

#[derive(Clone)]
pub enum Ast<'a> {
    Number {
        span: &'a str,
        value: isize,
    },
    Variable {
        span: &'a str,
        name: String,
    },
    UnaryOp {
        span: &'a str,
        op: UnaryOpKind,
        value: Box<Self>,
    },
    BinaryOp {
        span: &'a str,
        op: BinaryOpKind,
        left: Box<Self>,
        right: Box<Self>,
    },
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum UnaryOpKind {
    Negate,
    Sin,
    Cos,
    Tan,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum BinaryOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

impl Display for Ast<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ast::Number { value, .. } => write!(f, "{}", value),
            Ast::Variable { name, .. } => write!(f, "{}", name),
            Ast::UnaryOp { op, value, .. } => write!(f, "{}{}", op, value),
            Ast::BinaryOp {
                op, left, right, ..
            } => write!(f, "({} {} {})", left, op, right),
        }
    }
}

impl Display for UnaryOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOpKind::Negate => write!(f, "-"),
            UnaryOpKind::Sin => write!(f, "sin "),
            UnaryOpKind::Cos => write!(f, "cos "),
            UnaryOpKind::Tan => write!(f, "tan "),
        }
    }
}

impl Display for BinaryOpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOpKind::Add => write!(f, "+"),
            BinaryOpKind::Sub => write!(f, "-"),
            BinaryOpKind::Mul => write!(f, "*"),
            BinaryOpKind::Div => write!(f, "/"),
            BinaryOpKind::Mod => write!(f, "%"),
            BinaryOpKind::Pow => write!(f, "^"),
        }
    }
}
