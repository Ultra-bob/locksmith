use std::fmt;

use serde::{Deserialize, Serialize};

/// A stable identifier for a decoder implementation.
pub type DecoderId = String;

/// A single transformation step that produced an output.
///
/// This is recorded in a chain so we can:
/// - present user-friendly descriptions
/// - enforce constraint policies (no consecutive same op, etc.)
/// - analyze/compose multi-step transforms
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Step {
    /// Identifier of the decoder/operation, e.g., "caesar", "base64".
    pub op_id: DecoderId,
    /// Human-readable description, e.g., "Caesar shift 13".
    pub desc: String,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.op_id, self.desc)
    }
}

/// A decoder output paired with the step that produced it.
#[derive(Clone, Debug)]
pub struct TransformResult {
    pub output: String,
    pub step: Step,
}

/// Policy that controls whether a decoder may follow a prior sequence of steps.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    /// Forbid applying the exact same decoder right after itself.
    pub no_consecutive_same_op: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            no_consecutive_same_op: true,
        }
    }
}

/// A single transformation primitive that can produce zero or more outputs from an input.
///
/// Implementors should:
/// - return a stable `id()`
/// - optionally assign a `group()`
/// - optionally customize `policy()`
/// - implement `apply()` to return all possible outputs
pub trait Decoder: Send + Sync {
    /// Stable identifier for this decoder/operation (e.g., "caesar", "base64").
    fn id(&self) -> DecoderId;

    /// Constraint policy that controls when this decoder may follow a history of steps.
    fn policy(&self) -> Policy {
        Policy::default()
    }

    /// Apply this decoder to an input string, returning zero or more possible outputs.
    ///
    /// Note: Prefer returning a single `TransformResult` if the operation is deterministic,
    /// and multiple results if it is brute-force (like trying all Caesar shifts).
    fn apply(&self, input: &str) -> Vec<TransformResult>;

    /// Centralized constraint hook so the engine can ask whether this decoder may follow
    /// a given history. Override if you need custom logic beyond `policy()`.
    fn can_follow(&self, history: &[Step]) -> bool {
        let policy = self.policy();

        if let Some(last) = history.last() {
            if policy.no_consecutive_same_op && last.op_id == self.id() {
                return false;
            }
        }

        true
    }
}

/// Engine that manages and applies a set of decoders.
pub struct DecoderEngine {
    decoders: Vec<Box<dyn Decoder>>,
}

impl DecoderEngine {
    /// Create a new, empty decoder engine.
    pub fn new() -> Self {
        Self {
            decoders: Vec::new(),
        }
    }

    /// Register a decoder.
    pub fn register<D>(&mut self, decoder: D)
    where
        D: Decoder + 'static,
    {
        self.decoders.push(Box::new(decoder));
    }

    /// Register a pre-boxed decoder (e.g., when using trait objects).

    /// Immutable access to the internal decoder list.

    /// Apply all decoders that pass `can_follow(history)` to `input`.
    pub fn expand(&self, input: &str, history: &[Step]) -> Vec<TransformResult> {
        self.decoders
            .iter()
            .filter(|d| d.can_follow(history))
            .flat_map(|d| d.apply(input))
            .collect()
    }
}
