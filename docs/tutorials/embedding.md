# Tutorial: Embedding the Rusche interpreter

This guide shows how to embed the **core** `rusche` crate in a Rust application:
create an evaluator, parse source, evaluate expressions, and exchange values with
the host. It does not use [`rusche-cli`](../rusche-cli.md); that example host's
extras (I/O, `vec`, `dict`) are optional batteries you can add yourself.

A runnable version of the snippets lives in
[`examples/tutorial-embed`](../../examples/tutorial-embed/main.rs).

## Choose an evaluator

Rusche exposes three constructors on [`Evaluator`](https://docs.rs/rusche/latest/rusche/struct.Evaluator.html):

| Constructor | Contents |
| --- | --- |
| `Evaluator::new()` | Empty root environment — only the evaluator forms (`quote`, `quasiquote`, `begin`, `if`, `eval`, `apply`) |
| `Evaluator::with_builtin()` | Core native procedures (`num-add`, `car`, `lambda`, …) |
| `Evaluator::with_prelude()` / `Evaluator::default()` | Built-ins plus the prelude (`+`, `sqrt`, `let`, `while`, `and`, `or`, …) |

For scripting, start with `with_prelude()` (or `Default`). Use `new()` or
`with_builtin()` when you want a smaller surface.

```rust
use rusche::Evaluator;

let evaluator = Evaluator::with_prelude();
// same as Evaluator::default()
```

## Evaluate source with `eval_str`

The usual host path is `Evaluator::eval_str`, which tokenizes, parses every
top-level form, evaluates each one, and returns the last value (or `()` if the
source is empty). Lex, parse, and eval failures share the unified
[`Error`](https://docs.rs/rusche/latest/rusche/enum.Error.html) type:

```rust
use rusche::{Evaluator, Expr};

let evaluator = Evaluator::default();
let result = evaluator.eval_str("(+ 1 (% 9 2))").unwrap();
assert_eq!(result, Expr::from(2));
```

A file or buffer can hold several top-level forms:

```rust
use rusche::Evaluator;

fn eval_script(evaluator: &Evaluator, source: &str) -> Result<(), rusche::Error> {
    evaluator.eval_str(source)?;
    Ok(())
}
```

`Error` implements `Display` and `std::error::Error`. Use `message()` for the
bare text (without a leading span), `span()` for the source location, `help()`
for an optional hint, and `trace()` for the call stack collected while
unwinding. The [`rusche-cli`](../../crates/rusche-cli/src/main.rs) host prints
those against the original source lines.

### Lower-level pipeline

When you need incremental parsing (for example a multi-line REPL), use the
pieces underneath `eval_str`:

```rust
use rusche::{tokenize, Evaluator, Expr, Parser};

let source = "(+ 1 (% 9 2))";

let tokens = tokenize(source, None).unwrap();
let mut parser = Parser::with_tokens(tokens);
let expr = parser.parse().unwrap().unwrap();

let evaluator = Evaluator::default();
let result = evaluator.eval(&expr).unwrap();

assert_eq!(result, Expr::from(2));
```

- `tokenize(text, loc)` turns source into tokens. Pass `Some(Loc::…)` if you want
  spans relative to a known origin; `None` starts at the default location.
- `Parser::parse` returns `Ok(None)` when there are no more complete expressions,
  `Ok(Some(expr))` for one expression, or a `ParseError`.
- `Evaluator::eval` runs the expression in the root context and may trigger
  automatic garbage collection afterward.

## Exchange values with the host

### Define host values into the environment

Use `root_env().define` to inject numbers, strings, or other `Expr` values before
running script code:

```rust
use rusche::{Evaluator, Expr};

let evaluator = Evaluator::default();
evaluator.root_env().define("greeting", Expr::from("hello"));

let result = evaluator
    .eval_str("(str-append greeting \" world\")")
    .unwrap();

assert_eq!(result, Expr::from("hello world"));
```

For native procedures and foreign objects, see
[How to write a native function](native-functions.md) and
[How to write a foreign object wrapper](foreign.md).

### Read results back

Match on `Expr` variants after `eval`:

```rust
use rusche::Expr;

fn describe(expr: &Expr) -> String {
    match expr {
        Expr::Num(n, _) => format!("number {n}"),
        Expr::Bool(b, _) => format!("boolean {b}"),
        Expr::Str(s, _) => format!("string {s:?}"),
        Expr::Sym(name, _) => format!("symbol {name}"),
        Expr::List(list, _) => format!("list {list}"),
        Expr::Proc(proc, _) => format!("procedure <{}>", proc.fingerprint()),
        Expr::Foreign(object) => format!("foreign {:p}", object),
    }
}
```

Numbers are `f64`. Booleans are `true` / `false`; conditions in `if` must be
booleans. See the [language reference](../language-reference.md) for the full
type system.

## Call depth and garbage collection

- **Call depth.** Non-tail calls are capped by `Evaluator::max_call_depth()`
  (default `DEFAULT_MAX_CALL_DEPTH`). Exceeding it is an error instead of a Rust
  stack overflow. Adjust with `set_max_call_depth` for the thread you evaluate on.
  Tail calls do not count (see [Tail Calls](../language-reference.md#tail-calls)).
  A native procedure counts as one level while it runs, and anything it
  evaluates with `eval` nests inside it.
- **Garbage collection.** After a top-level `eval`, the evaluator may collect
  unreachable environments once their count reaches `gc_threshold()` (default
  `DEFAULT_GC_THRESHOLD`; pass `None` to `set_gc_threshold` to disable). Call
  `collect_garbage` anytime for an explicit collection.

Reachability starts from the root environment and the value `eval` just returned.
If the host stores `Expr` values (especially closures) inside a `Foreign` object,
register a tracer so the collector can see them — covered in
[How to write a foreign object wrapper](foreign.md).

## Next steps

- [How to write a native function](native-functions.md)
- [How to write a foreign object wrapper](foreign.md)
- Standalone REPL and file runner: [`crates/rusche-cli`](../../crates/rusche-cli/)
- Language surface: [language reference](../language-reference.md)
