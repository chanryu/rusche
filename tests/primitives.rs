mod common;

use common::{cons, num, test_eval};
use rusp::list::List;

#[test]
fn test_cond() {
    assert_eq!(test_eval("(cond (t 0) (f 1))"), Ok(num(0)));
    assert_eq!(test_eval("(cond (f 0) (t 1))"), Ok(num(1)));
}

#[test]
fn test_quote() {
    assert_eq!(test_eval("'1"), Ok(num(1)));
    assert_eq!(test_eval("'(1)"), Ok(cons(num(1), List::Nil).to_expr()));
    assert_eq!(
        test_eval("'(1 2)"),
        Ok(cons(num(1), cons(num(2), List::Nil)).to_expr())
    );
}
