use crate::eval::EvalError;
use crate::expr::Expr;
use crate::list::List;

pub fn get_exact_one_arg<'a>(proc_name: &str, args: &'a List) -> Result<&'a Expr, EvalError> {
    let mut iter = args.iter();
    let Some(arg) = iter.next() else {
        return Err(format!("{proc_name} needs an argument."));
    };
    if iter.next().is_none() {
        Ok(arg)
    } else {
        Err(format!("{proc_name} expects only 1 argument."))
    }
}

pub fn get_exact_two_args<'a>(
    proc_name: &str,
    args: &'a List,
) -> Result<(&'a Expr, &'a Expr), EvalError> {
    let mut iter = args.iter();
    let Some(arg0) = iter.next() else {
        return Err(format!("{}: requres two arguments", proc_name));
    };
    let Some(arg1) = iter.next() else {
        return Err(format!("{}: requres two arguments", proc_name));
    };
    if iter.next().is_none() {
        Ok((arg0, arg1))
    } else {
        Err(format!("{}: takes only two arguments", proc_name))
    }
}

pub fn make_formal_args(list: &List) -> Result<Vec<String>, EvalError> {
    let mut formal_args = Vec::new();
    for item in list.iter() {
        let Expr::Sym(formal_arg) = item else {
            return Err(format!("{item} is not a symbol."));
        };
        formal_args.push(formal_arg.clone());
    }

    Ok(formal_args)
}
