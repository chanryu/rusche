# Rusche

![Build and Test](https://github.com/chanryu/rusche/actions/workflows/main.yml/badge.svg?branch=readme)

Teaching myself **Rus**t by writing a **Sche**me interpreter.

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
