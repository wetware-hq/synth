"""Shared helpers: banner, deterministic record IO, text tables.

No network calls, no clocks, no randomness: every function here is a pure
function of its inputs so that re-runs are byte-identical.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

from Bio import SeqIO
from Bio.Seq import Seq
from Bio.SeqRecord import SeqRecord

BANNER_LINES = [
    "synth / cloning - design specifications only.",
    "Made-up inputs, research and not treatment. Not for use with real patients.",
    "No ordering or synthesis path. Text output only, no network.",
]

# Fixed so that GenBank output never carries a wall-clock date.
FIXED_DATE = "01-JAN-1970"


def banner(command: str) -> str:
    head = ["=" * 72, f"synth cloning :: {command}", *BANNER_LINES, "=" * 72]
    return "\n".join(head)


def print_banner(command: str) -> None:
    print(banner(command))


def guess_format(path: Path) -> str:
    suffix = path.suffix.lower()
    if suffix in {".gb", ".gbk", ".genbank"}:
        return "genbank"
    if suffix in {".fa", ".fasta", ".fna", ".seq"}:
        return "fasta"
    raise SystemExit(f"error: cannot tell format from file name: {path}")


def read_record(path: str | Path, fmt: str | None = None) -> SeqRecord:
    p = Path(path)
    if not p.exists():
        raise SystemExit(f"error: no such file: {p}")
    fmt = fmt or guess_format(p)
    records = list(SeqIO.parse(str(p), fmt))
    if not records:
        raise SystemExit(f"error: no records in {p}")
    if len(records) > 1:
        raise SystemExit(f"error: expected one record, found {len(records)} in {p}")
    rec = records[0]
    rec.seq = Seq(str(rec.seq).upper())
    return rec


def normalise_for_output(rec: SeqRecord, topology: str | None = None) -> SeqRecord:
    """Give a record the fixed metadata GenBank needs, with no clock values."""
    rec.annotations = dict(rec.annotations)
    rec.annotations["molecule_type"] = "DNA"
    rec.annotations["date"] = FIXED_DATE
    if topology:
        rec.annotations["topology"] = topology
    rec.annotations.setdefault("topology", "linear")
    if not rec.id or rec.id == "<unknown id>":
        rec.id = "construct"
    rec.name = (rec.name or rec.id)[:16].replace(" ", "_")
    return rec


def write_record(rec: SeqRecord, path: str | Path, fmt: str = "genbank") -> Path:
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    normalise_for_output(rec)
    with p.open("w") as fh:
        SeqIO.write(rec, fh, fmt)
    return p


def write_json(obj, path: str | Path) -> Path:
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    with p.open("w") as fh:
        json.dump(obj, fh, indent=2, sort_keys=True)
        fh.write("\n")
    return p


def read_json(path: str | Path):
    p = Path(path)
    if not p.exists():
        raise SystemExit(f"error: no such file: {p}")
    with p.open() as fh:
        return json.load(fh)


def gc_percent(seq: str) -> float:
    seq = seq.upper()
    if not seq:
        return 0.0
    return round(100.0 * (seq.count("G") + seq.count("C")) / len(seq), 1)


def table(headers: list[str], rows: list[list[str]]) -> str:
    cols = len(headers)
    widths = [len(h) for h in headers]
    for row in rows:
        for i in range(cols):
            widths[i] = max(widths[i], len(str(row[i])))
    out = [
        "  ".join(h.ljust(widths[i]) for i, h in enumerate(headers)),
        "  ".join("-" * widths[i] for i in range(cols)),
    ]
    for row in rows:
        out.append("  ".join(str(row[i]).ljust(widths[i]) for i in range(cols)))
    return "\n".join(out)


def wrapped_sequence(seq: str, width: int = 60) -> str:
    return "\n".join(seq[i : i + width] for i in range(0, len(seq), width))


def say(*parts: str) -> None:
    print(*parts, file=sys.stdout)
