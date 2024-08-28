use rusp::{
    env::Env,
    eval::{eval, EvalResult},
    expr::{Expr, NIL},
    parser::Parser,
    scanner::Scanner,
};

pub fn parse_expr(text: &str) -> Expr {
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

pub fn test_eval(expr: &str) -> EvalResult {
    let env = Env::new_root_env();
    env.set("t", Expr::Sym("#t".into()));
    env.set("f", NIL);
    eval(&parse_expr(expr), &env)
}
