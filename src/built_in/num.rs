use crate::eval::{eval, Env, EvalResult};
use crate::expr::Expr;
use crate::list::List;

fn binop(
    args: &List,
    env: &Env,
    identity: f64,
    is_associative: bool,
    func: fn(lhs: f64, rhs: f64) -> f64,
) -> EvalResult {
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

pub fn add(args: &List, env: &Env) -> EvalResult {
    binop(args, env, 0_f64, true, |lhs, rhs| lhs + rhs)
}

pub fn minus(args: &List, env: &Env) -> EvalResult {
    binop(args, env, 0_f64, false, |lhs, rhs| lhs - rhs)
}

pub fn multiply(args: &List, env: &Env) -> EvalResult {
    binop(args, env, 1_f64, true, |lhs, rhs| lhs * rhs)
}

pub fn divide(args: &List, env: &Env) -> EvalResult {
    binop(args, env, 1_f64, false, |lhs, rhs| lhs / rhs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::num;
    use crate::list::cons;
    use crate::macros::list;

    #[test]
    fn test_add() {
        let env = Env::new();

        // (+ 1) => 1
        let args = list!(num(1));
        assert_eq!(add(&args, &env), Ok(num(1)));

        // (+ 2 1) => 3
        let args = list!(num(2), num(1));
        assert_eq!(add(&args, &env), Ok(num(3)));
    }

    #[test]
    fn test_minus() {
        let env = Env::new();

        // (- 1) => -1
        let args = list!(num(1));
        assert_eq!(minus(&args, &env), Ok(num(-1)));

        // (- 2 1) => 1
        let args = list!(num(2), num(1));
        assert_eq!(minus(&args, &env), Ok(num(1)));
    }

    #[test]
    fn test_multiply() {
        let env = Env::new();

        // (* 1) => 1
        let args = list!(num(1));
        assert_eq!(multiply(&args, &env), Ok(num(1)));

        // (* 2 1) => 2
        let args = list!(num(2), num(1));
        assert_eq!(multiply(&args, &env), Ok(num(2)));

        // (* 3 2 1) => 6
        let args = list!(num(3), num(2), num(1));
        assert_eq!(multiply(&args, &env), Ok(num(6)));
    }

    #[test]
    fn test_divide() {
        let env = Env::new();

        // (/ 2) => 0.5
        let args = list!(num(2));
        assert_eq!(divide(&args, &env), Ok(num(0.5)));

        // (/ 4 2) => 2
        let args = list!(num(4), num(2));
        assert_eq!(divide(&args, &env), Ok(num(2)));
    }
}
