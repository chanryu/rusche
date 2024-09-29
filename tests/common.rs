use rusche::eval::{eval, EvalContext, Evaluator};
use rusche::lexer::tokenize;
use rusche::parser::Parser;

pub fn eval_str(text: &str) -> String {
    let evaluator = Evaluator::with_builtin();
    eval_str_env(text, &evaluator.context())
}

pub fn eval_str_env(text: &str, context: &EvalContext) -> String {
    let tokens = tokenize(text).expect(&format!("Failed to tokenize: {}", text));
    let mut parser = Parser::with_tokens(tokens);
    let Some(expr) = parser
        .parse()
        .expect(&format!("Failed to parse an expression: {}", text))
    else {
        panic!("No expression parsed from: {}", text);
    };

    match eval(&expr, context) {
        Ok(result) => result.to_string(),
        Err(error) => format!("Err: {}", error),
    }
}
