use rusp::env::Env;
use rusp::eval::eval;
use rusp::expr::Expr;
use rusp::parser::Parser;
use rusp::scanner::Scanner;

pub fn parse_single_expr(text: &str) -> Expr {
    let mut tokens = Vec::new();
    let mut scanner = Scanner::new(text.chars());
    while let Some(token) = scanner
        .get_token()
        .expect(&format!("Failed to get token: {}", text))
    {
        tokens.push(token);
    }

    let mut parser = Parser::new();
    parser.add_tokens(tokens);

    let expr = parser
        .parse()
        .expect(&format!("Failed to parse an expression: {}", text));
    if parser.is_parsing() {
        panic!("Too many tokens: {}", text);
    } else {
        expr
    }
}

pub fn eval_str(text: &str) -> String {
    let env = Env::new_root_env();
    match eval(&parse_single_expr(text), &env) {
        Ok(expr) => expr.to_string(),
        Err(error) => format!("Err: {}", error),
    }
}
