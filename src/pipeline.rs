//! Seek → synth → bloc pipeline helpers (see `AGENTS.md`).

use crate::bloc::{
    DesignSpec, DesignSpecOptions, enforce_synthesis_gate, design_spec_from_seek_frame,
};
use crate::bloc::{BlocVerdict, SynthesisGateError};
use crate::optimize::OptimizeOptions;
use crate::seek_frame::{FrameOptimizeResult, SeekFrame, optimize_seek_frame, validate_frame_for_optimisation};
use thiserror::Error;

/// Request to optimise one filled Seek frame and prepare bloc handoff.
#[derive(Debug, Clone)]
pub struct OptimizeRequest {
    /// User-facing host / chassis (recorded on features; maps to bloc `chassis`).
    pub display_host: String,
    pub optimize: OptimizeOptions,
    pub design_spec: DesignSpecOptions,
}

/// Optimised frame plus bloc design specification for `bloc screen`.
#[derive(Debug, Clone)]
pub struct SynthHandoff {
    pub frame_result: FrameOptimizeResult,
    pub design_spec: DesignSpec,
}

/// Errors across optimise + handoff.
#[derive(Debug, Error)]
pub enum PipelineError {
    #[error(transparent)]
    Frame(#[from] crate::seek_frame::FrameError),
    #[error(transparent)]
    DesignSpec(#[from] crate::bloc::DesignSpecError),
}

/// Optimise a filled DNA Seek frame and build a bloc design specification.
///
/// Does **not** run `bloc screen` or set release policy. Downstream hosts must screen the
/// returned [`DesignSpec`] and enforce [`enforce_synthesis_gate`] before synthesis.
pub fn optimize_and_handoff(
    frame: SeekFrame,
    request: &OptimizeRequest,
) -> Result<SynthHandoff, PipelineError> {
    validate_frame_for_optimisation(&frame)?;
    let frame_result = optimize_seek_frame(frame, &request.display_host, &request.optimize)?;
    let design_spec =
        design_spec_from_seek_frame(&frame_result.frame, &request.design_spec)?;
    Ok(SynthHandoff {
        frame_result,
        design_spec,
    })
}

/// After `bloc screen`, ensure the synthesis path may proceed.
pub fn gate_synthesis(verdict: &BlocVerdict) -> Result<(), SynthesisGateError> {
    enforce_synthesis_gate(verdict)
}
