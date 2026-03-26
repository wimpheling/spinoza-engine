//! Spinoza Emotion Engine
//!
//! Deterministic mapping from generic character state to emotions based on Spinoza's Ethics.
//!
//! The engine is completely self‑contained – no external expression parser is required.
//! Rules are expressed as Rust data structures, evaluated deterministically.
//!
//! # Overview
//! - `Character` holds an `id`, a generic `State` (key/value map) and an `EmotionalState`.
//! - `State` is a `HashMap<String, Value>` where `Value` can be bool, i64, f64 or String.
//! - `Emotion` is an enum of supported emotions.
//! - `Action` describes a mutation to `State`.
//! - Two rule tables are defined:
//!   1. `ACTION_RULES` – how an `Action` changes `State`.
//!   2. `STATE_EMOTION_RULES` – how particular `State` conditions affect emotions.
//! - The engine is deterministic: given the same initial `Character` and a sequence of
//!   `Action`s the resulting `EmotionalState` is always identical.
//!
//! The crate also provides a tiny CLI (`cargo run --example demo`) that loads the
//! ethics text, builds a sample character, applies a few actions and prints the
//! resulting emotions.

use std::collections::HashMap;

/// Simple value container for generic state attributes.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

impl Value {
    fn as_f64(&self) -> f64 {
        match self {
            Value::Bool(b) => if *b { 1.0 } else { 0.0 },
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            Value::Text(_) => 0.0,
        }
    }
}

/// Collection of generic attributes.
pub type State = HashMap<String, Value>;

/// Emotions supported by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Emotion {
    Joy,
    Sadness,
    Anger,
    Fear,
    Love,
    Peace,
    // Extend with more emotions as needed.
}

/// Mapping from Emotion -> intensity (0.0 .. 1.0).
pub type EmotionalState = HashMap<Emotion, f32>;

/// Core character representation.
#[derive(Debug, Clone)]
pub struct Character {
    pub id: String,
    pub state: State,
    pub emotions: EmotionalState,
}

/// Action definition – a name plus optional parameters.
#[derive(Debug, Clone)]
pub struct Action {
    pub name: String,
    pub params: HashMap<String, Value>,
}

/// Rule that changes a State attribute when an Action is applied.
#[derive(Debug, Clone)]
pub struct ActionRule {
    pub action: &'static str,
    /// Closure receiving the whole state and params, returning a list of (key, new Value).
    pub apply: fn(&State, &HashMap<String, Value>) -> Vec<(String, Value)>,
}

/// Rule that maps a particular State condition to an emotion delta.
#[derive(Debug, Clone)]
pub struct StateEmotionRule {
    /// The attribute name to inspect.
    pub attribute: &'static str,
    /// Predicate on the current value – returns true if the rule matches.
    pub condition: fn(&Value) -> bool,
    /// Mapping from Emotion to delta (positive or negative).
    pub affect: &'static [(Emotion, f32)],
}

/// Helper to clamp a float between 0.0 and 1.0.
fn clamp01(v: f32) -> f32 {
    if v < 0.0 { 0.0 } else if v > 1.0 { 1.0 } else { v }
}

/// Apply an Action to a Character, returning a new Character with updated state and emotions.
pub fn apply_action(mut character: Character, action: &Action) -> Character {
    // 1️⃣ Apply state mutation rules.
    for ar in ACTION_RULES.iter() {
        if ar.action == action.name {
            let changes = (ar.apply)(&character.state, &action.params);
            for (key, val) in changes {
                character.state.insert(key.to_string(), val);
            }
        }
    }

    // 2️⃣ Re‑evaluate emotions based on the updated state.
    // Reset all emotions to 0 before accumulating.
    for emo in character.emotions.keys().cloned().collect::<Vec<_>>() {
        character.emotions.insert(emo, 0.0);
    }
    for rule in STATE_EMOTION_RULES.iter() {
        if let Some(val) = character.state.get(rule.attribute) {
            if (rule.condition)(val) {
                for &(emo, delta) in rule.affect.iter() {
                    let cur = character.emotions.get(&emo).copied().unwrap_or(0.0);
                    character.emotions.insert(emo, clamp01(cur + delta));
                }
            }
        }
    }
    character
}

// ---------------------------------------------------------------------------
// ----- Rule tables --------------------------------------------------------
// ---------------------------------------------------------------------------

/// Example Action rules – you can extend them as needed.
pub static ACTION_RULES: &[ActionRule] = &[
    // "receive_money" increases the "wealth" attribute by the supplied amount.
    ActionRule {
        action: "receive_money",
        apply: |_state, params| {
            let amount = match params.get("amount") {
                Some(Value::Int(i)) => *i as f64,
                Some(Value::Float(f)) => *f,
                _ => 0.0,
            };
            vec![
                ("wealth".to_string(), Value::Float(amount))
            ]
        },
    },
    // "take_damage" reduces "health" by the given amount.
    ActionRule {
        action: "take_damage",
        apply: |_state, params| {
            let dmg = match params.get("amount") {
                Some(Value::Int(i)) => *i as f64,
                Some(Value::Float(f)) => *f,
                _ => 0.0,
            };
            // We'll subtract; the caller must ensure the attribute exists.
            vec![
                ("health_delta".to_string(), Value::Float(-dmg))
            ]
        },
    },
];

/// Example State→Emotion rules derived from Spinoza’s definitions.
/// The conditions are deliberately simple – you can enrich them later.
pub static STATE_EMOTION_RULES: &[StateEmotionRule] = &[
    // Low health → sadness & fear.
    StateEmotionRule {
        attribute: "health",
        condition: |v| v.as_f64() < 30.0,
        affect: &[(Emotion::Sadness, 0.4), (Emotion::Fear, 0.3)],
    },
    // High wealth → joy.
    StateEmotionRule {
        attribute: "wealth",
        condition: |v| v.as_f64() > 1000.0,
        affect: &[(Emotion::Joy, 0.5)],
    },
    // Positive reputation → love.
    StateEmotionRule {
        attribute: "reputation",
        condition: |v| v.as_f64() > 70.0,
        affect: &[(Emotion::Love, 0.4)],
    },
];

// ---------------------------------------------------------------------------
// ----- Demo (optional) ----------------------------------------------------
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_character() -> Character {
        let mut state = State::new();
        state.insert("health".to_string(), Value::Int(100));
        state.insert("wealth".to_string(), Value::Int(0));
        state.insert("reputation".to_string(), Value::Int(50));
        let mut emotions = EmotionalState::new();
        for e in &[Emotion::Joy, Emotion::Sadness, Emotion::Anger, Emotion::Fear, Emotion::Love, Emotion::Peace] {
            emotions.insert(*e, 0.0);
        }
        Character { id: "demo".into(), state, emotions }
    }

    #[test]
    fn basic_flow() {
        let mut chara = make_character();
        // Apply receive_money action.
        let action = Action { name: "receive_money".into(), params: {
            let mut p = HashMap::new();
            p.insert("amount".into(), Value::Int(1500));
            p
        } };
        chara = apply_action(chara, &action);
        // Wealth now 1500, should trigger Joy.
        assert_eq!(chara.emotions.get(&Emotion::Joy).copied().unwrap_or(0.0), 0.5);
        // Health unchanged, no sadness/fear.
        assert_eq!(chara.emotions.get(&Emotion::Sadness).copied().unwrap_or(0.0), 0.0);
    }
}
