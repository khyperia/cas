use crate::{Ast, BinaryOpKind, Lexer, Token};

pub fn parse(source: &str) -> Ast {
    match Parser::new(source) {
        None => panic!("empty source"),
        Some(mut parser) => {
            let result = parser.bin_operator();
            if !parser.lex.is_end() {
                panic!("expected EOF")
            } else {
                result
            }
        }
    }
}

struct Parser<'a> {
    lex: Lexer<'a>,
    current: Token<'a>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Option<Self> {
        let mut lex = Lexer::new(source);
        let current = lex.next()?;
        Some(Self { lex, current })
    }

    fn next(&mut self) -> bool {
        if let Some(next) = self.lex.next() {
            self.current = next;
            true
        } else {
            false
        }
    }

    fn bin_operator(&mut self) -> Ast<'a> {
        let mark = self.lex.mark();
        let primary = self.primary();
        self.bin_operator_1(mark, primary, 0)
    }

    fn bin_operator_1(&mut self, mark: &'a str, mut lhs: Ast<'a>, min_precedence: u32) -> Ast<'a> {
        while let Some(op) = operator_info(&self.current) {
            if op.0 < min_precedence {
                break;
            }
            self.next();
            let inner_mark = self.lex.mark();
            let mut rhs = self.primary();
            while let Some(inner_op) = operator_info(&self.current) {
                if inner_op.0 > op.0 || inner_op.1 && inner_op.0 == op.0 {
                    rhs = self.bin_operator_1(inner_mark, rhs, inner_op.0);
                } else {
                    break;
                }
            }
            lhs = Ast::BinaryOp {
                span: self.lex.span(mark),
                op: op.2,
                left: Box::new(lhs),
                right: Box::new(rhs),
            }
        }
        lhs
    }

    fn primary(&mut self) -> Ast<'a> {
        let result = match &self.current {
            Token::Ident(span, name) => Ast::Variable {
                span,
                name: name.clone(),
            },
            &Token::Number(span, value) => Ast::Number { span, value },
            Token::OpenParen => {
                self.next();
                let result = self.bin_operator();
                if self.current != Token::CloseParen {
                    unimplemented!()
                }
                result
            }
            _ => unimplemented!(),
        };
        self.next();
        result
    }
}

// precedence, right-associative, kind
fn operator_info(token: &Token) -> Option<(u32, bool, BinaryOpKind)> {
    match token {
        Token::Add => Some((1, false, BinaryOpKind::Add)),
        Token::Sub => Some((1, false, BinaryOpKind::Sub)),
        Token::Mul => Some((2, false, BinaryOpKind::Mul)),
        Token::Div => Some((2, false, BinaryOpKind::Div)),
        Token::Mod => Some((2, false, BinaryOpKind::Mod)),
        Token::Pow => Some((3, true, BinaryOpKind::Pow)),
        _ => None,
    }
}
