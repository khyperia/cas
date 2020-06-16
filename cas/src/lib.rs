mod ast;
mod engine;
mod lex;
mod parse;
mod rules_interpreter;

pub use ast::*;
pub use engine::*;
pub use lex::*;
pub use parse::*;
pub use rules_interpreter::*;

pub fn simplify(expr: &str) -> String {
    let ast = parse::parse(expr);
    let simplified = rewrite(&ast).expect("Expression did not simplify");
    format!("{}", simplified)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eq(left: &str, right: &str) {
        let left_eval = simplify(left);
        assert_eq!(left_eval, right);
    }

    #[test]
    fn it_works() {
        eq("2+2", "4");
        eq("2*(9/2)", "9");
        eq("(2/3)*(9/2/6)", "(1 / 2)");
        eq("(2/3)*(9/6)", "1");
    }
}
