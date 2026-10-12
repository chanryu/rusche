# `rusche-turtle`

[`rusche-turtle`](../crates/rusche-turtle) is an example host: a desktop turtle-graphics
window with a script editor, canvas, and output log. Scripts run on a worker-thread
`Evaluator`; turtle commands stream to the UI over a channel.

## Running

```bash
cargo run -p rusche-turtle
cargo run -p rusche-turtle -- crates/rusche-turtle/examples/star.rsc
```

| Control | Role |
| --- | --- |
| Run | Evaluate the editor (fresh env each time) |
| Stop | Cancel at the next turtle / `write` / `random` call |
| Examples | Load a bundled demo |
| Help | API hints (click a form to append it) |
| Clear / Speed | Under the canvas (`instant` skips walk animation) |

## Script API

Origin at canvas center, **y up**, heading **0° = east**, positive turns
counter-clockwise. Starts at `(0, 0)` facing north (`90°`) with pen down.

Core language: [language reference](language-reference.md). Host adds:

### Motion

| Form | Effect |
| --- | --- |
| `(forward dist)` / `(fd dist)` | Move forward; draw if pen down |
| `(back dist)` / `(bk dist)` | Move backward |
| `(left deg)` / `(lt deg)` | Turn left |
| `(right deg)` / `(rt deg)` | Turn right |
| `(goto x y)` | Absolute move (draws if pen down) |
| `(setheading deg)` / `(seth deg)` | Absolute heading |
| `(home)` | Origin, face north, no draw |

### Pen and canvas

| Form | Effect |
| --- | --- |
| `(penup)` / `(pu)`, `(pendown)` / `(pd)` | Pen up / down |
| `(pencolor name)` | `black` `white` `red` `green` `blue` `yellow` `orange` `purple` `cyan` `magenta` `gray` `brown` |
| `(pencolor r g b)` | RGB `0…255` |
| `(pensize w)` | Stroke width |
| `(clear)` | Clear and reset |
| `(hideturtle)` / `(showturtle)` | Sprite visibility |

### Queries and I/O

| Form | Result |
| --- | --- |
| `(position)` | `(x y)` |
| `(heading)` | Degrees |
| `(pen?)` | Pen down? |
| `(write …)` / `(writeln …)` | Output log |
| `(random)` | Float in `[0, 1)` |
| `(repeat n body…)` | Loop `body` `n` times |

```scheme
(pencolor "blue")
(repeat 36
  (repeat 4 (forward 100) (right 90))
  (right 10))
```

Demos: [`crates/rusche-turtle/examples/`](../crates/rusche-turtle/examples).

## Limitations

- **Stop** only interrupts at turtle natives, `write`/`writeln`, or `random`.
- Each **Run** starts a fresh evaluator.
- Example host (`publish = false`), like `rusche-cli`.
