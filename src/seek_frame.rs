//! [Seek](https://github.com/wetware-hq/seek) frame types ([`frame.schema.json`](https://github.com/wetware-hq/seek/blob/main/schema/frame.schema.json)).

use crate::optimize::{OptimizeError, OptimizeOptions, gc_content, optimize_cds};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Seek frame polymer kind (`DNA`, `RNA`, `AA`, `XNA`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum FrameKind {
    Dna,
    Rna,
    Aa,
    Xna,
}

impl std::fmt::Display for FrameKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameKind::Dna => f.write_str("DNA"),
            FrameKind::Rna => f.write_str("RNA"),
            FrameKind::Aa => f.write_str("AA"),
            FrameKind::Xna => f.write_str("XNA"),
        }
    }
}

/// Seek frame topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FrameTopo {
    Linear,
    Circular,
}

/// Seek frame lifecycle phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FramePhase {
    Spec,
    Filled,
    Kill,
    Tube,
}

impl std::fmt::Display for FramePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FramePhase::Spec => f.write_str("spec"),
            FramePhase::Filled => f.write_str("filled"),
            FramePhase::Kill => f.write_str("kill"),
            FramePhase::Tube => f.write_str("tube"),
        }
    }
}

/// Seek frame document (eight required root fields).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeekFrame {
    pub id: String,
    pub kind: FrameKind,
    pub topo: FrameTopo,
    pub seq: String,
    pub features: Vec<serde_json::Value>,
    pub want: Vec<String>,
    pub forbid: Vec<String>,
    pub phase: FramePhase,
}

/// Annotation appended by synth on successful optimisation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SynthOptimisedFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub host: String,
    pub species: String,
    pub gc: f64,
}

/// Result of optimising a Seek frame.
#[derive(Debug, Clone)]
pub struct FrameOptimizeResult {
    pub frame: SeekFrame,
    /// GC fraction of the optimised sequence.
    pub gc: f64,
    /// Resolved host key used for codon tables.
    pub species: String,
}

/// Errors when validating or optimising a Seek frame.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FrameError {
    #[error("synth optimises DNA frames only (got {0}); use seek.filler for AA→DNA first")]
    NotDna(FrameKind),
    #[error("frame phase must be `filled` before codon optimisation (got {0})")]
    WrongPhase(FramePhase),
    #[error("empty seq; keep phase `spec` until seek.filler or another fill step runs")]
    EmptySequence,
    #[error(transparent)]
    Optimize(#[from] OptimizeError),
}

/// Validate that a frame meets preconditions for codon optimisation.
pub fn validate_frame_for_optimisation(frame: &SeekFrame) -> Result<(), FrameError> {
    if frame.kind != FrameKind::Dna {
        return Err(FrameError::NotDna(frame.kind));
    }
    if frame.phase != FramePhase::Filled {
        return Err(FrameError::WrongPhase(frame.phase));
    }
    if frame.seq.is_empty() {
        return Err(FrameError::EmptySequence);
    }
    Ok(())
}

/// Optimise the coding sequence inside a filled DNA Seek frame.
///
/// `display_host` is recorded on the `synth_optimised` feature (e.g. CLI `--host` / bloc chassis).
/// `opts.host` should be the resolved codon-table key ([`crate::resolve_host`]).
pub fn optimize_seek_frame(
    mut frame: SeekFrame,
    display_host: &str,
    opts: &OptimizeOptions,
) -> Result<FrameOptimizeResult, FrameError> {
    validate_frame_for_optimisation(&frame)?;
    let opt = optimize_cds(&frame.seq, opts)?;
    let gc = gc_content(&opt);
    frame.seq = opt;
    frame.phase = FramePhase::Filled;
    let feature = SynthOptimisedFeature {
        feature_type: "synth_optimised".into(),
        host: display_host.to_string(),
        species: opts.host.clone(),
        gc,
    };
    frame
        .features
        .push(serde_json::to_value(feature).expect("serialise synth_optimised feature"));
    Ok(FrameOptimizeResult {
        frame,
        gc,
        species: opts.host.clone(),
    })
}

/// Parse a Seek frame from JSON (validates unknown root keys via [`SeekFrame`]).
pub fn parse_seek_frame(json: &str) -> Result<SeekFrame, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialise a Seek frame to pretty JSON with trailing newline.
pub fn to_seek_frame_json(frame: &SeekFrame) -> Result<String, serde_json::Error> {
    Ok(serde_json::to_string_pretty(frame)? + "\n")
}
