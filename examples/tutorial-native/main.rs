//! Companion example for docs/tutorials/native-functions.md
//!
//! `clamp` is a plain function. `bump` closes over one counter. `deposit` and
//! `balance` are registered on two evaluators, each capturing its own bank, so
//! one script cannot see the other's balance.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use rusche::{
    arity_error, eval_into_num, get_exact_1_arg, get_exact_3_args, tokenize, EvalContext,
    EvalResult, Evaluator, Expr, List, Parser,
};

fn clamp(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (value_expr, lo_expr, hi_expr) = get_exact_3_args(proc_name, args)?;
    let value = eval_into_num(proc_name, value_expr, context)?;
    let lo = eval_into_num(proc_name, lo_expr, context)?;
    let hi = eval_into_num(proc_name, hi_expr, context)?;

    Ok(Expr::from(value.clamp(lo, hi)))
}

struct Bank {
    balance: f64,
}

/// Registers `deposit` and `balance` against this bank. A second call with a
/// different `Bank` installs a second, independent pair of procedures.
fn install(evaluator: &Evaluator, bank: &Rc<RefCell<Bank>>) {
    let deposit_bank = Rc::clone(bank);
    evaluator
        .root_env()
        .define_native_proc("deposit", move |proc_name, args, context| {
            deposit(&deposit_bank, proc_name, args, context)
        });

    let balance_bank = Rc::clone(bank);
    evaluator
        .root_env()
        .define_native_proc("balance", move |proc_name, args, _| {
            balance(&balance_bank, proc_name, args)
        });
}

fn deposit(
    bank: &RefCell<Bank>,
    proc_name: &str,
    args: &List,
    context: &EvalContext,
) -> EvalResult {
    let amount = eval_into_num(proc_name, get_exact_1_arg(proc_name, args)?, context)?;
    let mut bank = bank.borrow_mut();
    bank.balance += amount;
    Ok(Expr::from(bank.balance))
}

fn balance(bank: &RefCell<Bank>, proc_name: &str, args: &List) -> EvalResult {
    if !args.is_nil() {
        return Err(arity_error(proc_name, 0..=0, args.len()));
    }
    Ok(Expr::from(bank.borrow().balance))
}

fn main() {
    let evaluator = Evaluator::with_prelude();
    evaluator.root_env().define_native_proc("clamp", clamp);

    let result = evaluator
        .eval(
            &Parser::with_tokens(tokenize("(clamp 15 0 10)", None).unwrap())
                .parse()
                .unwrap()
                .unwrap(),
        )
        .unwrap();

    assert_eq!(result, Expr::from(10));
    println!("{result}");

    // A closure can capture host state. The capture must be `'static`, so share it.
    let count = Rc::new(Cell::new(0));
    let count_for_proc = Rc::clone(&count);
    evaluator
        .root_env()
        .define_native_proc("bump", move |proc_name, args, _| {
            if !args.is_nil() {
                return Err(arity_error(proc_name, 0..=0, args.len()));
            }
            let next = count_for_proc.get() + 1;
            count_for_proc.set(next);
            Ok(Expr::from(next))
        });
    assert_eq!(evaluator.eval_str("(bump)").unwrap(), Expr::from(1));
    assert_eq!(evaluator.eval_str("(bump)").unwrap(), Expr::from(2));
    assert_eq!(count.get(), 2);

    // Each evaluator's natives close over their own host state. A function
    // pointer could only reach a shared static or thread-local, so a second
    // interpreter would see the first interpreter's balance.
    let alice_bank = Rc::new(RefCell::new(Bank { balance: 0.0 }));
    let bob_bank = Rc::new(RefCell::new(Bank { balance: 0.0 }));

    let alice = Evaluator::with_prelude();
    let bob = Evaluator::with_prelude();
    install(&alice, &alice_bank);
    install(&bob, &bob_bank);

    alice_bank.borrow_mut().balance = 100.0;
    assert_eq!(alice.eval_str("(balance)").unwrap(), Expr::from(100.0));

    let alice_balance = alice
        .eval_str(
            r#"
            (deposit 40)
            (deposit 2)
            (balance)
            "#,
        )
        .unwrap();
    let bob_balance = bob.eval_str("(deposit 7)").unwrap();

    assert_eq!(alice_balance, Expr::from(142.0));
    assert_eq!(bob_balance, Expr::from(7.0));
    assert_eq!(alice_bank.borrow().balance, 142.0);
    assert_eq!(bob_bank.borrow().balance, 7.0);
    assert_eq!(bob.eval_str("(balance)").unwrap(), Expr::from(7.0));

    println!("alice: {alice_balance}");
    println!("bob:   {bob_balance}");
}
