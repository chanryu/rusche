use rusp::eval::{eval, EvalContext, Evaluator};
use rusp::lexer::Lexer;
use rusp::parser::Parser;

pub fn eval_str(text: &str) -> String {
    let evaluator = Evaluator::with_builtin();
    eval_str_env(text, &evaluator.context())
}

pub fn eval_str_env(text: &str, context: &EvalContext) -> String {
    let mut tokens = Vec::new();
    let mut lexer = Lexer::new(text.chars());
    while let Some(token) = lexer
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

    match eval(&expr, context) {
        Ok(expr) => expr.to_string(),
        Err(error) => format!("Err: {}", error),
    }
}
