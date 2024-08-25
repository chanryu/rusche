use crate::eval::{eval, Env, EvalResult};
use crate::expr::Expr;

fn binop(
    args: &Expr,
    env: &Env,
    identity: f64,
    is_associative: bool,
    func: fn(lhs: f64, rhs: f64) -> f64,
) -> EvalResult {
    let args = args.collect();
    let mut result = identity;

    for (index, arg) in args.iter().enumerate() {
        match eval(&arg, env)? {
            Expr::Num(value) => {
                if index == 0 && args.len() > 1 && !is_associative {
                    result = value;
                } else {
                    result = func(result, value);
                }
            }
            _ => return Err(format!("{} is not a number!", arg)),
        }
    }

    Ok(Expr::Num(result))
}

pub fn add(args: &Expr, env: &Env) -> EvalResult {
    binop(args, env, 0_f64, true, |lhs, rhs| lhs + rhs)
}

pub fn minus(args: &Expr, env: &Env) -> EvalResult {
    binop(args, env, 0_f64, false, |lhs, rhs| lhs - rhs)
}

pub fn multiply(args: &Expr, env: &Env) -> EvalResult {
    binop(args, env, 1_f64, true, |lhs, rhs| lhs * rhs)
}

pub fn divide(args: &Expr, env: &Env) -> EvalResult {
    binop(args, env, 1_f64, false, |lhs, rhs| lhs / rhs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::test_utils::*;
    use crate::expr::NIL;

    #[test]
    fn test_add() {
        let env = Env::new();

        // (+ 1) => 1
        let args = cons(num(1), NIL);
        assert_eq!(add(&args, &env), Ok(num(1)));

        // (+ 2 1) => 3
        let args = cons(num(2), cons(num(1), NIL));
        assert_eq!(add(&args, &env), Ok(num(3)));
    }

    #[test]
    fn test_minus() {
        let env = Env::new();

        // (- 1) => -1
        let args = cons(num(1), NIL);
        assert_eq!(minus(&args, &env), Ok(num(-1)));

        // (- 2 1) => 1
        let args = cons(num(2), cons(num(1), NIL));
        assert_eq!(minus(&args, &env), Ok(num(1)));
    }

    #[test]
    fn test_multiply() {
        let env = Env::new();

        // (* 1) => 1
        let args = cons(num(1), NIL);
        assert_eq!(multiply(&args, &env), Ok(num(1)));

        // (* 2 1) => 2
        let args = cons(num(2), cons(num(1), NIL));
        assert_eq!(multiply(&args, &env), Ok(num(2)));

        // (* 3 2 1) => 6
        let args = cons(num(3), cons(num(2), cons(num(1), NIL)));
        assert_eq!(multiply(&args, &env), Ok(num(6)));
    }

    #[test]
    fn test_divide() {
        let env = Env::new();

        // (/ 2) => 0.5
        let args = cons(num(2), NIL);
        assert_eq!(divide(&args, &env), Ok(num(0.5)));

        // (/ 4 2) => 2
        let args = cons(num(4), cons(num(2), NIL));
        assert_eq!(divide(&args, &env), Ok(num(2)));
    }
}
