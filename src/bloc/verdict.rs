//! bloc screen verdict (`schemas/verdict.schema.json`).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// bloc screening policy (`RELEASE`, `HOLD`, `ESCALATE`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BlocPolicy {
    Release,
    Hold,
    Escalate,
}

impl BlocPolicy {
    /// Process exit code used by the `bloc` CLI for this policy.
    pub fn exit_code(self) -> i32 {
        match self {
            BlocPolicy::Release => 0,
            BlocPolicy::Hold => 10,
            BlocPolicy::Escalate => 20,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            BlocPolicy::Release => "RELEASE",
            BlocPolicy::Hold => "HOLD",
            BlocPolicy::Escalate => "ESCALATE",
        }
    }
}

impl std::fmt::Display for BlocPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlocRubric {
    #[serde(rename = "No Flag")]
    NoFlag,
    Flag,
    Undefined,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlocReason {
    pub code: String,
    pub clause: String,
    pub engine: String,
    #[serde(default)]
    pub fragment_id: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlocReceipt {
    pub spec_sha256: String,
    pub fasta_sha256: String,
    pub identity: String,
    pub inhibit_version: String,
    pub engine_revs: serde_json::Value,
    pub utc: String,
}

/// Typed verdict from `bloc screen`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlocVerdict {
    pub verdict_version: String,
    pub construct_id: String,
    pub rubric: BlocRubric,
    pub policy: BlocPolicy,
    pub reasons: Vec<BlocReason>,
    pub engines: serde_json::Value,
    pub receipt: BlocReceipt,
}

/// Parse a bloc verdict JSON document.
pub fn parse_bloc_verdict(json: &str) -> Result<BlocVerdict, serde_json::Error> {
    serde_json::from_str(json)
}

/// Error when a verdict blocks the synthesis path (per source `AGENTS.md`).
#[derive(Debug, Error, PartialEq, Eq)]
#[error("synthesis blocked: bloc policy {policy} (construct {construct_id})")]
pub struct SynthesisGateError {
    pub policy: BlocPolicy,
    pub construct_id: String,
}

impl SynthesisGateError {
    pub fn exit_code(&self) -> i32 {
        self.policy.exit_code()
    }
}

/// Enforce the bloc gate: only `RELEASE` may proceed toward synthesis.
///
/// synth does **not** run `bloc screen`; hosts supply a verdict from bloc and call this
/// before ordering, assembly, or transformation.
pub fn enforce_synthesis_gate(verdict: &BlocVerdict) -> Result<(), SynthesisGateError> {
    match verdict.policy {
        BlocPolicy::Release => Ok(()),
        policy => Err(SynthesisGateError {
            policy,
            construct_id: verdict.construct_id.clone(),
        }),
    }
}

impl BlocVerdict {
    /// Whether this verdict clears the synthesis path.
    pub fn allows_synthesis(&self) -> bool {
        self.policy == BlocPolicy::Release
    }
}
