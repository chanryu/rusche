use std::rc::Rc;

use crate::env::Env;
use crate::eval::{eval, EvalResult};
use crate::expr::Expr;
use crate::list::List;

use super::utils::{eval_to_num, eval_to_str, get_2_or_3_args, get_exact_1_arg, get_exact_2_args};

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
    let (arg1, arg2, opt_arg3) = get_2_or_3_args(proc_name, args)?;

    let text = eval_to_str(arg1, env)?;
    let text_len = text.chars().count() as i32;

    let beg = eval_to_num(arg2, env)?;
    let end = if let Some(arg3) = opt_arg3 {
        eval_to_num(arg3, env)?
    } else {
        text_len as f64
    };

    if beg.fract() != 0.0 {
        return Err(format!(
            "{}: start index must be an integer, but got {}.",
            proc_name, beg
        ));
    }

    if end.fract() != 0.0 {
        return Err(format!(
            "{}: end index must be an integer, but got {}.",
            proc_name, end
        ));
    }

    let to_index = |pos: f64| -> usize {
        let pos = (pos as i32).clamp(-text_len, text_len);
        if pos < 0 {
            (text_len + pos) as usize
        } else {
            pos as usize
        }
    };

    let beg = to_index(beg);
    let end = to_index(end);
    let (beg, end) = if beg <= end { (beg, end) } else { (end, beg) };

    Ok(Expr::Str(text.chars().skip(beg).take(end - beg).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::list::list;

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

        // (str-slice "abcdef" 1 3) => "bc"
        assert_eq!(
            slice("", &list!("abcdef", 1, 3), &env),
            Ok(Expr::from("bc"))
        );

        // (str-slice "abcdef" 1) => "abcdef"
        assert_eq!(
            slice("", &list!("abcdef", 1), &env),
            Ok(Expr::from("bcdef"))
        );

        // (str-slice "abcdef" -2) => ""
        assert_eq!(slice("", &list!("abcdef", -2), &env), Ok(Expr::from("ef")));

        // (str-slice "abcdef" -2 -4) => ""
        assert_eq!(
            slice("", &list!("abcdef", -2, -4), &env),
            Ok(Expr::from("cd"))
        );

        // error: (str-slice "abcdef" 0.5 1)
        assert!(slice("", &list!("abcdef", 0.5, 1), &env).is_err());
    }
}
