use rusche::{
    eval::{eval, EvalContext, Evaluator},
    lexer::tokenize,
    parser::Parser,
};

pub trait EvalToStr {
    fn eval_to_str(&self, src: &str) -> String;
}

impl EvalToStr for EvalContext {
    fn eval_to_str(&self, src: &str) -> String {
        let tokens = tokenize(src).expect(&format!("Failed to tokenize: {}", src));
        let mut parser = Parser::with_tokens(tokens);
        let Some(expr) = parser
            .parse()
            .expect(&format!("Failed to parse an expression: {}", src))
        else {
            panic!("No expression parsed from: {}", src);
        };

        match eval(&expr, self) {
            Ok(result) => result.to_string(),
            Err(error) => format!("Err: {:?}", error),
        }
    }
}

impl EvalToStr for Evaluator {
    fn eval_to_str(&self, src: &str) -> String {
        self.context().eval_to_str(src)
    }
}
