use crate::{parse, Ast, BinaryOpKind, UnaryOpKind};
use lazy_static::lazy_static;
use std::{borrow::Cow, collections::hash_map::HashMap};

struct Rule {
    pattern: Ast<'static>,
    replacement: Ast<'static>,
}

fn rules() -> Vec<Rule> {
    include_str!("rules_db.txt")
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_rule)
        .collect()
}

fn parse_rule(line: &'static str) -> Rule {
    let mut split = line.splitn(2, '=');
    let pattern = parse(split.next().unwrap());
    let replacement = parse(split.next().unwrap());
    Rule {
        pattern,
        replacement,
    }
}

lazy_static! {
    static ref RULES: Vec<Rule> = rules();
}

fn do_match<'a, 'b>(
    pattern: &Ast,
    ast: &'a Ast<'b>,
    matches: &mut HashMap<String, &'a Ast<'b>>,
) -> bool {
    match (pattern, ast) {
        (Ast::Variable { name, .. }, rhs) => {
            matches.insert(name.clone(), rhs);
            true
        }
        (Ast::Number { value: left, .. }, Ast::Number { value: right, .. }) => left == right,
        (
            Ast::UnaryOp {
                op: pattern_op,
                value: pattern_value,
                ..
            },
            Ast::UnaryOp {
                op: ast_op,
                value: ast_value,
                ..
            },
        ) if *pattern_op == *ast_op => do_match(pattern_value, ast_value, matches),
        (
            Ast::BinaryOp {
                op: pattern_op,
                left: pattern_left,
                right: pattern_right,
                ..
            },
            Ast::BinaryOp {
                op: ast_op,
                left: ast_left,
                right: ast_right,
                ..
            },
        ) if *pattern_op == *ast_op => {
            do_match(pattern_left, ast_left, matches) && do_match(pattern_right, ast_right, matches)
        }
        _ => false,
    }
}

fn do_replacement<'a>(replacement: &Ast<'a>, matches: &HashMap<String, &Ast<'a>>) -> Ast<'a> {
    match replacement {
        &Ast::Number { span, value } => Ast::Number { span, value },
        Ast::Variable { span, name } => {
            if let Some(&thing) = matches.get(name) {
                thing.clone()
            } else {
                // TODO: Is this okay?
                Ast::Variable {
                    span,
                    name: name.clone(),
                }
            }
        }
        Ast::UnaryOp { span, op, value } => {
            // TODO: Wrong span
            Ast::UnaryOp {
                span,
                op: *op,
                value: Box::new(do_replacement(value, matches)),
            }
        }
        Ast::BinaryOp {
            span,
            op,
            left,
            right,
        } => {
            // TODO: Wrong span
            Ast::BinaryOp {
                span,
                op: *op,
                left: Box::new(do_replacement(left, matches)),
                right: Box::new(do_replacement(right, matches)),
            }
        }
    }
}

fn builtin_binop(op: BinaryOpKind, left: isize, right: isize) -> isize {
    match op {
        BinaryOpKind::Add => left + right,
        BinaryOpKind::Sub => left - right,
        BinaryOpKind::Mul => left * right,
        BinaryOpKind::Div => left / right,
        BinaryOpKind::Mod => left % right,
        // TODO
        BinaryOpKind::Pow => left.pow(right as u32),
    }
}

// https://github.com/frewsxcv/rust-gcd/blob/master/src/lib.rs
fn gcd_binary(mut u: usize, mut v: usize) -> usize {
    if u == 0 {
        return v;
    }
    if v == 0 {
        return u;
    }
    let shift = (u | v).trailing_zeros();
    u >>= shift;
    v >>= shift;
    u >>= u.trailing_zeros();
    loop {
        v >>= v.trailing_zeros();
        if u > v {
            //XOR swap algorithm
            v ^= u;
            u ^= v;
            v ^= u;
        }
        v -= u; // Here v >= u.
        if v == 0 {
            break;
        }
    }
    u << shift
}

fn simplify(mut num: isize, mut denom: isize) -> Option<(isize, isize)> {
    let orig = (num, denom);
    if denom.is_negative() {
        num = -num;
    }
    let gcd = gcd_binary(num.abs() as usize, denom as usize);
    if gcd != 1 {
        num /= gcd as isize;
        denom /= gcd as isize;
    }
    if (num, denom) != orig {
        Some((num, denom))
    } else {
        None
    }
}

fn builtins<'a>(ast: &Ast<'a>) -> Option<Ast<'a>> {
    match ast {
        Ast::BinaryOp {
            span,
            op,
            left,
            right,
        } => match (&**left, &**right) {
            (Ast::Number { value: left, .. }, Ast::Number { value: right, .. }) => {
                if *op == BinaryOpKind::Div {
                    if let Some((num, denom)) = simplify(*left, *right) {
                        Some(Ast::BinaryOp {
                            span,
                            op: BinaryOpKind::Div,
                            left: Box::new(Ast::Number { span, value: num }),
                            right: Box::new(Ast::Number { span, value: denom }),
                        })
                    } else {
                        None
                    }
                } else {
                    Some(Ast::Number {
                        span,
                        value: builtin_binop(*op, *left, *right),
                    })
                }
            }
            _ => None,
        },
        Ast::UnaryOp {
            span,
            op: UnaryOpKind::Negate,
            value,
        } => match &**value {
            Ast::Number { value, .. } => Some(Ast::Number {
                span,
                value: -value,
            }),
            _ => None,
        },
        _ => None,
    }
}

fn rewrite_one<'a>(ast: &Ast<'a>) -> Option<Ast<'a>> {
    let mut matches = HashMap::new();
    for rule in RULES.iter() {
        matches.clear();
        if do_match(&rule.pattern, &ast, &mut matches) {
            return Some(do_replacement(&rule.replacement, &matches));
        }
    }
    builtins(ast)
}

pub fn recurse<'a>(ast: &Ast<'a>) -> Option<Ast<'a>> {
    match ast {
        Ast::UnaryOp { span, op, value } => Some(Ast::UnaryOp {
            span,
            op: *op,
            value: Box::new(rewrite(&**value)?),
        }),
        Ast::BinaryOp {
            span,
            op,
            left,
            right,
        } => {
            let new_left = rewrite(&**left);
            let new_right = rewrite(&**right);
            match (new_left, new_right) {
                (Some(new_left), Some(new_right)) => Some(Ast::BinaryOp {
                    span,
                    op: *op,
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                }),
                (Some(new_left), None) => Some(Ast::BinaryOp {
                    span,
                    op: *op,
                    left: Box::new(new_left),
                    right: right.clone(),
                }),
                (None, Some(new_right)) => Some(Ast::BinaryOp {
                    span,
                    op: *op,
                    left: left.clone(),
                    right: Box::new(new_right),
                }),
                (None, None) => None,
            }
        }
        _ => None,
    }
}

pub fn rewrite<'a>(ast: &Ast<'a>) -> Option<Ast<'a>> {
    let mut ast = Cow::Borrowed(ast);
    let mut first = true;
    loop {
        let mut did_any_rewrite = false;
        while let Some(new) = rewrite_one(&ast) {
            println!("rewrite {} to {}", ast, new);
            ast = Cow::Owned(new);
            did_any_rewrite = true;
        }
        if first || did_any_rewrite {
            if let Some(new) = recurse(&ast) {
                ast = Cow::Owned(new);
            } else {
                break;
            }
        } else {
            break;
        }
        first = false;
    }
    match ast {
        Cow::Owned(ast) => Some(ast),
        Cow::Borrowed(_) => None,
    }
}
