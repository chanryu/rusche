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
- Structured errors with source spans, optional help, and a call trace, for example:
  ```
  repl❯ (define plus
  ....❯     (lambda (x 7)   ;; 7 should be y
  ....❯         (+ x y)))

  error: `7` is not a symbol
    --> <repl>:2:16
    1| (define plus
    2|     (lambda (x 7)
     |                ^
  ```

## Usage

### Implementing or embedding Rusche interpreter

```rust
use rusche::{tokenize, Evaluator, Expr, Parser};

let source = "(+ 1 (% 9 2))"; // 1 + (9 % 2) = 1 + 1 = 2

// Tokenize source
let tokens = tokenize(source, None).unwrap();

// Create Parser with the tokens
let mut parser = Parser::with_tokens(tokens);

// Parse tokens into an expression
let expr = parser.parse().unwrap().unwrap();

// Create Evaluator with the built-in primitives and the prelude
let evaluator = Evaluator::default();

// Evaluate the parsed expression
let result = evaluator.eval(&expr);

assert_eq!(result, Ok(Expr::from(2)));

println!("{}", result.unwrap()); // this prints out 2
```

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

The core language is everything available from `Evaluator::default()` (built-ins plus the prelude). No I/O, no vectors, no Scheme name aliases — those are host concerns. Full detail: [language reference](docs/language-reference.md).

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

- **Booleans.** Literals are `true`/`false`. Conditions must be booleans; `'()` is just a value. Predicates return `true` or `false`. `#t`/`#f` exist only as `rusche-cli` aliases.
- **Equality.** `eq?` compares structurally and `=` is an alias for it.
- **Lists only.** `cons` requires a list as its second argument; there are no dotted pairs or `set-car!`/`set-cdr!`, and a lone `.` is a syntax error. Lists are immutable and shared.
- **Numbers.** All numbers are 64-bit floats.
- **Macros.** `defmacro` (unhygienic) is the macro system; there is no `syntax-rules`.
- **Names.** Type checks end in `?` (`num?`, `str?`); same-type ops use a type prefix (`num-add`, `str-append`); conversions use `type1->type2` (`num->str`). Scheme spellings are host aliases, not core.
- **Small surface.** `if` without an else branch and `define` return `()`; there is no named `let`, `case`, `do`, `when`, or `unless`. No characters, vectors, ports, or continuations.
- **`apply` and `eval` are syntax.** Like `if` and `begin`, they are recognised by the evaluator rather than bound as procedures, so they cannot be passed as values.
- **Rest parameters** are spelled with a `*` prefix, as in Ruby or Python, instead of Scheme's dotted syntax: `(define (f a *rest) ...)` and `(lambda (*args) ...)`. The parameter list is always a list.

### `rusche-cli` (example host)

[`rusche-cli`](https://github.com/chanryu/rusche/tree/main/crates/rusche-cli/) is a sample interpreter built on the library. It is **not** part of the core language. On top of `Evaluator::with_prelude()`, it adds:

- I/O: `display`, `write`, `newline`, `read`, `exit`, `load`
- System helpers: `getenv`, `clock`, `random`, `command-line`
- A `vec` foreign type
- Scheme-style aliases (`number?`, `modulo`, `string-append`, `string->number`, …)
- A REPL with history, multi-line editing, and meta-commands (`,help`, `,load`, …)

The `*.rsc` scripts under [`examples/`](https://github.com/chanryu/rusche/tree/main/examples) are written for this host (for example [fizzbuzz.rsc](https://github.com/chanryu/rusche/blob/main/examples/fizzbuzz.rsc) uses `display` and `modulo`). See [`docs/rusche-cli.md`](docs/rusche-cli.md).

```bash
cargo run -p rusche-cli
cargo run -p rusche-cli -- examples/fizzbuzz.rsc
cargo install --path crates/rusche-cli
```

## Documentation

- [Language reference](docs/language-reference.md) -- core special forms and built-ins (crate only)
- [`rusche-cli`](docs/rusche-cli.md) -- running the example interpreter: options, the REPL, I/O, `vec`, and Scheme aliases
- [API documentation on docs.rs](https://docs.rs/rusche/latest/rusche/) -- embedding Rusche in a Rust application
- Tutorials for host applications:
  - [Embedding the interpreter](docs/tutorials/embedding.md)
  - [Writing a native function](docs/tutorials/native-functions.md)
  - [Writing a foreign object wrapper](docs/tutorials/foreign.md)
