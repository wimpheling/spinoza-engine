# Spinoza Emotion Engine

A deterministic Rust library that maps a generic character **State** to **Emotions** based on rules inspired by Spinoza's *Ethics*.

## Features
- Pure Rust, no external expression parser.
- Extensible rule tables (`ACTION_RULES`, `STATE_EMOTION_RULES`).
- Simple `Character`, `State`, `Emotion` model.
- Deterministic: same input → same output.
- Includes a unit‑test demo.

## How to use
```rust
use spinoza_engine::{Character, Action, Emotion, apply_action};

let mut c = Character { /* … */ };
let a = Action { name: "receive_money".into(), params: /* … */ };
let c = apply_action(c, &a);
println!("Joy: {}", c.emotions[&Emotion::Joy]);
```

## Extending
Add new `ActionRule` or `StateEmotionRule` entries in `src/lib.rs`.  Add more `Emotion` variants as needed.
