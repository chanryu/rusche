# Rusche

[![ci](https://github.com/chanryu/rusche/actions/workflows/ci.yml/badge.svg)](https://github.com/chanryu/rusche/actions)
[![coverage](https://codecov.io/gh/chanryu/rusche/graph/badge.svg?token=EHPCRUWK96)](https://codecov.io/gh/chanryu/rusche)
[![crates.io](https://img.shields.io/crates/v/rusche)](https://crates.io/crates/rusche)
[![docs.rs](https://img.shields.io/docsrs/rusche/latest)](https://docs.rs/rusche/latest/rusche/)

## Overview

Rusche is a library for writing an interpreter for a Scheme-like language in Rust. It lets you embed a Scheme-like interpreter into your Rust applications, allowing you to use it as a scripting language or to create standalone interpreters.

Rusche is deliberately *Scheme-like*, not Scheme: it uses Scheme's syntax but keeps the core language small. See [Differences from Scheme](#differences-from-scheme).

## Features

- Minimalistic library with zero dependencies
- Lambdas and closures, with splat-style rest parameters (`(define (f a *rest) ...)`, `(lambda (*args) ...)`)
- Lexical scopes and binding
- Macros using special forms like quasiquote (`` ` ``), unquote (`,`), unquote-splicing (`,@`)
- Garbage collection
- Tail-call optimization, plus a call-depth limit so runaway recursion is an error rather than a stack overflow
- Interoperability with the hosting Rust application via user-defined (a.k.a. native) functions and the `Foreign` data type
- Structured errors with source spans, optional help, and a call trace. Given `first.rsc`:
  ```scheme
  (define (first-or-zero lst)
      (if lst (car lst) 0))   ;; `()` is not false in Rusche

  (first-or-zero (list))
  ```
  `rusche-cli` reports:
  ```
  error: `lst` evaluated to `()`, expected `true` or `false`
    --> first.rsc:2:9
    1| (define (first-or-zero lst)
    2|     (if lst (car lst) 0))   ;; `()` is not false in Rusche
     |         ^^^
    = help: conditions must be booleans; use `(not (null? x))` to test for an empty list
    = in `first-or-zero`, called at first.rsc:4:1
  ```

## Usage

### Implementing or embedding Rusche interpreter

```rust
use rusche::{
    utils::{eval_into_num, get_exact_2_args},
    EvalContext, EvalResult, Evaluator, Expr, List,
};

// A native function: Rust code that scripts can call. Arguments arrive
// unevaluated; the helpers in `rusche::utils` evaluate and type-check them.
fn hypot(proc_name: &str, args: &List, context: &EvalContext) -> EvalResult {
    let (a, b) = get_exact_2_args(proc_name, args)?;
    let a = eval_into_num(proc_name, a, context)?;
    let b = eval_into_num(proc_name, b, context)?;
    Ok(Expr::from(a.hypot(b)))
}

fn main() {
    // Built-ins plus the prelude (`+`, `*`, `sqrt`, `map`, `let`, ...)
    let evaluator = Evaluator::default();

    // Expose host functionality to scripts
    evaluator.root_env().define_native_proc("hypot", hypot);

    // Tokenize, parse, and evaluate every top-level form; the last value is returned
    let result = evaluator
        .eval_str(
            r#"
            (map (lambda (p) (apply hypot p)) '((3 4) (5 12)))
            "#,
        )
        .unwrap();

    println!("{result}"); // (5 13)
    assert_eq!(result, evaluator.eval_str("'(5 13)").unwrap());

    // Lex, parse, and eval failures share one `Error` type with a source span
    let err = evaluator.eval_str("(hypot \"nine\" 4)").unwrap_err();
    println!("{err}"); // 1:8-13: hypot: `"nine"` evaluated to `"nine"`, expected a number
}
```

This is [`examples/readme-demo`](examples/readme-demo/main.rs); run it with `cargo run --example readme-demo`. `Evaluator::eval_str` covers most hosts; for incremental parsing (e.g. a multi-line REPL) use `tokenize`, `Parser`, and `Evaluator::eval` directly, as shown in the [embedding tutorial](docs/tutorials/embedding.md#lower-level-pipeline). Use `root_env().define` to inject plain values and `Foreign` for Rust objects (see the [tutorials](#documentation)).

For a standalone REPL and file runner built on the library, see [`rusche-cli`](#rusche-cli-example-host) below.

#### Limits and garbage collection

- **Call depth.** Evaluation fails with an error (instead of overflowing the Rust stack) once
  more than `Evaluator::max_call_depth()` procedure calls are active. The default is tuned for
  the main thread of a debug build; adjust it with `Evaluator::set_max_call_depth` to match the
  stack size of the thread you evaluate on. Tail calls do not count towards the limit; see
  [Tail Calls](docs/language-reference.md#tail-calls) for which positions are tail positions.
- **Garbage collection.** Environments captured by closures form reference cycles, so the
  evaluator collects them: automatically after a top-level `Evaluator::eval` once the number of
  live environments reaches `Evaluator::gc_threshold()` (set it to `None` to disable), or on
  demand with `Evaluator::collect_garbage`. Reachability starts from the root environment and
  the value `eval` just returned. If your host stores `Expr` values inside a `Foreign` object,
  register a tracer with `Evaluator::register_foreign_tracer` so the collector can see them
  (see `crates/rusche-cli/src/builtin/vec.rs`).

### Core language

The core language is everything available from `Evaluator::default()` (built-ins plus the prelude). No I/O, no vectors — those are host concerns. Full detail: [language reference](docs/language-reference.md).

```scheme
(define (fizzbuzz n)
    (define (div? n m) (= (% n m) 0))
    (cond ((div? n 15) "FizzBuzz")
          ((div? n 3) "Fizz")
          ((div? n 5) "Buzz")
          (else n)))

(map fizzbuzz '(1 2 3 4 5 15))
; => (1 2 "Fizz" 4 "Buzz" "FizzBuzz")
```

#### Differences from Scheme

- **Booleans.** Literals are `true`/`false`; there is no `#t`/`#f`. Conditions must be booleans; `'()` is just a value. Predicates return `true` or `false`.
- **Equality.** `eq?` compares structurally and `=` is an alias for it.
- **Lists only.** `cons` requires a list as its second argument; there are no dotted pairs or `set-car!`/`set-cdr!`. Lists are immutable and shared.
- **Numbers.** All numbers are 64-bit floats.
- **Macros.** `defmacro` (unhygienic) is the macro system; there is no `syntax-rules`.
- **Names.** Type checks end in `?` (`num?`, `str?`, `sym?`, `proc?`, `atom?`); number and string ops use a type prefix (`num-add`, `str-append`) with short prelude aliases (`+`, `sqrt`, …); list/binding forms keep classic names (`car`, `lambda`); conversions use `type1->type2` (`num->str`). There are no Scheme spellings such as `number?` or `string-append`.
- **Small surface.** `if` without an else branch and `define` return `()`. There is no `do`. Characters, vectors, ports, and continuations are host concerns (the prelude offers macros such as `when` / `unless` / `case` / named `let` / `letrec` / `defrecord`).
- **`apply` and `eval` are syntax.** Like `if` and `begin`, they are recognised by the evaluator rather than bound as procedures, so they cannot be passed as values.
- **Rest parameters** are spelled with a `*` prefix, as in Ruby or Python, instead of Scheme's dotted syntax: `(define (f a *rest) ...)` and `(lambda (*args) ...)`. The parameter list is always a list.

### `rusche-cli` (example host)

[`rusche-cli`](https://github.com/chanryu/rusche/tree/main/crates/rusche-cli/) is a sample interpreter built on the library. It is **not** part of the core language. On top of `Evaluator::with_prelude()`, it adds:

- I/O: `display`, `write`, `newline`, `read`, `exit`, `load`
- System helpers: `sys-getenv`, `sys-clock`, `sys-random`, `command-line`
- `vec` and `dict` foreign types
- A REPL with history, multi-line editing, and meta-commands (`,help`, `,load`, …)

The `*.rsc` scripts under [`examples/`](https://github.com/chanryu/rusche/tree/main/examples) are written for this host (for example [fizzbuzz.rsc](https://github.com/chanryu/rusche/blob/main/examples/fizzbuzz.rsc) uses `display` and `read`). See [`docs/rusche-cli.md`](docs/rusche-cli.md).

```bash
cargo run -p rusche-cli
cargo run -p rusche-cli -- examples/fizzbuzz.rsc
cargo install --path crates/rusche-cli
```

## Documentation

- [Language reference](docs/language-reference.md) -- core special forms and built-ins (crate only)
- [`rusche-cli`](docs/rusche-cli.md) -- running the example interpreter: options, the REPL, I/O, `vec`, and `dict`
- [API documentation on docs.rs](https://docs.rs/rusche/latest/rusche/) -- embedding Rusche in a Rust application
- Tutorials for host applications:
  - [Embedding the interpreter](docs/tutorials/embedding.md)
  - [Writing a native function](docs/tutorials/native-functions.md)
  - [Writing a foreign object wrapper](docs/tutorials/foreign.md)
