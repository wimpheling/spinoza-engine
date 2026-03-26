# Project Methodology – Deterministic Spinoza Emotion Engine

## Overview

The engine maps **concrete world events** → **Spinoza‑state flags** → **emotions** in a fully
deterministic fashion.  All transformations are expressed as pure Rust data
structures (`enum`s and static slices).  The pipeline consists of three layers:

1. **WorldState** – concrete attributes such as `money`, `fame`, `salary`, …
2. **SpinozaState** – philosophical flags (`Good`, `Free`, `Necessary`, …).  Each
   flag stores a **coefficient** (`f32`).  A value of `0.0` means the flag is
   inactive; any positive value indicates activation and also serves as the
   scaling factor for the downstream emotion delta.
3. **EmotionalState** – intensities (`0.0 .. 1.0`) for the `Emotion` enum.

All code is **type‑safe**: keys are enums (`WorldAttr`, `SpinozaAttr`), actions
carry **action‑specific structs** (`EarnMoneyParams`, …), and every public
function returns `Result<…, EngineError>`.

## Design Decisions

| Decision | Implementation |
|----------|----------------|
| Action‑specific parameters | Separate struct per action (e.g. `EarnMoneyParams`). |
| Separate state maps | `WorldState = HashMap<WorldAttr, Value>` and `SpinozaState = HashMap<SpinozaAttr, f32>`. |
| Coefficient handling | No dedicated `*_Coef` variants – the coefficient is stored **inside** the flag value (`SpinozaAttr → f32`). |
| Source provenance | Every `StateEmotionRule` carries a `source` string comment with the line number and definition from Spinoza’s *Ethics*. |
| Error handling | All public APIs return `Result<…, EngineError>`; the engine never panics on invalid input. |

## Core Types (simplified)

```rust
pub enum WorldAttr { Money, Fame, Salary, Position }
pub enum SpinozaAttr { Good, Bad, Free, Necessary, Eternity, Affectiones }
pub enum Emotion { Joy, Sadness, Anger, Fear, Love, Peace, /* … */ }

pub type WorldState   = HashMap<WorldAttr,   Value>;   // Value = Bool/Int/Float/Text
pub type SpinozaState = HashMap<SpinozaAttr, f32>;     // flag → coefficient
pub type EmotionalState = HashMap<Emotion, f32>;
```

## Action Representation

```rust
pub enum Action {
    EarnMoney(EarnMoneyParams),
    LoseFame { amount: f64 },
    BecomeFree(BecomeFreeParams),
    // … other actions
}

impl Action {
    pub fn kind(&self) -> ActionKind { /* returns the enum discriminant */ }
}
```

## Rule Tables

### World → Spinoza (`ActionRule`)

```rust
pub struct ActionRule {
    pub kind: ActionKind,
    pub apply: fn(&WorldState, &SpinozaState, &Action)
               -> Result<(Vec<(WorldAttr, Value)>, Vec<(SpinozaAttr, f32)>), EngineError>,
}
```

*The closure updates both maps.  When a threshold is crossed it writes the flag
with the supplied coefficient (default 1.0).*

### Spinoza → Emotion (`StateEmotionRule`)

```rust
pub struct StateEmotionRule {
    pub attribute: SpinozaAttr,
    pub condition: fn(f32) -> bool,               // usually `|c| c > 0.0`
    pub affect:    &'static [(Emotion, f32)],      // base delta
    pub source:    &'static str,                  // citation from the text
}
```

During evaluation the stored coefficient is multiplied by the base delta.

## Engine Core (`apply_action`)

1. Look up the matching `ActionRule` and run its `apply` closure.  The returned
   vectors are merged into `character.world` and `character.spinoza`.
2. Reset all emotions to `0.0`.
3. Iterate over `STATE_EMOTION_RULES`; for each rule whose flag coefficient is
   positive, add `base_delta * coefficient` to the corresponding emotion (clamped
   to `0.0 .. 1.0`).

All steps are pure functions, so the same input always yields the same output.

## Development Workflow (used in this repository)

* **Keyword‑by‑keyword implementation** – each entry from `notes/keyword_list.txt`
  is turned into a concrete enum variant (if needed) and a placeholder rule.
* After every modification we run `cargo check` to guarantee the crate still
  compiles.  If a compilation error occurs we fix it immediately before committing.
* Each successful compile results in a commit on the `impl-keywords` branch and
  a push to the remote, so a PR can be opened for review at any time.

---

This file captures the entire methodology that guides the implementation of the
deterministic emotion engine.
