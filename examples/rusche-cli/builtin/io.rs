use rusche::{
    eval::{eval, EvalContext, EvalError,  EvalResult},
    expr::{Expr, NIL},
    list::List,
};
use std::io::Write;

pub fn load_io_procs(context: &EvalContext) {
    context.env.define_native_proc("print", print);
    context.env.define_native_proc("println", println);
    context.env.define_native_proc("read", read);
    context.env.define_native_proc("read-num", read_num);
}

fn print_args(args: &List, context: &EvalContext) -> Result<(), EvalError> {
    for expr in args.iter() {
        match eval(expr, context)? {
            Expr::Str(text, _) => print!("{}", text), // w/o double quotes
            expr => print!("{}", expr),
        }
    }
    Ok(())
}

fn print(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    print_args(args, context)?;
    let _ = std::io::stdout().flush();
    Ok(NIL)
}

fn println(_: &str, args: &List, context: &EvalContext) -> EvalResult {
    print_args(args, context)?;
    println!();
    Ok(NIL)
}

fn read_line() -> Result<String, EvalError> {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_line(&mut input) {
        return Err(EvalError {
            message: format!("Error reading input: {}", error),
            span: None,
        });
    }
    Ok(input.trim().to_string())
}

fn read(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    Ok(read_line()?.into())
}

fn read_num(proc_name: &str, _: &List, _: &EvalContext) -> EvalResult {
    match read_line()?.parse::<f64>() {
        Ok(num) => Ok(Expr::from(num)),
        Err(err) => Err(EvalError {
            message: format!("{}: {}", proc_name, err),
            span: None,
        }),
    }
}
