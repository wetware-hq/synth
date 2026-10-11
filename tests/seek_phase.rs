use synth::{
    FrameError, FrameKind, FramePhase, OptimizeOptions, SeekFrame, FrameTopo, optimize_seek_frame,
};

fn minimal_dna_frame(phase: FramePhase) -> SeekFrame {
    SeekFrame {
        id: "frm_test".into(),
        kind: FrameKind::Dna,
        topo: FrameTopo::Linear,
        seq: "ATGAAATAA".into(),
        features: vec![],
        want: vec![],
        forbid: vec![],
        phase,
    }
}

#[test]
fn rejects_non_filled_phase() {
    let frame = minimal_dna_frame(FramePhase::Spec);
    let err = optimize_seek_frame(frame, "e_coli_k12", &OptimizeOptions::default()).unwrap_err();
    assert!(matches!(err, FrameError::WrongPhase(FramePhase::Spec)));
}

#[test]
fn rejects_aa_frames() {
    let frame = SeekFrame {
        id: "frm_aa".into(),
        kind: FrameKind::Aa,
        topo: FrameTopo::Linear,
        seq: "MK".into(),
        features: vec![],
        want: vec![],
        forbid: vec![],
        phase: FramePhase::Filled,
    };
    let err = optimize_seek_frame(frame, "e_coli_k12", &OptimizeOptions::default()).unwrap_err();
    assert!(matches!(err, FrameError::NotDna(FrameKind::Aa)));
}
