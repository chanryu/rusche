# `rusche-turtle`

[`rusche-turtle`](../crates/rusche-turtle) is an example host, not part of the core
`rusche` crate: a desktop turtle-graphics window with a script editor, canvas, and
output log. Scripts are evaluated by an embedded Rusche `Evaluator` on a worker
thread; turtle commands stream to the UI over a channel.

## Running

```bash
cargo run -p rusche-turtle
cargo run -p rusche-turtle -- crates/rusche-turtle/examples/star.rsc
```

The optional path is loaded into the editor; press **Run** to evaluate.

| Control | Role |
| --- | --- |
| Run | Evaluate the editor contents (fresh environment each time) |
| Stop | Cancel at the next turtle / `write` / `random` call |
| Examples | Load a bundled demo script |
| Help | Toggle the API hint panel (click a form to append it to the script) |
| Clear / Speed | Under the canvas — clear drawing, or set walk speed (`instant` skips animation) |

## What it loads

| Layer | Contents |
| --- | --- |
| Core built-ins + prelude | [Language reference](language-reference.md) |
| Turtle natives | Motion, pen, queries (table below) |
| `write` / `writeln` | Append text to the output log |
| `random` | Uniform float in `[0, 1)` |
| `(repeat n body…)` | Prelude macro (core has no `do`) |

## Script API

Coordinates: origin at the canvas center, **y up**, heading **0° = east**, positive
turns are counter-clockwise. The turtle starts at `(0, 0)` facing north (`90°`)
with the pen down.

### Motion

| Form | Effect |
| --- | --- |
| `(forward dist)` / `(fd dist)` | Move forward; draw if pen is down |
| `(back dist)` / `(bk dist)` | Move backward |
| `(left deg)` / `(lt deg)` | Turn left (counter-clockwise) |
| `(right deg)` / `(rt deg)` | Turn right (clockwise) |
| `(goto x y)` | Move to absolute position (draws if pen down) |
| `(setheading deg)` / `(seth deg)` | Set absolute heading |
| `(home)` | Return to origin facing north without drawing |

### Pen and canvas

| Form | Effect |
| --- | --- |
| `(penup)` / `(pu)` | Lift pen |
| `(pendown)` / `(pd)` | Lower pen |
| `(pencolor name)` | Named color: `black`, `white`, `red`, `green`, `blue`, `yellow`, `orange`, `purple`, `cyan`, `magenta`, `gray`, `brown` |
| `(pencolor r g b)` | RGB channels in `0…255` |
| `(pensize w)` | Stroke width |
| `(clear)` | Clear canvas and reset turtle |
| `(hideturtle)` / `(showturtle)` | Hide or show the turtle sprite |

### Queries and I/O

| Form | Result |
| --- | --- |
| `(position)` | `(x y)` |
| `(heading)` | Degrees |
| `(pen?)` | `true` if pen is down |
| `(write …)` / `(writeln …)` | Log pane (strings raw; other values printed) |
| `(random)` | Float in `[0, 1)` |
| `(repeat n body…)` | Run `body` `n` times |

Example:

```scheme
(pencolor "blue")
(repeat 36
  (repeat 4 (forward 100) (right 90))
  (right 10))
```

Bundled demos live under [`crates/rusche-turtle/examples/`](../crates/rusche-turtle/examples).

## Limitations

- **Stop** only interrupts at turtle natives, `write`/`writeln`, or `random`. A
  pure CPU loop with no such calls is not interruptible.
- Each **Run** starts a fresh evaluator; definitions do not persist across runs.
- The app is an example host (`publish = false`), like `rusche-cli`.
