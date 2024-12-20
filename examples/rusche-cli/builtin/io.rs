use rusche::{eval, EvalContext, EvalError, EvalResult, Expr, List, NIL};
use std::io::Write;

pub fn load_io_procs(context: &EvalContext) {
    context.env.define_native_proc("print", print);
    context.env.define_native_proc("println", println);
    context.env.define_native_proc("read", read);
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

fn read(_: &str, _: &List, _: &EvalContext) -> EvalResult {
    let mut input = String::new();
    if let Err(error) = std::io::stdin().read_line(&mut input) {
        return Err(EvalError::from(format!("Error reading input: {}", error)));
    }
    Ok(input.trim().to_string().into())
}
