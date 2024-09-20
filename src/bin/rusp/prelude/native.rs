use std::{io::Write, rc::Rc};

use rusp::{
    builtin::utils::{eval_to_str, get_exact_1_arg},
    env::Env,
    eval::{eval, EvalResult},
    expr::{Expr, NIL},
    list::List,
};

pub fn print(_: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    for expr in args.iter() {
        match eval(expr, env)? {
            Expr::Str(text) => print!("{}", text), // w/o double quotes
            expr => print!("{}", expr),
        }
    }
    let _ = std::io::stdout().flush();
    Ok(NIL)
}

pub fn read(_: &str, _: &List, _: &Rc<Env>) -> EvalResult {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_line(&mut input) {
        return Err(format!("Error reading input: {}", error));
    }
    Ok(Expr::from(input.trim()))
}

pub fn parse_num(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;
    let text = eval_to_str(proc_name, expr, env)?;

    match text.parse::<f64>() {
        Ok(num) => Ok(Expr::from(num)),
        Err(_) => Err(format!("{}: '{}' is not a number", proc_name, text)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusp::{eval::EvalContext, list};

    #[test]
    fn test_parse_num() {
        let ctx = EvalContext::new();
        let env = ctx.root_env();
        let parse_num = |args| parse_num("parse-num", &args, env);

        assert_eq!(parse_num(list!("1")), Ok(Expr::from(1)));
        assert_eq!(parse_num(list!("-24.5")), Ok(Expr::from(-24.5)));
        assert_eq!(parse_num(list!("999.9")), Ok(Expr::from(999.9)));
        assert_eq!(parse_num(list!("-2e12")), Ok(Expr::from(-2e12)));

        assert!(parse_num(list!(1)).is_err());
        assert!(parse_num(list!("")).is_err());
        assert!(parse_num(list!("1", "2")).is_err());
        assert!(parse_num(list!("xyz")).is_err());
        assert!(parse_num(list!("1yz")).is_err());
    }
}
