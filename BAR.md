# synth — the bar

What a green means in synth, and what it does not. A green here is not the same assurance as a green in a sibling repo; read this before quoting one.

**Design specifications only, on made-up inputs, for research and not treatment.** No ordering path and no synthesis path exists anywhere in this environment; nothing here can cause DNA to be made.

## The bar

```sh
# codon station (Rust)
cargo test
cargo clippy --all-targets --all-features -- -D warnings

# cloning station (Python)
python3 -m venv cloning/.venv
cloning/.venv/bin/pip install -r cloning/requirements.txt
cd cloning && python -m pytest        # 29 tests, incl. failure paths and re-run checks
bash demo.sh                          # walks the cloning station from checked-in inputs; exits 0
```

A green is: `cargo test` passing, the cloning suite (29) passing, and `demo.sh` exiting 0. `cargo clippy -D warnings` is part of the bar for merges.

## What a green covers

- **Codon station** — frame parsing with `deny_unknown_fields`, validation (`kind` DNA, `phase` filled, non-empty `seq` whose normalised length is a multiple of three), preferred-codon optimisation that skips any candidate introducing a forbidden substring, Type IIS avoid list, best-effort GC-band nudge, translation preserved, and the `DesignSpec` handoff shape. Error paths (`BadLength`, `UnknownCodon`, `PatternConflict`, `NotDna`, `WrongPhase`, `EmptySequence`) are tested.
- **Gate behaviour** — `enforce_synthesis_gate` succeeds only on a bloc `RELEASE` verdict; `HOLD` exits 10 and `ESCALATE` exits 20. synth reads a verdict; it is a release *client*, never the release authority.
- **Cloning station** — the seven commands and their failure paths, with re-run determinism checks (two runs give identical output).
- **Reproducibility** — pinned dependencies, fixed record dates, sorted JSON keys and checked-in example inputs; the cloning suite's re-run checks assert byte-identical output across two runs, and `demo_transcript.txt` is kept as a known-good fallback.
- **No local values in core** — enzyme names, homology lengths, primer limits and protein-length cutoffs are arguments with neutral defaults; anything site-specific belongs in a cited site-profile data file.

## What a green does NOT cover

- **The release gates. They are specified but unbuilt.** None of `SY-GATE-HAZARD`, `SY-GATE-HUMANREF`, `SY-GATE-INTEGRITY`, `SY-GATE-IDENTITY` or `SY-GATE-SIGNOFF` is implemented, so no code runs under those ids. A green does **not** mean anything leaving this bench is screened, reference-checked, identity-matched or signed off. Until those gates exist, nothing here is cleared for anything, and an optimised sequence is not clearance.
- **`cargo clippy` on this box.** Clippy is part of the bar for merges but is not installed on this checkout, so it was not run here; `cargo test`, the cloning suite and `demo.sh` were.
- **Real patients or real use.** There is no ordering or synthesis path; a green is a code check on made-up inputs.

## Name guard

Not feasible here, and not faked. synth has no registry of check ids that *run*: the only ids it names, `SY-GATE-*`, name release gates that are unbuilt, so there is no running check a name-guard could hold to this file. Were any `SY-GATE-*` implemented, it would be listed above. Keep this in step by review.
