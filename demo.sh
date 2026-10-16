#!/usr/bin/env bash
# synth / cloning - the whole happy path, one command per step.
# Design specifications only, on made-up inputs, for research and not treatment.
# No network calls, no graphical interface, no ordering or synthesis path.
set -euo pipefail

cd "$(dirname "$0")/cloning"

if [[ -n "${SYNTH_PYTHON:-}" ]]; then
  PY="$SYNTH_PYTHON"
elif [[ -x ".venv/bin/python" ]]; then
  PY=".venv/bin/python"
else
  PY="python3"
fi

if ! "$PY" -c "import Bio, pydna, primer3" >/dev/null 2>&1; then
  echo "error: biopython, pydna and primer3-py are not installed for $PY" >&2
  echo "       python3 -m venv cloning/.venv && cloning/.venv/bin/pip install -r cloning/requirements.txt" >&2
  exit 1
fi

export PYTHONPATH="$PWD"
export PYTHONHASHSEED=0
OUT=out
rm -rf "$OUT"
mkdir -p "$OUT"

step () {
  echo
  echo "### STEP $1: $2"
  echo "\$ $3"
  echo
}

step 1 "load and inspect the made-up vector" \
  "python -m synthlab.inspect examples/demo_vector.gb"
"$PY" -m synthlab.inspect examples/demo_vector.gb

step 2 "label the regions of the made-up insert" \
  "python -m synthlab.annotate examples/demo_insert.fasta --features examples/demo_features.tsv --out out/insert_annotated.gb"
"$PY" -m synthlab.annotate examples/demo_insert.fasta \
  --features examples/demo_features.tsv --out "$OUT/insert_annotated.gb"

step 3 "find the protein recipes and read them out" \
  "python -m synthlab.orfs out/insert_annotated.gb --min-aa 100 --out out/orfs.json"
"$PY" -m synthlab.orfs "$OUT/insert_annotated.gb" --min-aa 100 --out "$OUT/orfs.json"

step 4 "find the single-cut enzymes that can open the vector" \
  "python -m synthlab.digest examples/demo_vector.gb --enzymes EcoRI,BamHI,HindIII --out out/digest.json"
"$PY" -m synthlab.digest examples/demo_vector.gb --enzymes EcoRI,BamHI,HindIII --out "$OUT/digest.json"

step 5 "design the primer pair, with homology tails for the join" \
  "python -m synthlab.primers examples/demo_insert.fasta --vector examples/demo_vector.gb --enzyme EcoRI --homology 30 --out out/primers.json"
"$PY" -m synthlab.primers examples/demo_insert.fasta --vector examples/demo_vector.gb \
  --enzyme EcoRI --homology 30 --out "$OUT/primers.json"

step 6 "simulate the copy step and the join step" \
  "python -m synthlab.assemble --template examples/demo_insert.fasta --primers out/primers.json --vector examples/demo_vector.gb --enzyme EcoRI --out out/construct.gb"
"$PY" -m synthlab.assemble --template examples/demo_insert.fasta --primers "$OUT/primers.json" \
  --vector examples/demo_vector.gb --enzyme EcoRI --out "$OUT/construct.gb"

step 7 "emit the construct specification with a plain-English summary" \
  "python -m synthlab.spec out/construct.gb --enzymes EcoRI,BamHI,HindIII --min-aa 100 --json-out out/construct_spec.json"
"$PY" -m synthlab.spec "$OUT/construct.gb" --enzymes EcoRI,BamHI,HindIII --min-aa 100 \
  --json-out "$OUT/construct_spec.json"

echo
echo "### DONE: 7 of 7 steps finished. Files are under cloning/out/."
echo "### Nothing was ordered, made, or sent anywhere."
