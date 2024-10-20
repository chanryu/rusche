# Rusche

[![ci](https://github.com/chanryu/rusche/actions/workflows/ci.yml/badge.svg)](https://github.com/chanryu/rusche/actions)
[![coverage](https://codecov.io/gh/chanryu/rusche/graph/badge.svg?token=EHPCRUWK96)](https://codecov.io/gh/chanryu/rusche)
[![crates.io](https://img.shields.io/crates/v/rusche)](https://crates.io/crates/rusche)
[![docs.rs](https://img.shields.io/docsrs/rusche/latest)](https://docs.rs/rusche/latest/rusche/)

## Overview

Rusche is a library for writing an interpreter for a Scheme-like language in Rust. It lets you embed a Scheme interpreter into your Rust applications, allowing you to use Scheme as a scripting language or to create standalone Scheme interpreters.


## Features

- Minimalistic library with no rumtime dependency
- Garbage collection
- Tail-call optimization
- Interoperability with hosting Rust application via `Foreign` data type.
- Token span support for proper error message handling, for example:
  ```
  rusche:01❯ (define plus
  ......:02❯     (lambda (x 7)   ;; 7 should be y
  ......:03❯         (+ x y)))
  error: 7 is not a symbol.
    1| (define plus
    2|     (lambda (x 7)
     |                ^
  ```

## Usage

### Implementing or embedding Rusche interpreter

```rust
use rusche::{Evaluator, Parser, tokenize};

let source = "(+ 1 (% 9 2))";

// Create Evaluator with basic primitives
let evaluator = Evaluator::with_prelude();

let mut parser = Parser::new();

// Tokenize source and add tokens to parser
let tokens = tokenize(source, None).unwrap();
parser.add_tokens(tokens);

// Parse tokens into an expression
let expr = parser.parse().unwrap().unwrap();

// Evaluate the parsed expression
let result = evaluator.eval(&expr).unwrap();

println!("{}", result); // this will print 2
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
      (m (read-num))) ; read a number from stdio and store it to `m`
    (while (<= n m)
        (println (fizzbuzz n))
        (set! n (+ n 1))))
```

To see more example, please checkout *.rsc files in the [examples](https://github.com/chanryu/rusche/tree/main/examples) directory.

## Documentation

- [Rusche Language Reference](https://github.com/chanryu/rusche/wiki/Rusche-Language-Reference)
