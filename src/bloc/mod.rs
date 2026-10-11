//! Wetware [bloc](https://github.com/wetware-hq/bloc) handoff types (design_spec `0.1.0`, verdict gate).

mod design_spec;
mod verdict;

pub use design_spec::{
    BlocChassis, DesignSpec, DesignSpecError, DesignSpecOptions, Designer, FragmentAlphabet,
    FragmentRole, IntendedBsl, IntendedFunction, design_spec_from_seek_frame, resolve_bloc_chassis,
    to_design_spec_json,
};
pub use verdict::{
    BlocPolicy, BlocVerdict, SynthesisGateError, enforce_synthesis_gate, parse_bloc_verdict,
};
