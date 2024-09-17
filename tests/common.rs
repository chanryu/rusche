use std::rc::Rc;

use rusp::env::Env;
use rusp::eval::{eval, EvalContext};
use rusp::parser::Parser;
use rusp::scanner::Scanner;

pub fn eval_str(text: &str) -> String {
    let context = EvalContext::new();
    eval_str_env(text, context.root_env())
}

pub fn eval_str_env(text: &str, env: &Rc<Env>) -> String {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new(text.chars());
    while let Some(token) = scanner
        .get_token()
        .expect(&format!("Failed to get token: {}", text))
    {
        tokens.push(token);
    }

    let mut parser = Parser::with_tokens(tokens);
    let expr = parser
        .parse()
        .expect(&format!("Failed to parse an expression: {}", text));
    if parser.is_parsing() {
        panic!("Too many tokens: {}", text);
    }

    match eval(&expr, env) {
        Ok(expr) => expr.to_string(),
        Err(error) => format!("Err: {}", error),
    }
}
