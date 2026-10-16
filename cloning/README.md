# synth / cloning

A terminal-native, agent-invokable molecular cloning environment.

**Design specifications only, on made-up inputs, for research and not treatment.**
Not for use with real patients. No ordering path and no synthesis path exists
anywhere in this toolset; nothing here can cause DNA to be made.

## What it is

Seven small single-purpose commands, one per step of an ordinary cloning
workflow. Each reads plain-text files and writes plain text; there is no
graphical interface and no interactive editor, and nothing makes a network call
at run time. Every command prints the same scope banner in its header.

The commands are thin wrappers. The biology is done by mature open-source
libraries: Biopython for record I/O, features, translation and restriction
analysis, primer3-py for primer design, pydna for the PCR and Gibson
simulations. Original code here is argument handling, plain-English output and
the checks that make a wrong result an error rather than a guess.

| Step | Command | What it does | Library |
|---|---|---|---|
| 1 | `synthlab.inspect` | load and inspect a GenBank or FASTA record | Biopython |
| 2 | `synthlab.annotate` | add labelled regions from a TSV, list them | Biopython |
| 3 | `synthlab.orfs` | find open reading frames and translate | Biopython |
| 4 | `synthlab.digest` | restriction digest, report cuts and fragments | Biopython |
| 5 | `synthlab.primers` | design a primer pair, optional Gibson tails | primer3-py |
| 6 | `synthlab.assemble` | simulate the PCR, then the Gibson join | pydna |
| 7 | `synthlab.spec` | emit the construct specification + plain-English summary | Biopython |

## Determinism

Dependencies are pinned in `requirements.txt`. No clock value, random seed or
network response reaches the output: GenBank records are written with a fixed
date, JSON is written with sorted keys, and the example inputs are checked in
rather than generated at run time. Two runs of `demo.sh` produce byte-identical
output; `demo_transcript.txt` at the repository root is a known-good run kept as
a fallback.

## Portability

No country-specific or site-specific value appears in the code. Enzyme names,
homology lengths, primer limits and minimum protein lengths are command-line
arguments with neutral defaults.

## Install

```sh
python3 -m venv cloning/.venv
cloning/.venv/bin/pip install -r cloning/requirements.txt
```

`demo.sh` picks `$SYNTH_PYTHON` if set, else `cloning/.venv/bin/python`, else
`python3`, and stops with a clear message if the libraries are missing.

## Demo flow

```sh
./demo.sh
```

It runs the seven steps from the checked-in example input and exits 0. Files
land in `cloning/out/`, which is not tracked. To run a step by hand:

```sh
cd cloning
export PYTHONPATH="$PWD"
python -m synthlab.inspect examples/demo_vector.gb
python -m synthlab.annotate examples/demo_insert.fasta --features examples/demo_features.tsv --out out/insert_annotated.gb
python -m synthlab.orfs out/insert_annotated.gb --min-aa 100 --out out/orfs.json
python -m synthlab.digest examples/demo_vector.gb --enzymes EcoRI,BamHI,HindIII --out out/digest.json
python -m synthlab.primers examples/demo_insert.fasta --vector examples/demo_vector.gb --enzyme EcoRI --homology 30 --out out/primers.json
python -m synthlab.assemble --template examples/demo_insert.fasta --primers out/primers.json --vector examples/demo_vector.gb --enzyme EcoRI --out out/construct.gb
python -m synthlab.spec out/construct.gb --enzymes EcoRI,BamHI,HindIII --min-aa 100 --json-out out/construct_spec.json
```

Every command takes `--help`.

## Example inputs

`examples/demo_insert.fasta` (720 bp, one invented 200-amino-acid gene) and
`examples/demo_vector.gb` (1891 bp circular backbone with one EcoRI, one BamHI
and one HindIII site). Both are invented, from a fixed seed, by
`examples/make_example_inputs.py`; they are not drawn from any organism, patient
or database. The finished construct is 2573 bp.

## Tests

```sh
cd cloning
export PYTHONPATH="$PWD"
python -m pytest
```

29 tests cover every command, including the failure paths: an unknown enzyme, an
enzyme that does not cut exactly once, out-of-range feature coordinates, a
vector without an enzyme, an assembly with no overlap, and two re-run checks for
byte-identical output.

## Not in scope

Interactive or graphical sequence editors, cloud storage, accounts, ordering,
synthesis export, and any path that turns a specification into a physical
molecule.
