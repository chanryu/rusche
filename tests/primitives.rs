mod common;

#[macro_use]
extern crate rusp;

use common::test_eval;
use rusp::{
    expr::shortcuts::num,
    list::{cons, List},
};

#[test]
fn test_cond() {
    assert_eq!(test_eval("(cond (t 0) (f 1))"), Ok(num(0)));
    assert_eq!(test_eval("(cond (f 0) (t 1))"), Ok(num(1)));
}

#[test]
fn test_quote() {
    assert_eq!(test_eval("'1"), Ok(num(1)));
    assert_eq!(test_eval("'(1)"), Ok(list!(num(1)).into()));
    assert_eq!(test_eval("'(1 2)"), Ok(list!(num(1), num(2)).into()));
}
