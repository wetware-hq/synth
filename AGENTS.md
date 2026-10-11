# Agent contract

Normative requirements for hosts and agents that use synth. Wording uses RFC 2119: **MUST**, **MUST NOT**, **SHOULD**.

## Scope

synth is a design-studio room that optimises sequence parameters for synthesis constraints and host expression. The systems implementation is a Rust library (`synth`) with a thin `synth` CLI. It does **not** set release policy.

The canonical sequence representation is a [Seek frame](https://github.com/wetware-hq/seek) (`schema/frame.schema.json`). The only sequence-bearing object that may proceed toward ordering, assembly, or transformation is a [bloc design specification](https://github.com/wetware-hq/bloc) (`schemas/design_spec.schema.json`, `spec_version` `0.1.0`) that has received a `RELEASE` verdict from `bloc screen`.

## Interface

```
seek frame (phase: filled)  →  synth (Rust)  →  optimised seek frame
                                              ↓
                                    bloc design specification  →  bloc screen  →  Verdict + card
```

Hosts **MUST** route any optimised sequence that is intended for synthesis through `bloc screen` before it leaves the local machine. synth **MUST NOT** be the sole basis for a synthesis clearance. Use [`enforce_synthesis_gate`](https://docs.rs/synth/latest/synth/fn.enforce_synthesis_gate.html) (or `synth --verdict`) so `HOLD` and `ESCALATE` block the synthesis path.

## Inputs

- Prefer a Seek frame JSON (eight root fields; see wetware-hq/seek).
- `kind` **MUST** be `DNA` for codon optimisation of coding sequences.
- `phase` **MUST** be `filled` before optimisation (use `seek.filler` for AA→DNA).
- FASTA is accepted as a convenience input (not for bloc handoff).
- Host / chassis names **SHOULD** match bloc `chassis` values where possible (`resolve_bloc_chassis` / `--host`).

## Outputs

- Optimised Seek frame (updated `seq`, phase `filled`, `synth_optimised` feature).
- Optional bloc `design_spec` JSON (`optimize_and_handoff`, `--design-spec`).
- Optimised FASTA (interchange only).

## Method (v0)

- Most-frequent synonymous codon for the target host (local CAI maximisation).
- Fallback to next synonymous codons to avoid declared patterns (default: BsaI / BsmBI sites).
- Soft GC adjustment into [gc_min, gc_max] (default 0.3–0.7).
- Translation preserved under NCBI standard code (table 1).

Host codon tables: `e_coli_k12` / `e_coli_b` → `e_coli`, `yeast` → `s_cerevisiae`, `human` → `h_sapiens` (`resolve_host`).

## Out of scope

- Release / HOLD / ESCALATE decisions (bloc owns these; synth only enforces an existing verdict when asked)
- Running `bloc screen` (hosts invoke the bloc binary)
- Wet-laboratory execution
- Sequences-of-concern screening
- Inventing DNA from amino-acid sequences (use `seek.filler` first)

## Status

v0.1. Systems path is Rust. Aligns with Seek frame v0 and bloc design_spec 0.1.0.
