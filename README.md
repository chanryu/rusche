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
- Lambdas and closures, with Scheme-style rest parameters (`(define (f a . rest) ...)`, `(lambda args ...)`)
- Lexical scopes and binding
- Macros using special forms like quasiquote (`` ` ``), unquote (`,`), unquote-splicing (`,@`)
- Garbage collection
- Tail-call optimization, plus a call-depth limit so runaway recursion is an error rather than a stack overflow
- Interoperability with the hosting Rust application via user-defined (a.k.a. native) functions and the `Foreign` data type
- `Span` support for informative error messages, for example:
  ```
  repl:01❯ (define plus
  ....:02❯     (lambda (x 7)   ;; 7 should be y
  ....:03❯         (+ x y)))

  error: 7 is not a symbol.
    1| (define plus
    2|     (lambda (x 7)
     |                ^        ;; Thanks to `Span`, we can show exactly where the error originates
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

To learn about how to implement a standalone interpreter with REPL, have a look at [examples/rusche-cli](https://github.com/chanryu/rusche/tree/main/examples/rusche-cli/).

#### Limits and garbage collection

- **Call depth.** Evaluation fails with an error (instead of overflowing the Rust stack) once
  more than `Evaluator::max_call_depth()` procedure calls are active. The default is tuned for
  the main thread of a debug build; adjust it with `Evaluator::set_max_call_depth` to match the
  stack size of the thread you evaluate on. Tail calls do not count towards the limit.
- **Garbage collection.** Environments captured by closures form reference cycles, so the
  evaluator collects them: automatically after a top-level `Evaluator::eval` once the number of
  live environments reaches `Evaluator::gc_threshold()` (set it to `None` to disable), or on
  demand with `Evaluator::collect_garbage`. Reachability starts from the root environment and
  the value `eval` just returned. If your host stores `Expr` values inside a `Foreign` object,
  register a tracer with `Evaluator::register_foreign_tracer` so the collector can see them
  (see `vec.rs` in `rusche-cli`).

### Rusche language

Here's a quick example to show what's possible with the Rusche language -- the same program as [examples/fizzbuzz.rsc](https://github.com/chanryu/rusche/blob/main/examples/fizzbuzz.rsc), here using the core procedure names.

```scheme
(define (fizzbuzz n)
    (define (div? n m) (= (% n m) 0))
    (cond ((div? n 15) "FizzBuzz")
          ((div? n 3) "Fizz")
          ((div? n 5) "Buzz")
          (else n)))

(display "Enter a number to fizzbuzz: ")

(let ((n 1)
      (m (num-parse (read)))) ; read a number from stdio and store it to `m`
    (while (<= n m)
        (display (fizzbuzz n)) (newline)
        (set! n (+ n 1))))
```

`display`, `newline`, and `read` are I/O procedures provided by `rusche-cli`, not by the core library. `rusche-cli` also defines Scheme-style aliases for the core procedures (`number?`, `string-append`, `string->number`, `modulo`, ...); see [scheme.rs](https://github.com/chanryu/rusche/blob/main/examples/rusche-cli/builtin/scheme.rs). The example scripts use those aliases.

To see more examples, please check out the *.rsc files in the [examples](https://github.com/chanryu/rusche/tree/main/examples) directory.

You can run `rusche-cli` yourself, either as a REPL or on a script:
```bash
cargo run --example rusche-cli
cargo run --example rusche-cli -- examples/fizzbuzz.rsc
```

#### Differences from Scheme

- **Truthiness.** `'()` is the only false value; `#t` is `1` and `#f` is `'()`. There is no boolean type, and predicates return `1` or `()`.
- **Equality.** `eq?` compares structurally and `=` is an alias for it.
- **Lists only.** `cons` requires a list as its second argument; there are no dotted pairs or `set-car!`/`set-cdr!`. Lists are immutable and shared.
- **Numbers.** All numbers are 64-bit floats.
- **Macros.** `defmacro` (unhygienic) is the macro system; there is no `syntax-rules`.
- **Names.** Core procedures use short names (`num?`, `str-append`, `atom?`, `%`, ...) rather than the Scheme ones; `rusche-cli` adds the Scheme spellings as aliases.
- **Small surface.** `<`, `>`, `<=`, `>=` are binary; `if` without an else branch and `define` return `()`; there is no `let*`, named `let`, `case`, `do`, `when`, or `unless`. No characters, vectors, ports, or continuations.
- **Rest parameters** do use Scheme syntax: `(define (f a . rest) ...)` and `(lambda args ...)`.

## Documentation

- [Rusche Language Reference](https://github.com/chanryu/rusche/wiki/Rusche-Language-Reference) -- every special form and built-in procedure, with examples
- [API documentation on docs.rs](https://docs.rs/rusche/latest/rusche/) -- for embedding Rusche in a Rust application
