//! synth - a design room for synthetic DNA: optimizes a coding sequence for the selected host, removes unwanted restriction digestion sites, holds GC in thermodynamically favorable range.
//!
//! Wetware pipeline (see [`AGENTS.md`](https://github.com/wetware-hq/synth/blob/main/AGENTS.md) and `README.md`):
//!
//! ```text
//! seek frame (phase: filled)  →  synth  →  optimised seek frame
//!                                           ↓
//!                                 bloc design_spec  →  bloc screen  →  RELEASE | HOLD | ESCALATE
//! ```
//!
//! synth does **not** set release policy. Hosts must route optimised sequences through
//! [`bloc screen`](https://github.com/wetware-hq/bloc) and call [`enforce_synthesis_gate`]
//! before synthesis.
//!
//! # Example
//!
//! ```
//! use synth::{
//!     DesignSpecOptions, OptimizeOptions, OptimizeRequest, optimize_and_handoff, resolve_host,
//! };
//! use synth::parse_seek_frame;
//!
//! let frame = parse_seek_frame(r#"{"id":"f","kind":"DNA","topo":"linear","seq":"ATGAAA","features":[],"want":[],"forbid":[],"phase":"filled"}"#).unwrap();
//! let request = OptimizeRequest {
//!     display_host: "e_coli_k12".into(),
//!     optimize: OptimizeOptions::for_host(resolve_host("e_coli_k12")),
//!     design_spec: DesignSpecOptions::default(),
//! };
//! let handoff = optimize_and_handoff(frame, &request).unwrap();
//! assert_eq!(handoff.design_spec.spec_version, "0.1.0");
//! ```

mod bloc;
mod codon;
mod fasta;
mod host;
mod optimize;
mod pipeline;
mod seek_frame;

pub use bloc::{
    BlocChassis, BlocPolicy, BlocVerdict, DesignSpec, DesignSpecError, DesignSpecOptions,
    Designer, FragmentAlphabet, FragmentRole, IntendedBsl, IntendedFunction, SynthesisGateError,
    design_spec_from_seek_frame, enforce_synthesis_gate, parse_bloc_verdict, resolve_bloc_chassis,
    to_design_spec_json,
};
pub use codon::{preferred_codon, synonymous_codons, translate_codon};
pub use fasta::{FastaRecord, format_fasta, optimize_fasta_records, parse_fasta};
pub use host::{default_avoid_patterns, resolve_host};
pub use optimize::{OptimizeError, OptimizeOptions, gc_content, optimize_cds};
pub use pipeline::{OptimizeRequest, PipelineError, SynthHandoff, gate_synthesis, optimize_and_handoff};
pub use seek_frame::{
    FrameError, FrameKind, FrameOptimizeResult, FramePhase, FrameTopo, SeekFrame,
    SynthOptimisedFeature, optimize_seek_frame, parse_seek_frame, to_seek_frame_json,
    validate_frame_for_optimisation,
};
