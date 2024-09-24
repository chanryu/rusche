use crate::eval::{eval, EvalContext, EvalResult};
use crate::expr::Expr;
use crate::list::List;

use super::utils::{eval_to_num, eval_to_str, get_2_or_3_args, get_exact_1_arg, get_exact_2_args};

pub fn is_str(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    if let Expr::Str(_, _) = eval(get_exact_1_arg(proc_name, args)?, context)? {
        Ok(Expr::from(true))
    } else {
        Ok(Expr::from(false))
    }
}

pub fn compare(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (arg1, arg2) = get_exact_2_args(proc_name, args)?;

    let result = match (eval(arg1, context)?, eval(arg2, context)?) {
        (Expr::Str(lhs, _), Expr::Str(rhs, _)) => lhs.cmp(&rhs),
        _ => {
            return Err(format!(
                "{}: both arguments must evaluate to strings.",
                proc_name
            ))
        }
    };

    Ok(Expr::from(result as i32))
}

pub fn concat(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut args = args.iter();
    let mut result = String::from("");
    while let Some(expr) = args.next() {
        match eval(expr, context)? {
            Expr::Str(text, _) => result += &text,
            _ => {
                return Err(format!(
                    "{}: `{}` does not evaluate to a string.",
                    proc_name, expr
                ))
            }
        }
    }
    Ok(Expr::Str(result, None))
}

pub fn length(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;
    if let Expr::Str(text, _) = eval(expr, context)? {
        Ok(Expr::from(text.chars().count() as i32))
    } else {
        Err(format!(
            "{}: `{}` does not evaluate to a string.",
            proc_name, expr
        ))
    }
}

pub fn slice(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (arg1, arg2, opt_arg3) = get_2_or_3_args(proc_name, args)?;

    let text = eval_to_str(proc_name, arg1, context)?;
    let text_len = text.chars().count() as i32;

    let beg = eval_to_num(proc_name, arg2, context)?;
    let end = if let Some(arg3) = opt_arg3 {
        eval_to_num(proc_name, arg3, context)?
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

    Ok(Expr::Str(
        text.chars().skip(beg).take(end - beg).collect(),
        None,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{eval::Evaluator, list::list};

    #[test]
    fn test_is_str() {
        let evaluator = Evaluator::new();
        let context = evaluator.root_context();

        // (str? "abc") => 1
        assert_eq!(is_str("", &list!("abc"), &context), Ok(Expr::from(true)));

        // (str? 1) => '()
        assert_eq!(is_str("", &list!(1), &context), Ok(Expr::from(false)));

        // (str? "abc" "def") => error
        assert!(is_str("", &list!("abc", "def"), &context).is_err());
    }

    #[test]
    fn test_compare() {
        let evaluator = Evaluator::new();
        let context = evaluator.root_context();

        // (str-compare "abc" "def") => 1
        assert_eq!(
            compare("", &list!("abc", "def"), &context),
            Ok(Expr::from(-1))
        );

        // (str-compare "abc" "abc") => 1
        assert_eq!(
            compare("", &list!("def", "def"), &context),
            Ok(Expr::from(0))
        );

        // (str-compare "def" "abc") => 1
        assert_eq!(
            compare("", &list!("def", "abc"), &context),
            Ok(Expr::from(1))
        );

        // (str? "abc") => error
        assert!(compare("", &list!("abc"), &context).is_err());

        // (str? "abc" "abc" "abc") => error
        assert!(compare("", &list!("abc", "abc", "abc"), &context).is_err());
    }

    #[test]
    fn test_concat() {
        let evaluator = Evaluator::new();
        let context = evaluator.root_context();

        // (str-concat "abc" "def") => "abcdef"
        assert_eq!(
            concat("", &list!("abc", "def"), &context),
            Ok(Expr::from("abcdef"))
        );

        // (str-concat "abc" "-" "def" "-" "123") => "abc-def-123"
        assert_eq!(
            concat("", &list!("abc", "-", "def", "-", "123"), &context),
            Ok(Expr::from("abc-def-123"))
        );

        // edge case: (str-conca) => ""
        assert_eq!(concat("", &list!(), &context), Ok(Expr::from("")));

        // edge case: (str-concat "abc") => "abc"
        assert_eq!(concat("", &list!("abc"), &context), Ok(Expr::from("abc")));
    }

    #[test]
    fn test_slice() {
        let evaluator = Evaluator::new();
        let context = evaluator.root_context();

        // (str-slice "abcdef" 0 1) => "a"
        assert_eq!(
            slice("", &list!("abcdef", 0, 1), &context),
            Ok(Expr::from("a"))
        );

        // (str-slice "abcdef" 0 2) => "ab"
        assert_eq!(
            slice("", &list!("abcdef", 0, 2), &context),
            Ok(Expr::from("ab"))
        );

        // (str-slice "abcdef" 1 3) => "bc"
        assert_eq!(
            slice("", &list!("abcdef", 1, 3), &context),
            Ok(Expr::from("bc"))
        );

        // (str-slice "abcdef" 1) => "abcdef"
        assert_eq!(
            slice("", &list!("abcdef", 1), &context),
            Ok(Expr::from("bcdef"))
        );

        // (str-slice "abcdef" -2) => ""
        assert_eq!(
            slice("", &list!("abcdef", -2), &context),
            Ok(Expr::from("ef"))
        );

        // (str-slice "abcdef" -2 -4) => ""
        assert_eq!(
            slice("", &list!("abcdef", -2, -4), &context),
            Ok(Expr::from("cd"))
        );

        // error: (str-slice "abcdef" 0.5 1)
        assert!(slice("", &list!("abcdef", 0.5, 1), &context).is_err());
    }
}
