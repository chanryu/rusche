# Tutorial: Writing a foreign object wrapper (`Foreign`)

[`Foreign`](https://docs.rs/rusche/latest/rusche/type.Foreign.html) lets the host
store arbitrary Rust values in the interpreter. The core language defines no
foreign types; hosts add them. Pair a foreign value with native procedures that
construct and operate on it — as [`rusche-cli`](../rusche-cli.md) does for `vec`.

A small runnable example lives in
[`examples/tutorial-foreign`](../../examples/tutorial-foreign/main.rs). The fuller
vector API is in [`crates/rusche-cli/src/builtin/vec.rs`](../../crates/rusche-cli/src/builtin/vec.rs).

## What `Foreign` is

```rust
pub type Foreign = Rc<dyn ForeignObject>;
```

A foreign value implements [`ForeignObject`](https://docs.rs/rusche/latest/rusche/trait.ForeignObject.html).
Store one as `Expr::foreign(value)`. Important properties:

- **Equality** is pointer identity (`Rc::ptr_eq`), not structural equality.
- **Display** uses [`ForeignObject::fmt`](https://docs.rs/rusche/latest/rusche/trait.ForeignObject.html#method.fmt).
  The default is `<type_name>`.
- **Garbage collection** uses [`ForeignObject::trace`](https://docs.rs/rusche/latest/rusche/trait.ForeignObject.html#method.trace).
  The default reports nothing. Override it when the value stores `Expr`s.

For mutable data, wrap the payload in `RefCell` (or another interior-mutability
type) before putting it in the `Rc`:

```rust
use std::cell::RefCell;
use rusche::{Expr, ForeignObject};

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

struct PointCell(RefCell<Point>);

impl ForeignObject for PointCell {
    fn type_name(&self) -> &'static str {
        "point"
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let point = self.0.borrow();
        write!(f, "<point {} {}>", point.x, point.y)
    }
}

let point = Expr::foreign(PointCell(RefCell::new(Point { x: 0.0, y: 0.0 })));
```

## Downcasting

`as_foreign::<T>()` downcasts to the type that implements `ForeignObject` (here
`PointCell`, which must be a type your crate owns). Evaluate the argument first:

```rust
use std::rc::Rc;
use rusche::{eval, ErrorKind, EvalContext, EvalError, Expr};

fn eval_into_point(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<PointCell>, EvalError> {
    let value = eval(expr, context)?;
    value.as_foreign::<PointCell>().ok_or_else(|| {
        EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: `{expr}` does not evaluate to a point"),
        )
        .with_span(expr.span())
    })
}
```

Keeping `span: expr.span()` on the error lets the host highlight the bad argument.

## Worked example: `Point`

Register constructors and accessors as native procedures (see
[native functions](native-functions.md)):

```rust
use std::{cell::RefCell, rc::Rc};
use rusche::{
    eval, eval_into_num, get_exact_1_arg, get_exact_2_args, get_exact_3_args, ErrorKind,
    EvalContext, EvalError, EvalResult, Evaluator, Expr, ForeignObject, List, NIL,
};

#[derive(Debug)]
struct Point {
    x: f64,
    y: f64,
}

struct PointCell(RefCell<Point>);

impl ForeignObject for PointCell {
    fn type_name(&self) -> &'static str {
        "point"
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let point = self.0.borrow();
        write!(f, "<point {} {}>", point.x, point.y)
    }
}

fn eval_into_point(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<PointCell>, EvalError> {
    let value = eval(expr, context)?;
    value.as_foreign::<PointCell>().ok_or_else(|| {
        EvalError::new(
            ErrorKind::Type,
            format!("{proc_name}: `{expr}` does not evaluate to a point"),
        )
        .with_span(expr.span())
    })
}

fn point(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (x_expr, y_expr) = get_exact_2_args(proc_name, args)?;
    let x = eval_into_num(proc_name, x_expr, context)?;
    let y = eval_into_num(proc_name, y_expr, context)?;
    Ok(Expr::foreign(PointCell(RefCell::new(Point { x, y }))))
}

fn point_x(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let point = eval_into_point(proc_name, arg, context)?;
    let x = point.0.borrow().x;
    Ok(Expr::from(x))
}

fn point_move(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (point_expr, dx_expr, dy_expr) = get_exact_3_args(proc_name, args)?;
    let point = eval_into_point(proc_name, point_expr, context)?;
    let dx = eval_into_num(proc_name, dx_expr, context)?;
    let dy = eval_into_num(proc_name, dy_expr, context)?;

    {
        let mut point = point.0.borrow_mut();
        point.x += dx;
        point.y += dy;
    }
    Ok(NIL)
}
```

Usage from Rusche:

```scheme
(define p (point 1 2))
(point-x p)          ; => 1
(point-move p 3 4)
(point-x p)          ; => 4
```

Run the example with:

```bash
cargo run --example tutorial-foreign
```

## Holding `Expr` values

If a foreign object stores `Expr` values — especially closures — override
`trace` so the garbage collector can see them. The default reports nothing, and
a closure that lives only inside the object can have its environment collected.

```rust
use std::cell::RefCell;
use rusche::{Expr, ForeignObject};

#[derive(Debug)]
struct ExprVec(RefCell<Vec<Expr>>);

impl ForeignObject for ExprVec {
    fn trace(&self, trace: &mut dyn FnMut(&Expr)) {
        let items: Vec<Expr> = self.0.borrow().iter().cloned().collect();
        for item in &items {
            trace(item);
        }
    }
}
```

Copy the `Expr`s out before calling `trace`. Holding the `RefCell` borrow across
the callback can panic if marking walks back into the same object.

The vector builtins in
[`vec.rs`](../../crates/rusche-cli/src/builtin/vec.rs) show the complete pattern:
create, push, get, and `trace` over every stored `Expr`. The dict builtins in
[`dict.rs`](../../crates/rusche-cli/src/builtin/dict.rs) show a foreign type whose
keys are a separate Rust enum (not `Expr`), with `trace` over values only. They
also override `fmt`, so `write` and the REPL print `<vec 1 2 3>`.

## Borrowing and re-entrancy

Do not hold a `RefCell` borrow across a call to `eval`. Evaluation can re-enter
the same foreign object (for example if a stored closure calls back into your
native procedure), which would panic on a nested borrow. Evaluate arguments
first, then borrow briefly to mutate, as `point_move` does above.

## Next steps

- [How to embed the Rusche interpreter](embedding.md)
- [How to write a native function](native-functions.md)
- Full vector wrapper: [`crates/rusche-cli/src/builtin/vec.rs`](../../crates/rusche-cli/src/builtin/vec.rs)
