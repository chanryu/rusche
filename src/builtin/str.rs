use crate::{
    eval::{eval, ErrorKind, EvalContext, EvalError, EvalResult},
    expr::Expr,
    list::List,
    utils::{
        eval_into_int, eval_into_str, get_2_or_3_args, get_exact_1_arg, get_exact_2_args,
        get_exact_3_args,
    },
};

pub fn is_str(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    if let Expr::Str(_, _) = eval(get_exact_1_arg(proc_name, args)?, context)? {
        Ok(Expr::from(true))
    } else {
        Ok(Expr::from(false))
    }
}

pub fn append(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let args = args.iter();
    let mut result = String::from("");
    for expr in args {
        match eval(expr, context)? {
            Expr::Str(text, _) => result += &text,
            value => {
                return Err(EvalError::new(
                    ErrorKind::Type,
                    format!("{proc_name}: `{expr}` evaluated to `{value}`, expected a string"),
                )
                .with_span(expr.span()));
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
    match eval(expr, context)? {
        Expr::Str(text, _) => Ok(Expr::from(text.chars().count() as i32)),
        value => Err(EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: `{expr}` evaluated to `{value}`, expected a string"),
        )
        .with_span(expr.span())),
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
        text_len
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

/// Character index of the first occurrence of `needle` in `haystack`, or `false`.
pub fn find(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (haystack_expr, needle_expr) = get_exact_2_args(proc_name, args)?;
    let haystack = eval_into_str(proc_name, haystack_expr, context)?;
    let needle = eval_into_str(proc_name, needle_expr, context)?;

    if needle.is_empty() {
        return Ok(Expr::from(0));
    }

    match haystack.find(&needle) {
        Some(byte_idx) => Ok(Expr::from(haystack[..byte_idx].chars().count() as i32)),
        None => Ok(Expr::from(false)),
    }
}

/// Split `text` on `delim`. An empty delimiter splits into one-character strings.
pub fn split(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (text_expr, delim_expr) = get_exact_2_args(proc_name, args)?;
    let text = eval_into_str(proc_name, text_expr, context)?;
    let delim = eval_into_str(proc_name, delim_expr, context)?;

    let parts: Vec<Expr> = if delim.is_empty() {
        text.chars().map(|c| Expr::from(c.to_string())).collect()
    } else {
        text.split(&delim).map(Expr::from).collect()
    };

    Ok(Expr::List(List::from(parts), None))
}

pub fn trim(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let text = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::from(text.trim()))
}

pub fn trim_left(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let text = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::from(text.trim_start()))
}

pub fn trim_right(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let text = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::from(text.trim_end()))
}

pub fn upcase(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let text = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::from(text.to_uppercase()))
}

pub fn downcase(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let text = eval_into_str(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    Ok(Expr::from(text.to_lowercase()))
}

/// Replace every non-overlapping occurrence of `from` with `to`. Empty `from` is an error.
pub fn replace(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (text_expr, from_expr, to_expr) = get_exact_3_args(proc_name, args)?;
    let text = eval_into_str(proc_name, text_expr, context)?;
    let from = eval_into_str(proc_name, from_expr, context)?;
    let to = eval_into_str(proc_name, to_expr, context)?;

    if from.is_empty() {
        return Err(EvalError::new(
            ErrorKind::Other,
            format!("{proc_name}: search string must not be empty"),
        )
        .with_span(from_expr.span()));
    }

    Ok(Expr::from(text.replace(&from, &to)))
}

/// Join a list of strings with a separator: `(str-join lst sep)`.
pub fn join(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (lst_expr, sep_expr) = get_exact_2_args(proc_name, args)?;
    let sep = eval_into_str(proc_name, sep_expr, context)?;

    let Expr::List(list, _) = eval(lst_expr, context)? else {
        return Err(EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: `{lst_expr}` evaluated to a non-list"),
        )
        .with_span(lst_expr.span()));
    };

    let mut result = String::new();
    for (i, item) in list.iter().enumerate() {
        match item {
            Expr::Str(text, _) => {
                if i > 0 {
                    result.push_str(&sep);
                }
                result.push_str(text);
            }
            value => {
                return Err(EvalError::new(
                    ErrorKind::Type,
                    format!("{proc_name}: list element `{value}` is not a string"),
                )
                .with_span(item.span()));
            }
        }
    }

    Ok(Expr::from(result))
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

    #[test]
    fn test_find_split_trim_case_replace_join() {
        let evaluator = crate::eval::Evaluator::new();
        let context = evaluator.context();
        let find = |args| find("str-find", &args, context);
        let split = |args| split("str-split", &args, context);
        let trim = |args| trim("str-trim", &args, context);
        let trim_left = |args| trim_left("str-trim-left", &args, context);
        let trim_right = |args| trim_right("str-trim-right", &args, context);
        let upcase = |args| upcase("str-upcase", &args, context);
        let downcase = |args| downcase("str-downcase", &args, context);
        let replace = |args| replace("str-replace", &args, context);
        let join = |args| join("str-join", &args, context);

        assert_eq!(find(list!("hello", "ll")), Ok(Expr::from(2)));
        assert_eq!(find(list!("hello", "x")), Ok(Expr::from(false)));
        assert_eq!(find(list!("hello", "")), Ok(Expr::from(0)));

        assert_eq!(
            split(list!("a,b,c", ",")),
            Ok(Expr::List(
                List::from(vec![
                    Expr::from("a"),
                    Expr::from("b"),
                    Expr::from("c")
                ]),
                None
            ))
        );
        assert_eq!(
            split(list!("ab", "")),
            Ok(Expr::List(
                List::from(vec![Expr::from("a"), Expr::from("b")]),
                None
            ))
        );

        assert_eq!(trim(list!("  hi  ")), Ok(Expr::from("hi")));
        assert_eq!(trim_left(list!("  hi  ")), Ok(Expr::from("hi  ")));
        assert_eq!(trim_right(list!("  hi  ")), Ok(Expr::from("  hi")));

        assert_eq!(upcase(list!("Hi")), Ok(Expr::from("HI")));
        assert_eq!(downcase(list!("Hi")), Ok(Expr::from("hi")));

        assert_eq!(
            replace(list!("a-b-c", "-", "_")),
            Ok(Expr::from("a_b_c"))
        );
        assert!(replace(list!("abc", "", "x")).is_err());

        use crate::expr::intern;
        assert_eq!(
            join(list!(list!(intern("quote"), list!("a", "b", "c")), "-")),
            Ok(Expr::from("a-b-c"))
        );
        assert_eq!(
            join(list!(list!(intern("quote"), list!()), "-")),
            Ok(Expr::from(""))
        );
    }
}
