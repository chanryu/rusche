# Rusche

[![ci](https://github.com/chanryu/rusche/actions/workflows/ci.yml/badge.svg)](https://github.com/chanryu/rusche/actions)
[![coverage](https://codecov.io/gh/chanryu/rusche/graph/badge.svg?token=EHPCRUWK96)](https://codecov.io/gh/chanryu/rusche)
[![crates.io](https://img.shields.io/crates/v/rusche)](https://crates.io/crates/rusche)
[![docs.rs](https://img.shields.io/docsrs/rusche/latest)](https://docs.rs/rusche/latest/rusche/)

## Overview

Rusche is a library for writing an interpreter for a Scheme-like language in Rust. It lets you embed a Scheme interpreter into your Rust applications, allowing you to use Scheme as a scripting language or to create standalone Scheme interpreters.


## Features

- Minimalistic library with zero dependency
- Lambdas and closures
- Lexical scopes and binding
- Macros using special forms like qusiquote (`` ` ``), unquote (`,`), unquote-splicing (`,@`)
- Garbage collection
- Tail-call optimization
- Interoperability with hosting Rust application via user-defined (a.k.a native) functions and `Foreign` data type.
- `Span` support for informative error message, for example:
  ```
  repl:01❯ (define plus
  ....:02❯     (lambda (x 7)   ;; 7 should be y
  ....:03❯         (+ x y)))

  error: 7 is not a symbol.
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

// Create Evaluator with basic primitives
let evaluator = Evaluator::default();

// Evaluate the parsed expression
let result = evaluator.eval(&expr);

assert_eq!(result, Ok(Expr::from(2)));

println!("{}", result.unwrap()); // this prints out 2
```

To learn about how to implement a standalone interpreter with REPL, have a look at [examples/rusche-cli](https://github.com/chanryu/rusche/tree/main/examples/rusche-cli/).

### Rusche language

Here's a quick example to show what's possible with the Rusche language.

```scheme
(defun fizzbuzz (n)
    (defun div? (n m) (= (% n m) 0))
    (cond ((div? n 15) "FizzBuzz")
          ((div? n 3) "Fizz")
          ((div? n 5) "Buzz")
          (#t n)))

(print "Enter a number to fizzbuzz: ")

(let ((n 1)
      (m (num-parse (read)))) ; read a number from stdio and store it to `m`
    (while (<= n m)
        (println (fizzbuzz n))
        (set! n (+ n 1))))
```

To see more examples, please checkout *.rsc files in the [examples](https://github.com/chanryu/rusche/tree/main/examples) directory.

Also, you can run `rusche-cli` yourself with the following command:
```bash
cargo run --example rusche-cli
```

## Documentation

- [Rusche Language Reference](https://github.com/chanryu/rusche/wiki/Rusche-Language-Reference)
