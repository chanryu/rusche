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
                return Err(EvalError {
                    message: format!("{proc_name}: `{expr}` does not evaluate to a string."),
                    span: expr.span(),
                })
            }
        }
    }
    Ok(Expr::Str(result, None))
}

pub fn compare(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (arg1, arg2) = get_exact_2_args(proc_name, args)?;

    let str1 = eval_into_str(proc_name, arg1, context)?;
    let str2 = eval_into_str(proc_name, arg2, context)?;

    Ok(Expr::from(str1.cmp(&str2) as i32))
}

pub fn length(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let expr = get_exact_1_arg(proc_name, args)?;
    if let Expr::Str(text, _) = eval(expr, context)? {
        Ok(Expr::from(text.chars().count() as i32))
    } else {
        Err(EvalError {
            message: format!("{proc_name}: `{expr}` does not evaluate to a string."),
            span: expr.span(),
        })
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
    use crate::macros::*;

    #[test]
    fn test_is_str() {
        setup_native_proc_test!(is_str);

        // (str? "abc") => 1
        assert_eq!(is_str(list!("abc")), Ok(Expr::from(true)));

        // (str? 1) => '()
        assert_eq!(is_str(list!(1)), Ok(Expr::from(false)));

        // (str? "abc" "def") => error
        assert!(is_str(list!("abc", "def")).is_err());
    }

    #[test]
    fn test_append() {
        setup_native_proc_test!(append);

        // (str-append "abc" "def") => "abcdef"
        assert_eq!(append(list!("abc", "def")), Ok(Expr::from("abcdef")));

        // (str-append "abc" "-" "def" "-" "123") => "abc-def-123"
        assert_eq!(
            append(list!("abc", "-", "def", "-", "123")),
            Ok(Expr::from("abc-def-123"))
        );

        // edge case: (str-conca) => ""
        assert_eq!(append(list!()), Ok(Expr::from("")));

        // edge case: (str-append "abc") => "abc"
        assert_eq!(append(list!("abc")), Ok(Expr::from("abc")));

        // (str-append 1) => error
        assert!(append(list!(1)).is_err());
    }

    #[test]
    fn test_compare() {
        setup_native_proc_test!(compare);

        // (str-compare "abc" "def") => 1
        assert_eq!(compare(list!("abc", "def")), Ok(Expr::from(-1)));

        // (str-compare "abc" "abc") => 1
        assert_eq!(compare(list!("def", "def")), Ok(Expr::from(0)));

        // (str-compare "def" "abc") => 1
        assert_eq!(compare(list!("def", "abc")), Ok(Expr::from(1)));

        // (str? "abc") => error
        assert!(compare(list!("abc")).is_err());

        // (str? "abc" "abc" "abc") => error
        assert!(compare(list!("abc", "abc", "abc")).is_err());
    }

    #[test]
    fn test_length() {
        setup_native_proc_test!(length);

        // (str-length "") => 0
        assert_eq!(length(list!("")), Ok(Expr::from(0)));

        // (str-length "abcdef") => 6
        assert_eq!(length(list!("abcdef")), Ok(Expr::from(6)));

        // (str-length) => error
        assert!(length(list!()).is_err());

        // (str-length 1) => error
        assert!(length(list!(1)).is_err());

        // (str-length "abc" "xyz") => error
        assert!(length(list!("abc", "xyz")).is_err());
    }

    #[test]
    fn test_slice() {
        setup_native_proc_test!(slice);

        // (str-slice "abcdef" 0 1) => "a"
        assert_eq!(slice(list!("abcdef", 0, 1)), Ok(Expr::from("a")));

        // (str-slice "abcdef" 0 2) => "ab"
        assert_eq!(slice(list!("abcdef", 0, 2)), Ok(Expr::from("ab")));

        // (str-slice "abcdef" 1 3) => "bc"
        assert_eq!(slice(list!("abcdef", 1, 3)), Ok(Expr::from("bc")));

        // (str-slice "abcdef" 1) => "abcdef"
        assert_eq!(slice(list!("abcdef", 1)), Ok(Expr::from("bcdef")));

        // (str-slice "abcdef" -2) => ""
        assert_eq!(slice(list!("abcdef", -2)), Ok(Expr::from("ef")));

        // (str-slice "abcdef" -2 -4) => ""
        assert_eq!(slice(list!("abcdef", -2, -4)), Ok(Expr::from("cd")));

        // error: (str-slice "abcdef" 0.5 1)
        assert!(slice(list!("abcdef", 0.5, 1)).is_err());
    }
}
