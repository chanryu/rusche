use rusche::{
    eval::{eval_source, EvalContext, Evaluator},
};

pub trait EvalToStr {
    fn eval_to_str(&self, src: &str) -> String;
}

impl EvalToStr for EvalContext {
    fn eval_to_str(&self, src: &str) -> String {
        match eval_source(src, self, false) {
            Ok(result) => result.to_string(),
            Err(error) => format!("Err: {error}"),
        }
    }
}

impl EvalToStr for Evaluator {
    fn eval_to_str(&self, src: &str) -> String {
        match self.eval_str(src) {
            Ok(result) => result.to_string(),
            Err(error) => format!("Err: {error}"),
        }
    }
}
