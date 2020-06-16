use crate::{parse, Ast, AstKind};
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
    if let AstKind::Variable(name) = &pattern.kind {
        matches.insert(name.clone(), ast);
        true
    } else if pattern.kind == ast.kind {
        // TODO: varargs
        assert_eq!(pattern.args.len(), ast.args.len());
        let mut zip = pattern.args.iter().zip(&ast.args);
        zip.all(|(subpat, subast)| do_match(subpat, subast, matches))
    } else {
        false
    }
}

fn do_replacement<'a>(replacement: &Ast<'a>, matches: &HashMap<String, &Ast<'a>>) -> Ast<'a> {
    if let AstKind::Variable(name) = &replacement.kind {
        matches[name].clone()
    } else {
        let old_args = replacement.args.iter();
        let args = old_args.map(|arg| do_replacement(arg, matches)).collect();
        Ast {
            span: replacement.span,
            kind: replacement.kind.clone(),
            args,
        }
    }
}

// https://github.com/frewsxcv/rust-gcd/blob/master/src/lib.rs
fn gcd_binary(mut u: u64, mut v: u64) -> u64 {
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

fn simplify(mut num: i64, mut denom: i64) -> Option<(i64, i64)> {
    let orig = (num, denom);
    if denom.is_negative() {
        num = -num;
    }
    let gcd = gcd_binary(num.abs() as u64, denom as u64);
    if gcd != 1 {
        num /= gcd as i64;
        denom /= gcd as i64;
    }
    if (num, denom) != orig {
        Some((num, denom))
    } else {
        None
    }
}

fn build_fraction(span: &str, num: i64, denom: i64) -> Ast {
    Ast {
        span,
        kind: AstKind::Div,
        args: vec![
            Ast {
                span,
                kind: AstKind::Number(num),
                args: Vec::new(),
            },
            Ast {
                span,
                kind: AstKind::Number(denom),
                args: Vec::new(),
            },
        ],
    }
}

fn do_builtin_op<'a>(
    span: &'a str,
    kind: &AstKind,
    mut args: impl Iterator<Item = i64>,
) -> Option<Ast<'a>> {
    let cfold_result = match kind {
        AstKind::Add => args.next().unwrap() + args.next().unwrap(),
        AstKind::Sub => args.next().unwrap() - args.next().unwrap(),
        AstKind::Mul => args.next().unwrap() * args.next().unwrap(),
        AstKind::Mod => args.next().unwrap() % args.next().unwrap(),
        AstKind::Pow => args.next().unwrap().pow(args.next().unwrap() as u32),
        AstKind::Negate => -args.next().unwrap(),
        AstKind::Div => {
            let (num, denom) = simplify(args.next().unwrap(), args.next().unwrap())?;
            return Some(build_fraction(span, num, denom));
        }
        _ => return None,
    };
    Some(Ast {
        span,
        kind: AstKind::Number(cfold_result),
        args: Vec::new(),
    })
}

fn builtins<'a>(ast: &Ast<'a>) -> Option<Ast<'a>> {
    fn is_num(arg: &Ast) -> bool {
        if let AstKind::Number(_) = arg.kind {
            true
        } else {
            false
        }
    }
    fn get_num(arg: &Ast) -> i64 {
        if let AstKind::Number(x) = arg.kind {
            x
        } else {
            panic!("should have been a number");
        }
    }
    if ast.args.iter().all(is_num) {
        do_builtin_op(ast.span, &ast.kind, ast.args.iter().map(get_num))
    } else {
        None
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
    // TODO: There's two collects in here. Would be nice to have only one.
    let result = ast.args.iter().map(|arg| rewrite(arg)).collect::<Vec<_>>();
    if result.iter().any(|v| v.is_some()) {
        Some(Ast {
            span: ast.span,
            kind: ast.kind.clone(),
            args: result
                .into_iter()
                .zip(&ast.args)
                .map(|(new, old)| new.unwrap_or_else(|| old.clone()))
                .collect(),
        })
    } else {
        None
    }
}

pub fn rewrite<'a>(ast: &Ast<'a>) -> Option<Ast<'a>> {
    let mut ast = Cow::Borrowed(ast);
    // TODO: Figure out if postfix or prefix traversal is faster
    loop {
        if let Some(new) = recurse(&ast) {
            ast = Cow::Owned(new);
        }
        let mut did_any_rewrite = false;
        while let Some(new) = rewrite_one(&ast) {
            println!("rewrite {} to {}", ast, new);
            ast = Cow::Owned(new);
            did_any_rewrite = true;
        }
        // recurse again if we rewrote the current expr
        if !did_any_rewrite {
            break;
        }
    }
    match ast {
        Cow::Owned(ast) => Some(ast),
        Cow::Borrowed(_) => None,
    }
}
