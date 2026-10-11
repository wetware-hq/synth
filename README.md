# synth

Codon optimisation for DNA synthesis and host expression. Ported from [heymcee/synth](https://github.com/heymcee/synth) into a portable Rust library with a thin CLI, integrated with [Seek](https://github.com/wetware-hq/seek) frames and [bloc](https://github.com/wetware-hq/bloc) screening.

## Wetware pipeline

Contract (normative detail in [`AGENTS.md`](AGENTS.md), sourced from heymcee/synth):

```text
seek frame (phase: filled)  →  synth  →  optimised seek frame
                                          ↓
                                bloc design_spec  →  bloc screen  →  RELEASE | HOLD | ESCALATE
```

- **Input:** Seek frame JSON (`kind: DNA`, `phase: filled`) or FASTA (convenience).
- **Output:** Optimised Seek frame (same schema) plus optional bloc `design_spec` (`spec_version` `0.1.0`).
- **Gate:** Only a bloc `RELEASE` verdict clears synthesis; use `enforce_synthesis_gate` or `--verdict` after `bloc screen`.

synth does **not** run `bloc screen` or set release policy.

| Item | Value |
|------|--------|
| Crate | `synth` (library + `synth` binary) |
| Seek schema | [frame.schema.json](https://github.com/wetware-hq/seek/blob/main/schema/frame.schema.json) |
| bloc schema | [design_spec.schema.json](https://github.com/wetware-hq/bloc/blob/main/schemas/design_spec.schema.json) |
| License | MIT |

### Host / chassis mapping

| Input (CLI / chassis) | Codon table (`resolve_host`) | bloc `chassis` (`resolve_bloc_chassis`) |
|-----------------------|------------------------------|----------------------------------------|
| `e_coli_k12`, `ecoli` | `e_coli` | `e_coli_k12` |
| `e_coli_b` | `e_coli` | `e_coli_b` |
| `yeast` | `s_cerevisiae` | `yeast` |
| `human` | `h_sapiens` | `other` (not a bloc chassis enum value) |

## CLI

```bash
cargo build --release
cargo run --release -- examples/gfp_frame.json --host e_coli_k12 -o optimised.json
cargo run --release -- examples/gfp_frame.json --design-spec gfp_spec.json
cargo run --release -- examples/gfp_frame.json --verdict verdict.json   # after bloc screen
cargo run --release -- input.fasta --host yeast -o optimized.fasta
```

Options: `--host`, `--avoid`, `--gc-min`, `--gc-max`, `--design-spec`, `--verdict`, `--designer-human`, `--designer-agent`.

## Library

```toml
[dependencies]
synth = { version = "0.1", default-features = false }
```

```rust
use synth::{
    DesignSpecOptions, OptimizeOptions, OptimizeRequest, enforce_synthesis_gate,
    optimize_and_handoff, parse_bloc_verdict, parse_seek_frame, resolve_host,
};

let frame = parse_seek_frame(&std::fs::read_to_string("frame.json")?)?;
let handoff = optimize_and_handoff(
    frame,
    &OptimizeRequest {
        display_host: "e_coli_k12".into(),
        optimize: OptimizeOptions::for_host(resolve_host("e_coli_k12")),
        design_spec: DesignSpecOptions::default(),
    },
)?;
std::fs::write("optimised.json", synth::to_seek_frame_json(&handoff.frame_result.frame)?)?;
std::fs::write("spec.json", synth::to_design_spec_json(&handoff.design_spec)?)?;

// After: bloc screen spec.json → verdict.json
let verdict = parse_bloc_verdict(&std::fs::read_to_string("verdict.json")?)?;
enforce_synthesis_gate(&verdict)?; // Err if HOLD or ESCALATE
```

### Public API (summary)

| Area | Types / functions |
|------|-------------------|
| Seek | `SeekFrame`, `FrameKind`, `FramePhase`, `parse_seek_frame`, `optimize_seek_frame`, `validate_frame_for_optimisation` |
| Optimise | `optimize_cds`, `OptimizeOptions`, `OptimizeError`, `gc_content` |
| Pipeline | `OptimizeRequest`, `optimize_and_handoff`, `SynthHandoff` |
| bloc | `DesignSpec`, `design_spec_from_seek_frame`, `BlocVerdict`, `enforce_synthesis_gate`, `parse_bloc_verdict` |
| Host | `resolve_host`, `resolve_bloc_chassis`, `default_avoid_patterns` |
| FASTA | `parse_fasta`, `format_fasta`, `optimize_fasta_records` |

Run `cargo doc --open` for rustdoc on each item.

## Layout

```
src/
  lib.rs
  seek_frame.rs   Seek frame in/out
  pipeline.rs     optimise + design_spec handoff
  bloc/           design_spec + verdict gate types
  optimize.rs     CAI-style optimisation
  codon.rs        genetic code tables
  host.rs         chassis aliases
  fasta.rs        FASTA convenience path
  main.rs         CLI (feature `cli`)
AGENTS.md         host/agent contract (from heymcee/synth)
schema/           optimise_request JSON schema
examples/         Seek frame fixtures
```

## Development

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## Attribution

Rust implementation originally developed in [heymcee/synth](https://github.com/heymcee/synth) (MIT). Continued as [wetware-hq/synth](https://github.com/wetware-hq/synth).
