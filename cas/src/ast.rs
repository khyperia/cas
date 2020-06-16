use std::fmt::Display;

#[derive(Clone)]
pub struct Ast<'a> {
    pub span: &'a str,
    pub kind: AstKind,
    pub args: Vec<Self>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum AstKind {
    Variable(String),
    Number(i64),
    Negate,
    Sin,
    Cos,
    Tan,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

impl Display for Ast<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            AstKind::Variable(var) => write!(f, "{}", var),
            AstKind::Number(num) => write!(f, "{}", num),
            AstKind::Negate => write!(f, "-{}", self.args[0]),
            AstKind::Sin => write!(f, "sin {}", self.args[0]),
            AstKind::Cos => write!(f, "cos {}", self.args[0]),
            AstKind::Tan => write!(f, "tan {}", self.args[0]),
            AstKind::Add => write!(f, "({} + {})", self.args[0], self.args[1]),
            AstKind::Sub => write!(f, "({} - {})", self.args[0], self.args[1]),
            AstKind::Mul => write!(f, "({} * {})", self.args[0], self.args[1]),
            AstKind::Div => write!(f, "({} / {})", self.args[0], self.args[1]),
            AstKind::Mod => write!(f, "({} % {})", self.args[0], self.args[1]),
            AstKind::Pow => write!(f, "({} ^ {})", self.args[0], self.args[1]),
        }
    }
}
