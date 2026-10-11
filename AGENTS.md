# Integration contract

Instructions for hosts and automation that call the `synth` library or `synth` CLI. synth does **not** issue release decisions; bloc does.

## Tool purpose

- Replace codons with host-preferred synonyms (NCBI code table 1; translation preserved).
- Avoid forbidden DNA substrings (default Type IIS motifs).
- Best-effort GC adjustment toward `[gc_min, gc_max]` (default 0.30–0.70); may return a sequence still outside the band.
- Emit an optimised Seek frame and optionally a bloc `design_spec` for `bloc screen`.
- Optionally enforce that a supplied bloc verdict is `RELEASE` before treating synthesis as allowed.

## When to use

- Input is a **DNA** Seek frame with `phase: filled` and non-empty `seq` (length multiple of 3).
- Synthesis-friendly codons are required for a known host (`e_coli_k12`, `yeast`, `human`, etc.).
- Seek fill is complete; bloc screening is the next or parallel step.

## When not to use

- `kind` is `RNA`, `AA`, or `XNA` → use Seek tools first (e.g. `seek.filler` for AA→DNA).
- `phase` is not `filled` or `seq` is empty → fill sequence first.
- A safety or release decision is required → run `bloc screen`; do not infer clearance from synth output.
- De novo amino-acid sequence design → not supported.

## Input shapes

### Seek frame (required fields only)

```json
{
  "id": "string",
  "kind": "DNA" | "RNA" | "AA" | "XNA",
  "topo": "linear" | "circular",
  "seq": "string",
  "features": [],
  "want": [],
  "forbid": [],
  "phase": "spec" | "filled" | "kill" | "tube"
}
```

**Preconditions for optimisation:** `kind == DNA`, `phase == filled`, `seq` non-empty, `len(seq) % 3 == 0` after normalisation. Unknown root keys → parse error.

### FASTA

`>id` header lines and sequence lines. No frame schema. The CLI does not emit `design_spec` from FASTA input.

### Host string

Pass to `--host` or `OptimizeRequest.display_host`. Codon tables use `resolve_host` (e.g. `e_coli_k12` → `e_coli`, `yeast` → `s_cerevisiae`, `human` → `h_sapiens`). bloc `chassis` in emitted spec uses `resolve_bloc_chassis` (`human` → `other`).

### Optimise options

- `avoid`: list of forbidden substrings; default `GGTCTC`, `GAGACC`, `CGTCTC`, `GAGACG`.
- `gc_min` / `gc_max`: default `0.3` / `0.7`.

## Output shapes

### Optimised Seek frame

Same fields as input. Changes:

- `seq`: optimised DNA.
- `phase`: `filled`.
- `features`: prior entries plus one object:

```json
{
  "type": "synth_optimised",
  "host": "<display_host>",
  "species": "<resolve_host key>",
  "gc": <float>
}
```

### bloc design_spec (`spec_version` `0.1.0`)

Produce via `optimize_and_handoff` or `--design-spec`. Required root fields per bloc schema: `construct_id`, `designer`, `chassis`, `intended_function`, `intended_bsl`, `not_for_synthesis`, `fragments[]`. CLI defaults include `not_for_synthesis: true` — override in library via `DesignSpecOptions` when appropriate.

### bloc verdict (input to gate, not produced by synth)

```json
{
  "policy": "RELEASE" | "HOLD" | "ESCALATE",
  "construct_id": "...",
  ...
}
```

## API calls (library)

1. `parse_seek_frame(json)` → `SeekFrame`
2. `validate_frame_for_optimisation(&frame)` → `Ok` or `FrameError`
3. `optimize_seek_frame(frame, display_host, &OptimizeOptions)` → optimised frame
4. `optimize_and_handoff(frame, &OptimizeRequest)` → `SynthHandoff { frame_result, design_spec }`
5. After `bloc screen`: `parse_bloc_verdict(json)` then `enforce_synthesis_gate(&verdict)` → `Ok` only if `RELEASE`

FASTA: `parse_fasta` → `optimize_fasta_records` → `format_fasta`.

## Failure modes

| Error | Meaning | Remediation |
|-------|---------|-------------|
| `FrameError::NotDna` | Wrong `kind` | Run AA→DNA fill; retry with DNA frame |
| `FrameError::WrongPhase` | Not `filled` | Complete Seek fill; set `phase: filled` |
| `FrameError::EmptySequence` | No sequence | Fill `seq` first |
| `OptimizeError::BadLength` | Length not multiple of 3 | Fix sequence length |
| `OptimizeError::UnknownCodon` | Invalid codon | Fix input sequence |
| `OptimizeError::PatternConflict` | Motifs block all synonyms at a site | Relax `--avoid` or change constraints |
| `serde_json::Error` | Invalid JSON | Fix file |
| `SynthesisGateError` | Verdict not `RELEASE` | Stop synthesis path; follow bloc card |
| CLI exit `2` | Frame validation/optimise failed | Read stderr message |
| CLI exit `10` / `20` | HOLD / ESCALATE verdict | Do not order synthesis |

## Pipeline rules

1. Use a Seek frame when bloc handoff is required; FASTA is for sequence-only interchange.
2. After synth, run `bloc screen` on `design_spec` before any synthesis or ordering step.
3. Call `enforce_synthesis_gate` (or `synth --verdict`) when a verdict file exists; do not proceed on `HOLD` or `ESCALATE`.
4. Do not treat `synth_optimised` feature or optimised `seq` as a safety clearance.

## Do not assume

- GC is always inside 0.30–0.70 after optimisation (best-effort only).
- `not_for_synthesis: true` in default CLI spec means the plan is cleared for ordering (it does not).
- synth runs bloc or returns a verdict.
- `human` maps to a bloc chassis enum value (it maps to `other`).
- FASTA output is sufficient for bloc without building a `design_spec`.
- Pattern avoidance removes all restriction sites beyond the configured substrings.

## CLI reference

```
synth <input> [-o PATH] [--host HOST] [--avoid PAT]... [--gc-min N] [--gc-max N]
              [--design-spec PATH] [--verdict PATH] [--designer-human S] [--designer-agent S]
```

Exit codes: `0` success; `1` I/O/parse/FASTA errors; `2` frame optimise; `10` HOLD verdict; `20` ESCALATE verdict.
