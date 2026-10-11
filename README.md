# synth

synth - a design room for synthetic DNA: optimizes a coding sequence for the selected host, removes unwanted restriction digestion sites, holds GC in thermodynamically favorable range.

synth is a program that prepares a DNA coding sequence for gene synthesis or related work. It substitutes synonymous codons that are frequent in a selected host organism while preserving the encoded protein under NCBI genetic code table 1. It excludes configured DNA substrings that correspond to Type IIS restriction sites used in assembly. It applies a best-effort adjustment so the GC fraction may move toward a configured band; the default band is 0.30 to 0.70 on a 0.0–1.0 scale (`gc_min` and `gc_max`, defaulting to `0.3` and `0.7`). The adjustment is a single forward pass over codons; if no pass satisfies the band without violating substring rules, the program still returns the codon-optimised sequence and that sequence may remain outside the band. The primary input is a filled Seek frame: a JSON object with root fields `id`, `kind`, `topo`, `seq`, `features`, `want`, `forbid`, and `phase`, with unknown root keys rejected at parse (`parse_seek_frame`, `SeekFrame`). Codon optimisation requires `kind` DNA, `phase` filled, non-empty `seq`, and length of `seq` divisible by three after normalisation (whitespace removed, letters uppercased, `U` mapped to `T`). Kinds RNA, AA, and XNA are valid in Seek but yield `FrameError::NotDna` if optimisation is requested. The CLI treats input as a Seek frame when the path ends in `.json` or `.seek` or when the file body begins with `{`; otherwise it reads FASTA as a secondary, convenience format with no `design_spec` emission on that path.

The host argument (`--host`, default `e_coli_k12`, or `OptimizeRequest.display_host` in the library) selects the codon preference table via `resolve_host` and is recorded on the appended `synth_optimised` feature. The same string maps to bloc `chassis` in an emitted design specification through `resolve_bloc_chassis` as follows.

| Host input | `resolve_host` | `resolve_bloc_chassis` |
|------------|----------------|-------------------------|
| `e_coli_k12`, `ecoli`, `e_coli` | `e_coli` | `e_coli_k12` |
| `e_coli_b` | `e_coli` | `e_coli_b` |
| `yeast`, `s_cerevisiae` | `s_cerevisiae` | `yeast` |
| `cell_free` | `e_coli` | `cell_free` |
| `other_bsl1` | `e_coli` | `other_bsl1` |
| `none` | `e_coli` | `none` |
| `human`, `h_sapiens` | `h_sapiens` | `other` |
| `other` | `other` | `other` |
| any other string | lowercased; unknown keys fall back to the E. coli codon table in `codon.rs` | `other` |

Unless overridden by repeated `--avoid` or `OptimizeOptions.avoid`, forbidden substrings are `GGTCTC`, `GAGACC`, `CGTCTC`, and `GAGACG` (`default_avoid_patterns`), matched as substrings anywhere in the growing sequence during codon choice.

The main output for frame input is an optimised Seek frame of the same shape: `seq` replaced, `phase` remaining `filled`, and one added feature object with `type` `synth_optimised`, fields `host` (display host), `species` (resolved codon key), and `gc` (post-optimisation fraction). Serialisation uses `to_seek_frame_json` (pretty JSON with trailing newline). The library entry points `validate_frame_for_optimisation`, `optimize_seek_frame`, and the combined path `optimize_and_handoff` (with `OptimizeRequest`, returning `SynthHandoff`) implement this behaviour. Optional bloc handoff produces `DesignSpec` at `spec_version` `"0.1.0"` via `design_spec_from_seek_frame` or `optimize_and_handoff`; `construct_id` equals frame `id`; fragments derive from each feature with `type` CDS (sequence slice from 1-based `start`/`end` when present) or, if none exist, a single `cds` fragment with alphabet `dna` over the full `seq`. CLI emission of `design_spec` (`--design-spec`) sets `designer.agent` to `synth`, `designer.human` to `wetware`, `intended_function` to `reporter`, `intended_bsl` to `BSL-1`, and `not_for_synthesis` to `true`, with `chassis` from `resolve_bloc_chassis(--host)`. FASTA output keeps the same format; each record id gains the suffix `_opt` (`parse_fasta`, `optimize_fasta_records`, `format_fasta`).

In the Wetware pipeline, a filled Seek frame passes through synth to an optimised frame; downstream, a bloc design specification is screened by `bloc screen`, which yields a policy of RELEASE, HOLD, or ESCALATE. synth does not run `bloc screen` and does not assign release policy. It can parse an existing verdict (`parse_bloc_verdict`, `BlocVerdict`) and block treatment of synthesis as allowed unless policy is RELEASE (`enforce_synthesis_gate`, `SynthesisGateError`; CLI `--verdict` after successful frame output, exiting with bloc-aligned codes for HOLD and ESCALATE). synth does not perform laboratory work, does not screen sequences of concern, and does not translate amino-acid designs to DNA; that step belongs to Seek tooling such as `seek.filler`. Normative detail for automated hosts appears in [`AGENTS.md`](AGENTS.md). JSON schemas are linked from [Seek frame](https://github.com/wetware-hq/seek/blob/main/schema/frame.schema.json), [bloc design_spec](https://github.com/wetware-hq/bloc/blob/main/schemas/design_spec.schema.json), [bloc verdict](https://github.com/wetware-hq/bloc/blob/main/schemas/verdict.schema.json), and [optimise request](schema/optimize_request.schema.json).

The public Rust API re-exports, among others: Seek types and I/O (`SeekFrame`, `FrameKind`, `FramePhase`, `FrameTopo`, `parse_seek_frame`, `to_seek_frame_json`); frame validation and optimisation (`validate_frame_for_optimisation`, `optimize_seek_frame`, `FrameError`); core CDS logic (`optimize_cds`, `OptimizeOptions`, `OptimizeError`, `gc_content`); host helpers (`resolve_host`, `resolve_bloc_chassis`, `default_avoid_patterns`); pipeline (`OptimizeRequest`, `optimize_and_handoff`, `SynthHandoff`, `PipelineError`); bloc types and I/O (`DesignSpec`, `DesignSpecOptions`, `design_spec_from_seek_frame`, `to_design_spec_json`, `DesignSpecError`); verdict gate (`BlocVerdict`, `BlocPolicy`, `parse_bloc_verdict`, `enforce_synthesis_gate`, `gate_synthesis`); and genetic-code helpers (`translate_codon`, `preferred_codon`, `synonymous_codons`). The crate feature `cli` (default on) builds the `synth` binary; `default-features = false` omits `clap`.

Library errors include `OptimizeError::BadLength` (length not divisible by three), `OptimizeError::UnknownCodon` (offset into `seq`), `OptimizeError::PatternConflict` (no legal synonymous codon at a position), `FrameError::NotDna`, `FrameError::WrongPhase`, `FrameError::EmptySequence`, `DesignSpecError`, and `SynthesisGateError` when policy is HOLD or ESCALATE. The CLI exits `0` on success; `1` on I/O failure, JSON parse failure, empty FASTA, FASTA optimisation failure, or verdict read/parse failure; `2` on Seek frame validation or optimisation failure; `10` when `--verdict` is set and policy is HOLD; `20` when `--verdict` is set and policy is ESCALATE. `SynthesisGateError::exit_code()` returns `10` or `20` to match.

The command-line interface accepts positional `input`, optional `-o` / `--output` (default `<stem>_opt.json` or `<stem>_opt.fasta`), `--host` (default `e_coli_k12`), repeatable `--avoid`, `--gc-min` (default `0.3`), `--gc-max` (default `0.7`), optional `--design-spec`, optional `--verdict`, `--designer-human` (default `wetware`), and `--designer-agent` (default `synth`).

Installation and use:

```bash
git clone https://github.com/wetware-hq/synth.git
cd synth
cargo build --release
./target/release/synth examples/gfp_frame.json --host e_coli_k12 -o optimised.json
./target/release/synth examples/gfp_frame.json --design-spec gfp_spec.json
./target/release/synth examples/gfp_frame.json --verdict verdict.json
./target/release/synth input.fasta --host yeast -o optimized.fasta
```

Library dependency and call pattern:

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
let verdict = parse_bloc_verdict(&std::fs::read_to_string("verdict.json")?)?;
enforce_synthesis_gate(&verdict)?;
```

For development, run `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`.
