# synth

synth - a design room for synthetic DNA: optimizes a coding sequence for the selected host, removes unwanted restriction digestion sites, holds GC in thermodynamically favorable range.

## Clinical summary

synth is a computer program that adjusts a DNA coding sequence before gene synthesis or related laboratory work. The program replaces codons with alternatives that are common in a chosen host organism while keeping the same protein sequence. It also tries to remove short DNA motifs associated with Type IIS restriction enzymes used in DNA assembly, and it tries to keep the fraction of G and C bases within a set range. The preferred input is a filled Seek frame, which is a structured JSON record used in the Wetware design tools. The program can also read a simple FASTA file when a frame is not available. The main output is an updated Seek frame with the same structure as the input and an annotation that optimisation occurred. The program can also write a bloc design specification file that downstream screening software expects. synth does not decide whether a design is safe to build. Another program called bloc performs that review and returns a verdict of RELEASE, HOLD, or ESCALATE. synth can check an existing bloc verdict and refuse to treat a design as cleared for synthesis when the verdict is not RELEASE. synth does not run laboratory steps, does not screen for sequences of concern, and does not create DNA from an amino-acid sequence alone. Amino-acid sequences must be converted to DNA elsewhere before synth is used.

## Technical specification

### Pipeline position

```text
seek frame (phase: filled)  →  synth  →  optimised seek frame
                                          ↓
                                bloc design_spec  →  bloc screen  →  RELEASE | HOLD | ESCALATE
```

Integration contract for hosts and automation: [`AGENTS.md`](AGENTS.md). Schemas: [Seek frame](https://github.com/wetware-hq/seek/blob/main/schema/frame.schema.json), [bloc design_spec](https://github.com/wetware-hq/bloc/blob/main/schemas/design_spec.schema.json), [bloc verdict](https://github.com/wetware-hq/bloc/blob/main/schemas/verdict.schema.json), [optimise request](schema/optimize_request.schema.json).

### Input contracts

**Seek frame (primary).** JSON object with exactly these root fields (unknown keys rejected at parse): `id`, `kind`, `topo`, `seq`, `features`, `want`, `forbid`, `phase`. Optimisation requires `kind: DNA`, `phase: filled`, and non-empty `seq`. Length of `seq` must be a multiple of three after normalisation. Kinds `RNA`, `AA`, and `XNA` are valid Seek kinds but are rejected for codon optimisation. Detection: file extension `.json` or `.seek`, or body starts with `{`.

**FASTA (secondary).** One or more records; convenience only. No bloc handoff from FASTA. Detection: any non-JSON input path.

**Host string (`--host` / `display_host`).** Selects codon preference table and is stored on the `synth_optimised` feature. Mapped to internal codon keys via `resolve_host` and to bloc `chassis` via `resolve_bloc_chassis` when emitting `design_spec`.

| Host input | `resolve_host` (codons) | `resolve_bloc_chassis` |
|------------|-------------------------|-------------------------|
| `e_coli_k12`, `ecoli`, `e_coli` | `e_coli` | `e_coli_k12` |
| `e_coli_b` | `e_coli` | `e_coli_b` |
| `yeast`, `s_cerevisiae` | `s_cerevisiae` | `yeast` |
| `cell_free` | `e_coli` | `cell_free` |
| `other_bsl1` | `e_coli` | `other_bsl1` |
| `none` | `e_coli` | `none` |
| `human`, `h_sapiens` | `h_sapiens` | `other` |
| `other` | `other` | `other` |
| (any other string) | lowercased; unknown keys use E. coli codon table as fallback | `other` |

### Output contracts

**Optimised Seek frame.** Same schema as input. `seq` replaced; `phase` remains `filled`. A feature object is appended: `type: synth_optimised`, `host` (display host), `species` (resolved codon key), `gc` (fraction after optimisation). Serialisation: `to_seek_frame_json` (pretty JSON + trailing newline).

**bloc design_spec (optional).** `spec_version: "0.1.0"`. Built by `design_spec_from_seek_frame` / `optimize_and_handoff`. `construct_id` = frame `id`. Fragments: one entry per `CDS` feature (sequence slice from `start`/`end` if present), or a single `cds` / `dna` fragment covering full `seq` if no CDS features. CLI defaults: `designer.agent: synth`, `designer.human: wetware`, `intended_function: reporter`, `intended_bsl: BSL-1`, `not_for_synthesis: true`, `chassis` from `resolve_bloc_chassis(--host)`.

**FASTA.** Same format as input; record ids gain suffix `_opt`.

### Optimisation algorithm

1. **Normalisation.** Remove whitespace; uppercase; `U` → `T`.
2. **Codon pass.** For each codon, choose the most frequent synonymous codon for the host (NCBI genetic code table 1). If the growing sequence would contain a forbidden substring, try the next synonymous codon in preference order. Failure → `OptimizeError::PatternConflict`.
3. **GC adjustment (best effort, single pass).** If GC is outside `[gc_min, gc_max]`, scan codons once and swap to synonymous codons that move GC toward the target band without introducing forbidden substrings. If a swap places GC inside the band, return that sequence. If the scan finishes inside the band, return it. Otherwise return the codon-optimised sequence from step 2 even if GC remains outside the band. Default window: **0.30–0.70** (`gc_min` / `gc_max`).
4. **Translation.** Amino-acid sequence is preserved when steps 1–3 succeed.

**Default avoided restriction motifs (substring match, not enzyme names):** `GGTCTC`, `GAGACC`, `CGTCTC`, `GAGACG` (BsaI / BsmBI Type IIS sites). Override with repeated `--avoid` or `OptimizeOptions.avoid`.

### Public API surface

| Module / export | Responsibility |
|-----------------|----------------|
| `parse_seek_frame`, `SeekFrame`, `FrameKind`, `FramePhase`, `FrameTopo` | Seek JSON types |
| `validate_frame_for_optimisation`, `optimize_seek_frame`, `to_seek_frame_json` | Frame path |
| `optimize_cds`, `OptimizeOptions`, `OptimizeError`, `gc_content` | Core CDS logic |
| `resolve_host`, `default_avoid_patterns` | Host aliases and default motifs |
| `parse_fasta`, `format_fasta`, `optimize_fasta_records`, `FastaRecord` | FASTA path |
| `OptimizeRequest`, `optimize_and_handoff`, `SynthHandoff`, `PipelineError` | Frame + `design_spec` |
| `DesignSpec`, `DesignSpecOptions`, `design_spec_from_seek_frame`, `to_design_spec_json`, `resolve_bloc_chassis` | bloc handoff |
| `parse_bloc_verdict`, `BlocVerdict`, `BlocPolicy`, `enforce_synthesis_gate`, `SynthesisGateError` | Verdict gate |
| `translate_codon`, `preferred_codon`, `synonymous_codons` | Genetic code helpers |

Library crate: `default-features = false` omits the CLI (`clap`). Feature `cli` (default) builds the `synth` binary.

### Error and exit semantics

**Library errors**

| Type | Meaning |
|------|---------|
| `OptimizeError::BadLength` | `seq` length not divisible by 3 |
| `OptimizeError::UnknownCodon` | Non-standard codon at byte offset |
| `OptimizeError::PatternConflict` | No synonymous codon avoids all patterns at a position |
| `FrameError::NotDna` | `kind` ≠ DNA |
| `FrameError::WrongPhase` | `phase` ≠ `filled` |
| `FrameError::EmptySequence` | `seq` empty |
| `DesignSpecError` | Cannot build `design_spec` from frame |
| `SynthesisGateError` | Verdict policy is `HOLD` or `ESCALATE` |

**CLI exit codes**

| Code | Condition |
|------|-----------|
| 0 | Success |
| 1 | I/O failure, JSON parse error, empty FASTA, FASTA optimisation error, verdict read/parse error |
| 2 | Seek frame validation or optimisation failure |
| 10 | `--verdict` present and policy `HOLD` (matches bloc) |
| 20 | `--verdict` present and policy `ESCALATE` (matches bloc) |

`enforce_synthesis_gate` returns `Ok(())` only for `RELEASE`. `SynthesisGateError::exit_code()` uses 10 / 20.

### CLI

| Flag | Default | Effect |
|------|---------|--------|
| `input` | (required) | Seek frame or FASTA path |
| `-o`, `--output` | `<stem>_opt.json` or `_opt.fasta` | Output path |
| `--host` | `e_coli_k12` | Host / chassis string |
| `--avoid` | (see motifs above) | Repeatable forbidden substrings |
| `--gc-min` | `0.3` | Lower GC bound |
| `--gc-max` | `0.7` | Upper GC bound |
| `--design-spec` | (none) | Write bloc `design_spec` JSON |
| `--verdict` | (none) | After success, require `RELEASE` in verdict file |
| `--designer-human` | `wetware` | `designer.human` in emitted spec |
| `--designer-agent` | `synth` | `designer.agent` in emitted spec |

### Installation and usage

```bash
git clone https://github.com/wetware-hq/synth.git
cd synth
cargo build --release
```

Seek frame:

```bash
./target/release/synth examples/gfp_frame.json --host e_coli_k12 -o optimised.json
./target/release/synth examples/gfp_frame.json --design-spec gfp_spec.json
./target/release/synth examples/gfp_frame.json --verdict verdict.json
```

FASTA:

```bash
./target/release/synth input.fasta --host yeast -o optimized.fasta
```

Library dependency:

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

### Development

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```