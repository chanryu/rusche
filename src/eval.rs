use crate::expr::Expr;

pub type EvalResult = Result<Expr, String>;

pub fn eval(expr: &Expr) -> EvalResult {
    match expr {
        Expr::Nil => Ok(Expr::Nil),
        Expr::Num(value) => Ok(Expr::Num(value.clone())),
        Expr::Str(text) => Ok(Expr::Str(text.clone())),
        Expr::Sym(text) => {
            if text == "add" {
                Ok(Expr::Proc(add))
            } else {
                Err(format!("Undefined symbol: {:?}", text))
            }
        }
        Expr::Proc(func) => Ok(Expr::Proc(func.clone())),
        Expr::List(cons) => {
            if let Expr::Proc(func) = eval(&cons.car)? {
                func(&cons.cdr)
            } else {
                Err(String::from("A Proc is expected."))
            }
        }
    }
}

fn add(args: &Expr) -> EvalResult {
    let mut sum = 0_f64;
    let mut args = args;
    loop {
        match args {
            Expr::Nil => break,
            Expr::List(ref cons) => {
                if let Expr::Num(value) = eval(&cons.car)? {
                    sum += value;
                    args = &cons.cdr;
                } else {
                    return Err(String::from("Not a number!"));
                }
            }
            _ => return Err(String::from("Combination must be a proper list")),
        }
    }
    Ok(Expr::Num(sum))
}
