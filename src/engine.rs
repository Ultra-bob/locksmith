use std::fmt;

/// A stable identifier for a decoder implementation.
pub type DecoderId = &'static str;

/// A single transformation step that produced an output.
///
/// This is recorded in a chain so we can:
/// - present user-friendly descriptions
/// - enforce constraint policies (no consecutive same op, etc.)
/// - analyze/compose multi-step transforms
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Step {
    /// Identifier of the decoder/operation, e.g., "caesar", "base64".
    pub op_id: DecoderId,
    /// Human-readable description, e.g., "Caesar shift 13".
    pub desc: String,
    /// Group/category for constraint grouping, e.g., "shift", "radix64".
    pub group: &'static str,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.group.is_empty() {
            write!(f, "{}: {}", self.op_id, self.desc)
        } else {
            write!(f, "{}[{}]: {}", self.op_id, self.group, self.desc)
        }
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
    /// Forbid applying another decoder from the same group within the last N steps.
    /// Example:
    /// - 0 disables this rule
    /// - 1 forbids same-group immediately after
    /// - 2 forbids same-group if any of the last two steps were of that group, etc.
    pub no_group_repeat_within: usize,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            no_consecutive_same_op: true,
            no_group_repeat_within: 1,
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

    /// Group name to classify families of operations (e.g., "shift", "radix64").
    /// Used by default policies to prevent near-duplicate exploration.
    fn group(&self) -> &'static str {
        "generic"
    }

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

            if policy.no_group_repeat_within > 0 {
                let n = policy.no_group_repeat_within.min(history.len());
                if history[history.len() - n..]
                    .iter()
                    .any(|s| s.group == self.group())
                {
                    return false;
                }
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

    /// Immutable access to the internal decoder list.
    pub fn decoders(&self) -> &[Box<dyn Decoder>] {
        &self.decoders
    }

    /// Apply all decoders that pass `can_follow(history)` to `input`.
    pub fn expand(&self, input: &str, history: &[Step]) -> Vec<TransformResult> {
        self.decoders
            .iter()
            .filter(|d| d.can_follow(history))
            .flat_map(|d| d.apply(input))
            .collect()
    }
}
