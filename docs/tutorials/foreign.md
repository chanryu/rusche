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
pub type Foreign = Rc<dyn Any>;
```

Store one as `Expr::Foreign(Rc::new(value))`. Important properties:

- **Equality** is pointer identity (`Rc::ptr_eq`), not structural equality.
- **Display** prints `<foreign: 0x…>` — there is no printable contents by default.
- The concrete type inside the `Rc` is what you downcast and what you register
  with a garbage-collection tracer.

For mutable data, wrap the payload in `RefCell` (or another interior-mutability
type) before putting it in the `Rc`:

```rust
use std::{cell::RefCell, rc::Rc};
use rusche::Expr;

struct Point {
    x: f64,
    y: f64,
}

let point = Expr::Foreign(Rc::new(RefCell::new(Point { x: 0.0, y: 0.0 })));
```

## Downcasting

Use `eval_into_foreign` to evaluate an expression to `Rc<dyn Any>`, then
`downcast` to the **concrete type inside the `Rc`** (here `RefCell<Point>`, not
`Rc<RefCell<Point>>`):

```rust
use std::{cell::RefCell, rc::Rc};
use rusche::{eval_into_foreign, EvalContext, EvalError, Expr};

type PointCell = RefCell<Point>;

fn eval_into_point(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<PointCell>, EvalError> {
    eval_into_foreign(proc_name, expr, context)?
        .downcast::<PointCell>()
        .map_err(|_| {
            EvalError::new(
                rusche::ErrorKind::Type,
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
    eval_into_foreign, get_exact_1_arg, get_exact_2_args,
    utils::eval_into_num, EvalContext, EvalError, EvalResult, Evaluator, Expr, List, NIL,
};

struct Point {
    x: f64,
    y: f64,
}

type PointCell = RefCell<Point>;

fn eval_into_point(
    proc_name: &str,
    expr: &Expr,
    context: &EvalContext,
) -> Result<Rc<PointCell>, EvalError> {
    eval_into_foreign(proc_name, expr, context)?
        .downcast::<PointCell>()
        .map_err(|_| {
            EvalError::new(
                rusche::ErrorKind::Type,
                format!("{proc_name}: `{expr}` does not evaluate to a point"),
            )
            .with_span(expr.span())
        })
}

fn point(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (x_expr, y_expr) = get_exact_2_args(proc_name, args)?;
    let x = eval_into_num(proc_name, x_expr, context)?;
    let y = eval_into_num(proc_name, y_expr, context)?;
    Ok(Expr::Foreign(Rc::new(RefCell::new(Point { x, y }))))
}

fn point_x(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let arg = get_exact_1_arg(proc_name, args)?;
    let point = eval_into_point(proc_name, arg, context)?;
    let x = point.borrow().x;
    Ok(Expr::from(x))
}

fn point_move(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (point_expr, dx_expr, dy_expr) = rusche::utils::get_exact_3_args(proc_name, args)?;
    let point = eval_into_point(proc_name, point_expr, context)?;
    let dx = eval_into_num(proc_name, dx_expr, context)?;
    let dy = eval_into_num(proc_name, dy_expr, context)?;

    {
        let mut point = point.borrow_mut();
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

## Holding `Expr` values and GC tracers

If a foreign object stores `Expr` values — especially closures — the garbage
collector cannot see them unless you register a tracer for that concrete type:

```rust
use std::{cell::RefCell, rc::Rc};
use rusche::{Evaluator, Expr};

type ExprVec = RefCell<Vec<Expr>>;

let evaluator = Evaluator::with_prelude();
evaluator.register_foreign_tracer::<ExprVec>(|vec, trace| {
    vec.borrow().iter().for_each(trace);
});
```

The type parameter on `register_foreign_tracer` must match the type used in
`Rc::new(...)` and in `downcast`. Without a tracer, a closure that lives only
inside the foreign object can have its environment collected; calling it later
fails.

The vector builtins in
[`vec.rs`](../../crates/rusche-cli/src/builtin/vec.rs) show the complete pattern:
create, push, get, and a tracer over every stored `Expr`. The dict builtins in
[`dict.rs`](../../crates/rusche-cli/src/builtin/dict.rs) show a Foreign type whose
keys are a separate Rust enum (not `Expr`), with a tracer over values only.

## Borrowing and re-entrancy

Do not hold a `RefCell` borrow across a call to `eval`. Evaluation can re-enter
the same foreign object (for example if a stored closure calls back into your
native procedure), which would panic on a nested borrow. Evaluate arguments
first, then borrow briefly to mutate, as `point_move` does above.

## Next steps

- [How to embed the Rusche interpreter](embedding.md)
- [How to write a native function](native-functions.md)
- Full vector wrapper: [`crates/rusche-cli/src/builtin/vec.rs`](../../crates/rusche-cli/src/builtin/vec.rs)
