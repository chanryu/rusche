use crate::{
    eval::{eval, EvalContext, EvalError, EvalResult},
    expr::Expr,
    list::List,
    utils::{eval_into_int, eval_into_str, get_2_or_3_args, get_exact_1_arg, get_exact_2_args},
};

pub fn is_str(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    if let Expr::Str(_, _) = eval(get_exact_1_arg(proc_name, args)?, context)? {
        Ok(Expr::from(true))
    } else {
        Ok(Expr::from(false))
    }
}

pub fn append(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let mut args = args.iter();
    let mut result = String::from("");
    while let Some(expr) = args.next() {
        match eval(expr, context)? {
            Expr::Str(text, _) => result += &text,
            _ => {
                return Err(EvalError::from(format!(
                    "{}: `{}` does not evaluate to a string.",
                    proc_name, expr
                )))
            }
        }
    }
    Ok(Expr::Str(result, None))
}

pub fn compare(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (arg1, arg2) = get_exact_2_args(proc_name, args)?;

    let result = match (eval(arg1, context)?, eval(arg2, context)?) {
        (Expr::Str(lhs, _), Expr::Str(rhs, _)) => lhs.cmp(&rhs),
        _ => {
            return Err(EvalError::from(format!(
                "{}: both arguments must evaluate to strings.",
                proc_name
            )))
        }
    };

    Ok(Expr::from(result as i32))
}

pub fn length(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;
    if let Expr::Str(text, _) = eval(expr, context)? {
        Ok(Expr::from(text.chars().count() as i32))
    } else {
        Err(EvalError::from(format!(
            "{}: `{}` does not evaluate to a string.",
            proc_name, expr
        )))
    }
}

pub fn slice(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (arg1, arg2, opt_arg3) = get_2_or_3_args(proc_name, args)?;

    let text = eval_into_str(proc_name, arg1, context)?;
    let text_len = text.chars().count() as i32;

    let beg = eval_into_int(proc_name, "start index", arg2, context)?;
    let end = if let Some(arg3) = opt_arg3 {
        eval_into_int(proc_name, "end index", arg3, context)?
    } else {
        text_len as i32
    };

    let to_index = |pos: i32| -> usize {
        let pos = pos.clamp(-text_len, text_len);
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
        let context = evaluator.context();

        // (str? "abc") => 1
        assert_eq!(is_str("", &list!("abc"), context), Ok(Expr::from(true)));

        // (str? 1) => '()
        assert_eq!(is_str("", &list!(1), context), Ok(Expr::from(false)));

        // (str? "abc" "def") => error
        assert!(is_str("", &list!("abc", "def"), context).is_err());
    }

    #[test]
    fn test_compare() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (str-compare "abc" "def") => 1
        assert_eq!(
            compare("", &list!("abc", "def"), context),
            Ok(Expr::from(-1))
        );

        // (str-compare "abc" "abc") => 1
        assert_eq!(
            compare("", &list!("def", "def"), context),
            Ok(Expr::from(0))
        );

        // (str-compare "def" "abc") => 1
        assert_eq!(
            compare("", &list!("def", "abc"), context),
            Ok(Expr::from(1))
        );

        // (str? "abc") => error
        assert!(compare("", &list!("abc"), context).is_err());

        // (str? "abc" "abc" "abc") => error
        assert!(compare("", &list!("abc", "abc", "abc"), context).is_err());
    }

    #[test]
    fn test_concat() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (str-append "abc" "def") => "abcdef"
        assert_eq!(
            append("", &list!("abc", "def"), context),
            Ok(Expr::from("abcdef"))
        );

        // (str-append "abc" "-" "def" "-" "123") => "abc-def-123"
        assert_eq!(
            append("", &list!("abc", "-", "def", "-", "123"), context),
            Ok(Expr::from("abc-def-123"))
        );

        // edge case: (str-conca) => ""
        assert_eq!(append("", &list!(), context), Ok(Expr::from("")));

        // edge case: (str-append "abc") => "abc"
        assert_eq!(append("", &list!("abc"), context), Ok(Expr::from("abc")));
    }

    #[test]
    fn test_slice() {
        let evaluator = Evaluator::new();
        let context = evaluator.context();

        // (str-slice "abcdef" 0 1) => "a"
        assert_eq!(
            slice("", &list!("abcdef", 0, 1), context),
            Ok(Expr::from("a"))
        );

        // (str-slice "abcdef" 0 2) => "ab"
        assert_eq!(
            slice("", &list!("abcdef", 0, 2), context),
            Ok(Expr::from("ab"))
        );

        // (str-slice "abcdef" 1 3) => "bc"
        assert_eq!(
            slice("", &list!("abcdef", 1, 3), context),
            Ok(Expr::from("bc"))
        );

        // (str-slice "abcdef" 1) => "abcdef"
        assert_eq!(
            slice("", &list!("abcdef", 1), context),
            Ok(Expr::from("bcdef"))
        );

        // (str-slice "abcdef" -2) => ""
        assert_eq!(
            slice("", &list!("abcdef", -2), context),
            Ok(Expr::from("ef"))
        );

        // (str-slice "abcdef" -2 -4) => ""
        assert_eq!(
            slice("", &list!("abcdef", -2, -4), context),
            Ok(Expr::from("cd"))
        );

        // error: (str-slice "abcdef" 0.5 1)
        assert!(slice("", &list!("abcdef", 0.5, 1), context).is_err());
    }
}
