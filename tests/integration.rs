use std::path::Path;

use synth::{
    BlocPolicy, DesignSpecOptions, FramePhase, OptimizeOptions, OptimizeRequest, SeekFrame,
    enforce_synthesis_gate, optimize_and_handoff, optimize_cds, parse_bloc_verdict, parse_seek_frame,
    preferred_codon, resolve_host,
};

#[test]
fn gfp_frame_pipeline_handoff() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/gfp_frame.json");
    let text = std::fs::read_to_string(path).expect("read gfp example");
    let frame: SeekFrame = parse_seek_frame(&text).expect("parse frame");
    assert_eq!(frame.phase, FramePhase::Filled);
    let request = OptimizeRequest {
        display_host: "e_coli_k12".into(),
        optimize: OptimizeOptions::from_chassis("e_coli_k12"),
        design_spec: DesignSpecOptions::default(),
    };
    let handoff = optimize_and_handoff(frame, &request).expect("optimise");
    assert_eq!(handoff.design_spec.spec_version, "0.1.0");
    assert_eq!(handoff.design_spec.construct_id, "frm_gfp");
    assert!(!handoff.design_spec.fragments.is_empty());
    assert!(handoff.frame_result.gc > 0.0 && handoff.frame_result.gc < 1.0);
}

#[test]
fn host_aliases_map_to_same_optimisation() {
    let seq = "ATGAAATTTTAA";
    let a = optimize_cds(seq, &OptimizeOptions::from_chassis("e_coli_k12")).unwrap();
    let b = optimize_cds(seq, &OptimizeOptions::from_chassis("ecoli")).unwrap();
    assert_eq!(a, b);
    assert_eq!(resolve_host("yeast"), "s_cerevisiae");
}

#[test]
fn host_codon_tables_differ() {
    assert_ne!(
        preferred_codon('L', "e_coli"),
        preferred_codon('L', "s_cerevisiae")
    );
}

#[test]
fn synthesis_gate_blocks_non_release() {
    let raw = r#"{
      "verdict_version": "0.1.0",
      "construct_id": "frm_gfp",
      "rubric": "Flag",
      "policy": "HOLD",
      "reasons": [{"code": "test", "clause": "test", "engine": "test"}],
      "engines": {},
      "receipt": {
        "spec_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        "fasta_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        "identity": "0000000000000000000000000000000000000000000000000000000000000000",
        "inhibit_version": "0",
        "engine_revs": {},
        "utc": "2026-01-01T00:00:00Z"
      }
    }"#;
    let verdict = parse_bloc_verdict(raw).unwrap();
    assert_eq!(verdict.policy, BlocPolicy::Hold);
    let err = enforce_synthesis_gate(&verdict).unwrap_err();
    assert_eq!(err.exit_code(), 10);
}
