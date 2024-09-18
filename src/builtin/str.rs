use std::rc::Rc;

use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

use super::utils::{eval_to_num, eval_to_str, get_exact_1_arg, get_exact_2_args, get_exact_3_args};

pub fn is_str(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    if let Expr::Str(_) = eval(get_exact_1_arg(proc_name, args)?, env)? {
        Ok(Expr::from(true))
    } else {
        Ok(Expr::from(false))
    }
}

pub fn compare(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (arg1, arg2) = get_exact_2_args(proc_name, args)?;

    let result = match (eval(arg1, env)?, eval(arg2, env)?) {
        (Expr::Str(lhs), Expr::Str(rhs)) => lhs.cmp(&rhs),
        _ => {
            return Err(format!(
                "{}: both arguments must evaluate to strings.",
                proc_name
            ))
        }
    };

    Ok(Expr::from(result as i32))
}

pub fn concat(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let mut args = args.iter();
    let mut result = String::from("");
    while let Some(expr) = args.next() {
        match eval(expr, env)? {
            Expr::Str(text) => result += &text,
            _ => {
                return Err(format!(
                    "{}: `{}` does not evaluate to a string.",
                    proc_name, expr
                ))
            }
        }
    }
    Ok(Expr::Str(result))
}

pub fn length(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;
    if let Expr::Str(text) = eval(expr, env)? {
        Ok(Expr::from(text.chars().count() as i32))
    } else {
        Err(format!(
            "{}: `{}` does not evaluate to a string.",
            proc_name, expr
        ))
    }
}

pub fn slice(proc_name: &str, args: &List, env: &Rc<Env>) -> EvalResult {
    let (arg1, arg2, arg3) = get_exact_3_args(proc_name, args)?;

    let text = eval_to_str(arg1, env)?;
    let beg = eval_to_num(arg2, env)?;
    let len = eval_to_num(arg3, env)?;

    if beg.fract() != 0.0 || len.fract() != 0.0 {
        return Err(format!(
            "{}: start and end must be integers, but got {} and {}.",
            proc_name, beg, len
        ));
    }

    Ok(Expr::Str(
        text.chars()
            .skip(beg as usize)
            .take(len as usize)
            .collect::<String>(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::list;

    #[test]
    fn test_is_str() {
        let env = Env::for_unit_test();

        // (str? "abc") => 1
        assert_eq!(is_str("", &list!("abc"), &env), Ok(Expr::from(true)));

        // (str? 1) => '()
        assert_eq!(is_str("", &list!(1), &env), Ok(Expr::from(false)));

        // (str? "abc" "def") => error
        assert!(is_str("", &list!("abc", "def"), &env).is_err());
    }

    #[test]
    fn test_compare() {
        let env = Env::for_unit_test();

        // (str-compare "abc" "def") => 1
        assert_eq!(compare("", &list!("abc", "def"), &env), Ok(Expr::from(-1)));

        // (str-compare "abc" "abc") => 1
        assert_eq!(compare("", &list!("def", "def"), &env), Ok(Expr::from(0)));

        // (str-compare "def" "abc") => 1
        assert_eq!(compare("", &list!("def", "abc"), &env), Ok(Expr::from(1)));

        // (str? "abc") => error
        assert!(compare("", &list!("abc"), &env).is_err());

        // (str? "abc" "abc" "abc") => error
        assert!(compare("", &list!("abc", "abc", "abc"), &env).is_err());
    }

    #[test]
    fn test_concat() {
        let env = Env::for_unit_test();

        // (str-concat "abc" "def") => "abcdef"
        assert_eq!(
            concat("", &list!("abc", "def"), &env),
            Ok(Expr::from("abcdef"))
        );

        // (str-concat "abc" "-" "def" "-" "123") => "abc-def-123"
        assert_eq!(
            concat("", &list!("abc", "-", "def", "-", "123"), &env),
            Ok(Expr::from("abc-def-123"))
        );

        // edge case: (str-conca) => ""
        assert_eq!(concat("", &list!(), &env), Ok(Expr::from("")));

        // edge case: (str-concat "abc") => "abc"
        assert_eq!(concat("", &list!("abc"), &env), Ok(Expr::from("abc")));
    }

    #[test]
    fn test_slice() {
        let env = Env::for_unit_test();

        // (str-slice "abcdef" 0 1) => "a"
        assert_eq!(slice("", &list!("abcdef", 0, 1), &env), Ok(Expr::from("a")));

        // (str-slice "abcdef" 0 2) => "ab"
        assert_eq!(
            slice("", &list!("abcdef", 0, 2), &env),
            Ok(Expr::from("ab"))
        );

        // (str-slice "abcdef" 1 2) => "bc"
        assert_eq!(
            slice("", &list!("abcdef", 1, 2), &env),
            Ok(Expr::from("bc"))
        );

        // edge case: (str-slice "abcdef" 0 999) => "abcdef"
        assert_eq!(
            slice("", &list!("abcdef", 0, 999), &env),
            Ok(Expr::from("abcdef"))
        );

        // edge case: (str-slice "abcdef" 0 -1) => ""
        assert_eq!(slice("", &list!("abcdef", 0, -1), &env), Ok(Expr::from("")));

        // edge case: (str-slice "abcdef" 999 10) => ""
        assert_eq!(
            slice("", &list!("abcdef", 999, 10), &env),
            Ok(Expr::from(""))
        );

        // error: (str-slice "abcdef" 0.5 1)
        assert!(slice("", &list!("abcdef", 0.5, 1), &env).is_err());
    }
}
