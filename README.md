# Rusche

[![CI](https://github.com/chanryu/rusche/actions/workflows/ci.yml/badge.svg)](https://github.com/chanryu/rusche/actions)
[![Code Coverage](https://codecov.io/gh/chanryu/rusche/graph/badge.svg?token=EHPCRUWK96)](https://codecov.io/gh/chanryu/rusche)

## Overview

Rusche is a library for writing an interpreter for a Scheme-like language in Rust. It lets you embed a Scheme interpreter into your Rust applications, allowing you to use Scheme as a scripting language or to create standalone Scheme interpreters.


## Features

- Minimalistic library with no rumtime dependency
- Garbage collection
- Tail-call optimization
- Interoperability with hosting Rust application via `Foreign` data type.

## Installation

_To be filled after publishing Rusche to crate.io._

## Usage

### Rusche language

```scheme
(define (fizzbuzz n)
    (define (rem0 n m) (= (% n m) 0))
    (cond ((rem0 n 15) "FizzBuzz")
          ((rem0 n 3) "Fizz")
          ((rem0 n 5) "Buzz")
          (#t n)))

(print "Enter a number: ")

(let ((n 1)
      (m (read-num)))
    (while (<= n m)
        (println (fizzbuzz n))
        (set! n (+ n 1))))
```

### Implementing or embedding Rusche interpreter

```rust
use rusche::eval::Evaluator;
use rusche::lexer::tokenize;
use rusche::parser::Parser;

let evaluator = Evaluator::with_prelude();
let tokens = tokenize("(+ 1 (% 9 2))")?;
let mut parser = Parser::with_tokens();

if let Some(expr) = parser.parse()? {
    match evaluator.eval(&expr) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
}

```

To learn about how to implement a standalone interpreter with REPL, have a look at [examples/rusche-cli](https://github.com/chanryu/rusche/blob/readme/examples/rusche-cli/repl.rs).

## Documentation

- [Rusche Language Reference](https://github.com/chanryu/rusche/wiki/Rusche-Language-Reference)

<!--
## Background

About five years ago, I decided to learn more about Lisp by [writing a Lisp interpreter in C++](https://github.com/chanryu/mlisp). 
I had heard a lot about how insightful Lisp can be, so I wanted to experience it for myself.

I could have just learned a Lisp variant like Scheme, but my goal wasn't to become a Lisp programmer. Instead, I wanted to 
understand what Paul Graham talked about in [The Roots of Lisp](https://paulgraham.com/rootsoflisp.html):

> In 1960, John McCarthy published a remarkable paper in which he did for programming something like what Euclid did for 
> geometry. He showed how, given a handful of simple operators and a notation for functions, you can build a whole 
> programming language. He called this language Lisp, for "List Processing," because one of his key ideas was to use a 
> simple data structure called a list for both code and data.

The project was successful. Writing the interpreter taught me what makes Lisp different from other languages. Lisp's macro system
was very enlightening, and writing C++ code to make macros work felt like meditation.

Back then, I chose C++ because it's the programming language I'm most comfortable with. If I were to do it again today, I would 
probably choose Swift. Even though I'm still better at C++, Swift makes me more productive.

This time, I'm using Rust because I have a different goal -- Learning Rust. I'm not sure how this project will turn out, but there are a few 
Rust features I want to explore along the way.
-->
