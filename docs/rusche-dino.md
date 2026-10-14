# `rusche-dino`

[`rusche-dino`](../crates/rusche-dino) is an example host: a Chrome-dino-style endless
runner. Gameplay is written in Rusche; the Rust side only pumps the frame and paints
the scene list that `tick` returns. There are no game natives and no foreign objects.

## Running

```bash
cargo run -p rusche-dino
```

| Control | Action |
| --- | --- |
| Space, ↑, or click | Jump (also starts the run and restarts after game over) |
| ↓ | Duck on the ground; fast-fall in the air |

Edit the bundled scripts under [`crates/rusche-dino/src/rsc/`](../crates/rusche-dino/src/rsc/)
and rebuild. The host evaluates them in a fixed order with `eval_str_named`.

## Script layout

| File | Role |
| --- | --- |
| `state.rsc` | Canvas / sprite sizes, colors, LCG `rand`, globals, `reset` |
| `player.rsc` | Run, jump, duck, death pose — `step-player` |
| `field.rsc` | Speed, clouds, obstacles, collision, score — `step-field` |
| `scene.rsc` | Scene command list — `draw-scene` |
| `game.rsc` | `(tick dt jump? duck?)` |

Sprites are white-alpha PNGs under
[`crates/rusche-dino/src/sprite_png/`](../crates/rusche-dino/src/sprite_png/),
derived from [Wikimedia Commons — Dinosaur Game](https://commons.wikimedia.org/wiki/Category:Dinosaur_Game)
(see [`assets/README.md`](../crates/rusche-dino/assets/README.md)). Sizes in
`state.rsc` must match those PNGs.

## Host contract

Each frame the app calls:

```scheme
(tick dt jump? duck?)
```

- `dt` — seconds since the last frame (clamped by the host)
- `jump?` / `duck?` — keys held this frame (booleans)

`tick` updates globals with `set!` and returns a list of draw commands in a 600×150
virtual canvas (origin top-left, y down). The host scales by 2× for the window.

| Command | Meaning |
| --- | --- |
| `(clear r g b)` | Fill background (`0…255`) |
| `(rect x y w h r g b)` | Filled rectangle |
| `(sprite name x y r g b)` | 1-bit bitmap tinted with that color; `name` is a symbol |

Night mode is only different numbers in those commands. The host does not know about
score, obstacles, or day/night.

## Limitations

- No sound, no persistent high score, no script editor.
- Sprite PNGs come from Wikimedia Commons (Chromium offline assets); see `assets/README.md`.
