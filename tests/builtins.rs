mod common;

use common::create_test_env;
use rusp::{eval::eval, expr::Expr, parser::Parser, scanner::Scanner};

fn parse(text: &str) -> Expr {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new(text.chars());
    while let Some(token) = scanner.get_token().expect("Failed to get token!") {
        tokens.push(token);
    }

    let mut parser = Parser::new();
    parser.add_tokens(tokens);

    let expr = parser.parse().expect("Failed to parse an expression!");
    if parser.is_parsing() {
        panic!("Too many tokens!");
    } else {
        expr
    }
}

#[test]
fn test_cond() {
    let env = create_test_env();

    let expr = parse("(cond (f 0) (t 1))");

    assert_eq!(eval(&expr, &env), Ok(Expr::Num(1_f64)));
}
