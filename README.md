# synth

synth is an environment: a molecular cloning bench you drive from a terminal. Every capability is a small single-purpose command over a plain-text sequence record, composable by a person or by an agent. There is no graphical interface, no interactive editor and no network call at run time.

**Design specifications only, on made-up inputs, for research and not treatment.** Not for use with real patients. No ordering path and no synthesis path exists anywhere in this environment; nothing here can cause DNA to be made.

## Bench

The bench has two stations. The cloning station walks an ordinary cloning workflow one step at a time; the codon station prepares a coding sequence for synthesis and hands off to screening.

### Cloning station

Seven commands in [`cloning/`](cloning/README.md), thin wrappers over mature libraries. The biology is Biopython, primer3-py and pydna; the original code is argument handling, plain-English output, and the checks that make a wrong result an error rather than a guess.

| Step | Command | In | Out | Library |
|---|---|---|---|---|
| 1 | `synthlab.inspect` | a GenBank or FASTA record | its length, topology and features | Biopython |
| 2 | `synthlab.annotate` | a record and a TSV of labelled regions | an annotated GenBank record | Biopython |
| 3 | `synthlab.orfs` | an annotated record | reading frames and their translations | Biopython |
| 4 | `synthlab.digest` | a record and an enzyme list | cut positions and fragment sizes | Biopython |
| 5 | `synthlab.primers` | an insert, a vector and an enzyme | a primer pair with optional Gibson tails | primer3-py |
| 6 | `synthlab.assemble` | template, primers and vector | the simulated PCR product joined into the vector | pydna |
| 7 | `synthlab.spec` | a construct record | the construct specification and a plain-words summary | Biopython |

### Codon station

The `synth` binary optimises a coding sequence for a selected host, avoids Type IIS motifs and nudges GC into a band, then hands a design specification to `bloc` for screening. It is a release client, never the release authority: `bloc` owns RELEASE, HOLD and ESCALATE, and an optimised sequence is not clearance.

```
SeekFrame (phase filled, kind DNA) -> synth -> SeekFrame + DesignSpec -> bloc screen -> verdict -> gate
```

## Invariants

- Text in, text out. No graphical interface anywhere.
- No network call at run time, and no on-disk cache. Codon tables are compiled in.
- Deterministic: pinned dependencies, fixed record dates, sorted JSON keys, checked-in example inputs. Two runs produce byte-identical output, and `demo_transcript.txt` is a known-good run kept as a fallback.
- No country-specific or site-specific value in core code. Enzyme names, homology lengths, primer limits and protein length cutoffs are arguments with neutral defaults; anything local belongs in a cited site-profile data file.
- No ordering or synthesis path, in any station.

## Gates not yet built

The environment is specified with release gates that are **not implemented**. Until they are, nothing leaving this bench is signed off for anything:

| Gate | Check id |
|---|---|
| A screen of the sequence itself before export | `SY-GATE-HAZARD` |
| Every target checked against the normal human reference sequence | `SY-GATE-HUMANREF` |
| Round-trip, reading-frame and reproducibility checks as a release condition | `SY-GATE-INTEGRITY` |
| Two-identifier match at person and specimen level, with separately signed sequencing and tissue-type artefacts | `SY-GATE-IDENTITY` |
| Two named signatures, a signed consent sheet, and a plain-words summary beside every card | `SY-GATE-SIGNOFF` |

## Run

```sh
python3 -m venv cloning/.venv
cloning/.venv/bin/pip install -r cloning/requirements.txt
bash demo.sh
```

`demo.sh` walks the whole cloning station from the checked-in example inputs and exits 0. Tests run from the `cloning` directory with `python -m pytest`; 29 cover every command and its failure paths. The codon station builds with `cargo build --release` and is tested with `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings`.

## Bar

`cargo test`, the cloning suite (29), and `demo.sh` exiting 0 are synth's bar (`cargo clippy -D warnings` too, for merges). What a green covers and what it does not — above all that the `SY-GATE-*` release gates are unbuilt, so a green clears nothing for release — is [BAR.md](BAR.md).

## Codon station reference

### Input

The primary input is a filled Seek frame: a JSON document with exactly eight required root fields (`id`, `kind`, `topo`, `seq`, `features`, `want`, `forbid`, `phase`). The parser rejects any other top-level key. Before optimisation the frame must have `kind` DNA, `phase` filled, and a non-empty `seq` whose length is a multiple of three after normalisation (whitespace removed, letters uppercased, U treated as T). Frames with kind RNA, AA, or XNA are valid Seek records, but optimisation returns `FrameError::NotDna` if invoked on them.

The command-line program treats input as a Seek frame when the path ends in `.json` or `.seek`, or when the file content begins with `{`. Any other path is read as FASTA (one or more `>` records). FASTA input supports codon optimisation only; it does not produce a bloc design specification.

### Host and codons

You select the expression context with `--host` on the command line (default `e_coli_k12`) or with `OptimizeRequest.display_host` in the library. The function `resolve_host` maps that display string to an internal codon-table key stored as `species` on the `synth_optimised` feature. The function `resolve_bloc_chassis` maps the same display string to the `chassis` field when building a bloc design specification.

| Host input | `resolve_host` (codon table) | `resolve_bloc_chassis` (bloc chassis) |
|------------|------------------------------|---------------------------------------|
| `e_coli_k12`, `ecoli`, `e_coli` | `e_coli` | `e_coli_k12` |
| `e_coli_b` | `e_coli` | `e_coli_b` |
| `yeast`, `s_cerevisiae` | `s_cerevisiae` | `yeast` |
| `cell_free` | `e_coli` | `cell_free` |
| `other_bsl1` | `e_coli` | `other_bsl1` |
| `none` | `e_coli` | `none` |
| `human`, `h_sapiens` | `h_sapiens` | `other` |
| `other` | `e_coli` | `other` |
| any other string | lowercased with spaces and hyphens as underscores; returned unchanged unless it matches a row above; codon choice uses the E. coli table unless the key is `s_cerevisiae` or `h_sapiens` | `other` |

For each codon position the optimiser considers synonymous codons in descending order of relative usage within the amino-acid family (frequencies sum to 1.0 per amino acid). It selects the first candidate that does not introduce a forbidden substring in the sequence built so far plus the candidate codon.

### Constraints

Unless you pass repeated `--avoid` flags, the program uses `default_avoid_patterns`: the Type IIS recognition motifs `GGTCTC`, `GAGACC`, `CGTCTC`, and `GAGACG` (BsaI and BsmBI). In the library, set the list on `OptimizeOptions.avoid`. If no synonymous codon can satisfy the avoid rules at a position, optimisation fails with `OptimizeError::PatternConflict`.

GC content is the fraction of bases that are G or C, from 0.0 to 1.0. Defaults are `gc_min` 0.30 and `gc_max` 0.70 (`--gc-min` / `--gc-max` on the CLI). After the preferred-codon pass, the program may perform a single forward pass of synonymous swaps to move GC into the band when possible without violating the avoid list. This step is best effort: if the band cannot be reached, the codon-optimised sequence is still returned and may remain outside the band.

### Output

For Seek frame input, the main output is another Seek frame with the same structural fields. The `seq` field holds the optimised DNA; `phase` remains filled. The program appends one feature object with `type` `synth_optimised`, plus `host` (display host), `species` (resolved codon key), and `gc` (fraction after optimisation). Serialisation uses `to_seek_frame_json` (pretty JSON with a trailing newline). Library entry points include `validate_frame_for_optimisation`, `optimize_seek_frame`, and `optimize_and_handoff`, which returns a `SynthHandoff` containing the frame result and a `DesignSpec`.

A bloc design specification is JSON for `bloc screen`, with `spec_version` `"0.1.0"` (Rust type `DesignSpec`). `construct_id` equals the frame `id`. Each feature with `type` CDS becomes a fragment; when `start` and `end` are present, the fragment sequence is the slice of `seq` using one-based inclusive coordinates. When no CDS features exist, the program emits one fragment with role `cds` and alphabet `dna` over the full `seq`. On the CLI, `--design-spec` writes this file with `designer.agent` `synth`, `designer.human` `wetware`, `intended_function` reporter, `intended_bsl` BSL-1, and `not_for_synthesis` true; `chassis` comes from `resolve_bloc_chassis` applied to `--host`. FASTA output keeps the same format except each record id gains the suffix `_opt`.

Published JSON schemas include the [Seek frame](https://github.com/wetware-hq/seek/blob/main/schema/frame.schema.json), [bloc design_spec](https://github.com/wetware-hq/bloc/blob/main/schemas/design_spec.schema.json), [bloc verdict](https://github.com/wetware-hq/bloc/blob/main/schemas/verdict.schema.json), and [optimise request](schema/optimize_request.schema.json).

### Gate

bloc screening returns a verdict whose policy is RELEASE, HOLD, or ESCALATE. synth reads an existing verdict file via `parse_bloc_verdict` and applies `enforce_synthesis_gate` (or `gate_synthesis`), which succeeds only when policy is RELEASE. On the command line, `--verdict` runs this check after a successful Seek frame run. HOLD yields exit code 10 and ESCALATE yields exit code 20, matching bloc; `SynthesisGateError::exit_code` uses the same values.

### API

The crate exposes a library API and an optional binary. The `cli` feature (on by default) builds the `synth` binary; set `default-features = false` in Cargo to depend on the library without clap.

Seek types: `SeekFrame`, `FrameKind`, `FramePhase`, `FrameTopo`, `parse_seek_frame`, `to_seek_frame_json`, `FrameError`. Sequence work: `optimize_cds`, `OptimizeOptions`, `OptimizeError`, `gc_content`. Host helpers: `resolve_host`, `resolve_bloc_chassis`, `default_avoid_patterns`. Pipeline: `OptimizeRequest`, `optimize_and_handoff`, `SynthHandoff`, `PipelineError`. Bloc documents: `DesignSpec`, `DesignSpecOptions`, `design_spec_from_seek_frame`, `to_design_spec_json`, `DesignSpecError`. Verdict handling: `BlocVerdict`, `BlocPolicy`, `parse_bloc_verdict`, `enforce_synthesis_gate`, `gate_synthesis`. Genetic code helpers: `translate_codon`, `preferred_codon`, `synonymous_codons`, `codon_relative_frequency`, `CodonUsage`. FASTA: `parse_fasta`, `optimize_fasta_records`, `format_fasta`.

Library errors include `OptimizeError::BadLength`, `OptimizeError::UnknownCodon` (with byte offset), `OptimizeError::PatternConflict`, `FrameError::NotDna`, `FrameError::WrongPhase`, `FrameError::EmptySequence`, `DesignSpecError`, and `SynthesisGateError` for HOLD or ESCALATE.

The CLI accepts one positional `input` path and optional `-o` / `--output` (default `<stem>_opt.json` or `<stem>_opt.fasta`), `--host`, repeatable `--avoid`, `--gc-min`, `--gc-max`, `--design-spec`, `--verdict`, `--designer-human` (default `wetware`), and `--designer-agent` (default `synth`). Exit code 0 indicates success. Code 1 covers I/O failure, Seek JSON parse failure, empty FASTA, FASTA optimisation failure, or verdict read/parse failure. Code 2 covers frame validation or optimisation failure. Codes 10 and 20 apply when `--verdict` is set and policy is HOLD or ESCALATE respectively.

### Provenance

Codon usage is compiled into `src/codon_tables.rs` at build time. There is no network access at runtime and no on-disk cache. Tables are derived from vendored FDA DNA HIVE CoCoPUTs (HIVE-CUTs) genomic count snapshots under `data/cocoputs/`, regenerated with `python3 data/cocoputs/generate_tables.py`. Retrieval date for the vendored files is **2026-10-11**.

| synth host key (after `resolve_host` where applicable) | CoCoPUTs taxid | Vendored file |
|--------------------------------------------------------|----------------|---------------|
| `e_coli` and aliases (`e_coli_k12`, `cell_free`, `other`, `none`, …) | 511145 (*Escherichia coli* K-12 substr. MG1655) | `taxid_511145_ecoli_k12_mg1655_genomic.tsv` |
| `s_cerevisiae` | 4932 (*Saccharomyces cerevisiae*) | `taxid_4932_s_cerevisiae_genomic.tsv` |
| `h_sapiens` | 9606 (*Homo sapiens*) | `taxid_9606_h_sapiens_genomic.tsv` |

Bulk files from the CoCoPUTs download tab are preferred when refreshing; the current snapshots were obtained via the public `ionTaxidCollapseExt` CGI endpoint documented in `data/cocoputs/README.md`. If CoCoPUTs is unavailable during regeneration, fall back to Kazusa-derived tables in [Edinburgh Genome Foundry codon-usage-tables](https://github.com/Edinburgh-Genome-Foundry/codon-usage-tables) (GenBank release 160.0, 2007) and document that provenance explicitly. The vendored data in this repository is CoCoPUTs, not Kazusa.

### Build

```bash
git clone https://github.com/wetware-hq/synth.git
cd synth
cargo build --release
./target/release/synth examples/gfp_frame.json --host e_coli_k12 -o optimised.json
./target/release/synth examples/gfp_frame.json --design-spec gfp_spec.json
./target/release/synth examples/gfp_frame.json --verdict verdict.json
./target/release/synth input.fasta --host yeast -o optimized.fasta
```

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

Contributors run `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` before merging changes.

### References

Codon usage in this crate is taken from the FDA DNA HIVE [CoCoPUTs and HIVE-CUTs](https://dnahive.fda.gov/dna.cgi?cmd=cuts_main) database. The vendored genomic RefSeq snapshots use NCBI taxonomy identifiers 511145 (*Escherichia coli* K-12 substr. MG1655), 4932 (*Saccharomyces cerevisiae*), and 9606 (*Homo sapiens*), retrieved on 2026-10-11, as described under `data/cocoputs/`.

Alexaki A, Kames J, Holcomb DD, Athey J, Santana-Quintero LV, Lam PVN, Hamasaki-Katagiri N, Osipova E, Simonyan V, Bar H, Komar AA, Kimchi-Sarfaty C. Codon and Codon-Pair Usage Tables (CoCoPUTs): facilitating genetic variation analyses and recombinant gene design. *Journal of Molecular Biology*. 2019. [https://doi.org/10.1016/j.jmb.2019.04.021](https://doi.org/10.1016/j.jmb.2019.04.021)

Protein translation follows the NCBI standard genetic code (translation table 1), as documented in [The Genetic Codes](https://www.ncbi.nlm.nih.gov/Taxonomy/Utils/wprintgc.cgi?mode=c) at NCBI.

Seek frame documents conform to the [Seek frame schema](https://github.com/wetware-hq/seek/blob/main/schema/frame.schema.json). Bloc handoff and screening use the [bloc design_spec schema](https://github.com/wetware-hq/bloc/blob/main/schemas/design_spec.schema.json) and the [bloc verdict schema](https://github.com/wetware-hq/bloc/blob/main/schemas/verdict.schema.json).
