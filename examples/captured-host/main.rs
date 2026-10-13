//! Each evaluator's natives close over their own host state.
//!
//! A native used to be a bare function pointer, so the only way to reach host
//! data was a `static` or `thread_local`. Those are shared by every evaluator
//! on the thread: a second interpreter would see the first interpreter's
//! balance. `define_native_proc` now accepts a closure, so each registration
//! captures its own `Rc`.
//!
//! Run with `cargo run --example captured-host`.

use std::cell::RefCell;
use std::rc::Rc;

use rusche::{
    arity_error,
    utils::{eval_into_num, get_exact_1_arg},
    EvalContext, EvalResult, Evaluator, Expr, List,
};

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
    let alice_bank = Rc::new(RefCell::new(Bank { balance: 0.0 }));
    let bob_bank = Rc::new(RefCell::new(Bank { balance: 0.0 }));

    let alice = Evaluator::with_prelude();
    let bob = Evaluator::with_prelude();
    install(&alice, &alice_bank);
    install(&bob, &bob_bank);

    // The host can seed a value the script then reads. Nothing is stored in a global.
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

    // Each closure updated only the bank it captured.
    assert_eq!(alice_balance, Expr::from(142.0));
    assert_eq!(bob_balance, Expr::from(7.0));
    assert_eq!(alice_bank.borrow().balance, 142.0);
    assert_eq!(bob_bank.borrow().balance, 7.0);
    // Bob's script never ran against Alice's bank, and Alice's deposits never
    // landed in Bob's. A thread-local host would have mixed these.
    assert_eq!(bob.eval_str("(balance)").unwrap(), Expr::from(7.0));

    println!("alice: {alice_balance}");
    println!("bob:   {bob_balance}");
}
