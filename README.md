# synth

synth - a design room for synthetic DNA: optimizes a coding sequence for the selected host, removes unwanted restriction digestion sites, holds GC in thermodynamically favorable range.

synth is a program that prepares a DNA coding sequence for gene synthesis or related laboratory work. The program changes codons to alternatives that are common in a host organism you select. The protein sequence stays the same. The program uses the standard NCBI genetic code, which is genetic code table 1 in the NCBI numbering. The program also tries to keep short DNA motifs out of the sequence. By default these motifs are the recognition sites for BsaI and BsmBI, which are Type IIS restriction enzymes often used in DNA assembly. The program represents these as four DNA substrings: `GGTCTC`, `GAGACC`, `CGTCTC`, and `GAGACG`. You can replace this list with your own substrings. The program also tries to keep the GC content in a target range. GC content is the fraction of bases that are G or C, from 0.0 to 1.0. The default lower bound is 0.30 and the default upper bound is 0.70. In the code these bounds are named `gc_min` and `gc_max`, and their defaults are `0.3` and `0.7`. The GC step is best effort. The program makes one forward pass over the codons. If it cannot reach the band without breaking the substring rules, it still returns the codon-optimised sequence. That sequence may remain outside the band.

The usual input is a filled Seek frame. A Seek frame is a JSON record used in Wetware design tools. The record has eight required fields at the top level: `id`, `kind`, `topo`, `seq`, `features`, `want`, `forbid`, and `phase`. The parser rejects any other top-level field. In the Rust library you parse JSON with `parse_seek_frame`, which returns a `SeekFrame` value. Before optimisation the frame must have `kind` set to DNA, `phase` set to filled, and a non-empty `seq`. The sequence length must be a multiple of three after normalisation. Normalisation removes whitespace, uppercases letters, and treats U as T. Frames with kind RNA, AA, or XNA are valid Seek records, but the optimiser returns `FrameError::NotDna` if you ask it to optimise them. The command-line program treats a file as a Seek frame when the path ends in `.json` or `.seek`, or when the file content starts with `{`. Any other input path is read as FASTA. FASTA is a simple text format with one or more sequences. The FASTA path does not write a bloc design specification file.

You choose the host with `--host` on the command line. The default is `e_coli_k12`. In the library you pass the same string as `OptimizeRequest.display_host`. The function `resolve_host` maps that string to an internal codon table key. The optimiser stores the display string on an annotation feature and stores the resolved key as `species` on that feature. When the program writes a bloc design specification, the function `resolve_bloc_chassis` maps the same host string to the chassis field expected by bloc. The table below lists the mappings implemented in the code.

| Host input | `resolve_host` (codon table) | `resolve_bloc_chassis` (bloc chassis) |
|------------|------------------------------|---------------------------------------|
| `e_coli_k12`, `ecoli`, `e_coli` | `e_coli` | `e_coli_k12` |
| `e_coli_b` | `e_coli` | `e_coli_b` |
| `yeast`, `s_cerevisiae` | `s_cerevisiae` | `yeast` |
| `cell_free` | `e_coli` | `cell_free` |
| `other_bsl1` | `e_coli` | `other_bsl1` |
| `none` | `e_coli` | `none` |
| `human`, `h_sapiens` | `h_sapiens` | `other` |
| `other` | `other` | `other` |
| any other string | lowercased; unknown names use the E. coli codon table | `other` |

If you do not pass `--avoid` on the command line, the program uses `default_avoid_patterns`, which is the four substrings listed above. In the library you set the list on `OptimizeOptions.avoid`. During codon choice the program treats a pattern as forbidden when it appears anywhere in the sequence built so far plus the candidate codon.

For Seek frame input the main output is another Seek frame with the same structure. The `seq` field holds the optimised DNA. The `phase` field stays filled. The program appends one object to `features`. That object has `type` `synth_optimised`. It also has `host`, which is the display host you passed in, `species`, which is the resolved codon key, and `gc`, which is the GC fraction after optimisation. The function `to_seek_frame_json` writes pretty-printed JSON with a trailing newline. You can validate a frame with `validate_frame_for_optimisation`, optimise it with `optimize_seek_frame`, or run both steps and build a bloc handoff with `optimize_and_handoff`. That combined function takes an `OptimizeRequest` and returns a `SynthHandoff`, which contains the frame result and a `DesignSpec`.

A bloc design specification is a JSON document for the bloc screening tool. The program can emit version `0.1.0`, written as `spec_version` `"0.1.0"` in JSON. The Rust type is `DesignSpec`. You obtain it with `design_spec_from_seek_frame` or as part of `optimize_and_handoff`. The construct identifier in the spec equals the frame `id`. Each coding feature in the frame becomes a fragment. The program looks for features whose `type` is CDS. When `start` and `end` are present, the fragment sequence is the slice of `seq` using one-based inclusive coordinates. When there are no CDS features, the program emits one fragment with role cds and alphabet dna over the full `seq`. On the command line, `--design-spec` writes this file. The CLI fills `designer.agent` with `synth` and `designer.human` with `wetware`. It sets `intended_function` to reporter, `intended_bsl` to BSL-1, and `not_for_synthesis` to true. The chassis field comes from `resolve_bloc_chassis` applied to `--host`. For FASTA input the output format is unchanged except that each record id gains the suffix `_opt`. The library functions are `parse_fasta`, `optimize_fasta_records`, and `format_fasta`.

In the Wetware workflow a filled Seek frame is optimised by synth. The next step toward synthesis is to screen a design specification with bloc. The external command is `bloc screen`. It returns a verdict whose policy is RELEASE, HOLD, or ESCALATE. synth does not run that screen and does not decide release. It can read a verdict file that bloc already produced. The function `parse_bloc_verdict` returns a `BlocVerdict`. The function `enforce_synthesis_gate` succeeds only when policy is RELEASE. Otherwise it returns `SynthesisGateError`. On the command line, `--verdict` runs this check after a successful frame run. If policy is HOLD the process exits with code 10. If policy is ESCALATE it exits with code 20. Those codes match bloc. The helper `SynthesisGateError::exit_code()` uses the same numbers. synth does not run experiments in a lab. It does not search for sequences of concern. It does not convert an amino-acid design into DNA. Use Seek tools such as `seek.filler` for that step. Automated hosts should follow [`AGENTS.md`](AGENTS.md). JSON schemas for the frame, design specification, verdict, and optimise request are published in the [Seek](https://github.com/wetware-hq/seek/blob/main/schema/frame.schema.json), [bloc design_spec](https://github.com/wetware-hq/bloc/blob/main/schemas/design_spec.schema.json), [bloc verdict](https://github.com/wetware-hq/bloc/blob/main/schemas/verdict.schema.json), and [optimise request](schema/optimize_request.schema.json) documents.

The Rust crate exposes a library API and an optional binary. The binary is enabled by the `cli` feature, which is on by default. Set `default-features = false` in Cargo if you want the library without the command-line dependency on clap. Seek types include `SeekFrame`, `FrameKind`, `FramePhase`, and `FrameTopo`, with `parse_seek_frame` and `to_seek_frame_json` for JSON. Frame errors use `FrameError`. Core sequence work uses `optimize_cds`, `OptimizeOptions`, `OptimizeError`, and `gc_content`. Host naming uses `resolve_host`, `resolve_bloc_chassis`, and `default_avoid_patterns`. The pipeline uses `OptimizeRequest`, `optimize_and_handoff`, `SynthHandoff`, and `PipelineError`. Bloc documents use `DesignSpec`, `DesignSpecOptions`, `design_spec_from_seek_frame`, `to_design_spec_json`, and `DesignSpecError`. Verdict handling uses `BlocVerdict`, `BlocPolicy`, `parse_bloc_verdict`, `enforce_synthesis_gate`, and `gate_synthesis`. Lower-level genetic code helpers are `translate_codon`, `preferred_codon`, and `synonymous_codons`.

When optimisation fails in the library, `OptimizeError::BadLength` means the sequence length is not divisible by three. `OptimizeError::UnknownCodon` includes a byte offset into `seq`. `OptimizeError::PatternConflict` means no synonymous codon could avoid the forbidden substrings at a position. `FrameError::NotDna`, `FrameError::WrongPhase`, and `FrameError::EmptySequence` describe frame preconditions. `DesignSpecError` means the program could not build a design specification from the frame. `SynthesisGateError` means the verdict policy was HOLD or ESCALATE. The command-line program exits with code 0 on success. It exits with code 1 for input or output errors, for JSON parse failure on a frame, for an empty FASTA file, for FASTA optimisation failure, or for failure to read or parse a verdict file. It exits with code 2 when frame validation or optimisation fails. It exits with code 10 when `--verdict` is set and the policy is HOLD. It exits with code 20 when `--verdict` is set and the policy is ESCALATE.

The command takes one positional argument, `input`, which is the path to a Seek frame or FASTA file. The flag `-o` or `--output` sets the output path. If you omit it, the program writes `<stem>_opt.json` for frame input or `<stem>_opt.fasta` for FASTA input, where `<stem>` is the input file name without extension. The flag `--host` sets the host string and defaults to `e_coli_k12`. The flag `--avoid` may be repeated; each value adds a forbidden substring. The flags `--gc-min` and `--gc-max` set the GC band and default to `0.3` and `0.7`. The flag `--design-spec` writes a bloc design specification to the path you give. The flag `--verdict` reads a bloc verdict from the path you give and requires RELEASE before the program exits successfully. The flags `--designer-human` and `--designer-agent` set fields on the emitted design specification; they default to `wetware` and `synth`.

To build and run from a clone of this repository:

```bash
git clone https://github.com/wetware-hq/synth.git
cd synth
cargo build --release
./target/release/synth examples/gfp_frame.json --host e_coli_k12 -o optimised.json
./target/release/synth examples/gfp_frame.json --design-spec gfp_spec.json
./target/release/synth examples/gfp_frame.json --verdict verdict.json
./target/release/synth input.fasta --host yeast -o optimized.fasta
```

To depend on the library from another Rust project, add this to `Cargo.toml`:

```toml
[dependencies]
synth = { version = "0.1", default-features = false }
```

A typical library sequence parses a frame, runs `optimize_and_handoff`, writes JSON files, and enforces a verdict after bloc has screened the design specification:

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

Contributors run `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` before merging changes.
