"""Step 1: load and inspect a plain-text sequence record (GenBank or FASTA)."""

from __future__ import annotations

import argparse

from synthlab.common import (
    gc_percent,
    print_banner,
    read_record,
    table,
)


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="synthlab.inspect",
        description="Load a GenBank or FASTA record and print what is in it.",
    )
    p.add_argument("record", help="path to a .gb/.gbk or .fa/.fasta file")
    p.add_argument("--format", choices=["genbank", "fasta"], default=None)
    return p


def describe(rec) -> str:
    seq = str(rec.seq)
    topology = rec.annotations.get("topology", "linear")
    rows = [
        ["id", rec.id],
        ["name", rec.name],
        ["description", rec.description or "(none)"],
        ["length_bp", str(len(seq))],
        ["topology", topology],
        ["gc_percent", f"{gc_percent(seq)}"],
        ["feature_count", str(len(rec.features))],
    ]
    out = [table(["field", "value"], rows), ""]
    out.append("First 60 bases: " + seq[:60])
    out.append("Last 60 bases:  " + seq[-60:])
    out.append("")
    out.append(
        "In plain English: this file holds one DNA sequence of "
        f"{len(seq)} letters, drawn as a {topology} molecule, "
        f"with {len(rec.features)} labelled regions recorded on it."
    )
    return "\n".join(out)


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    print_banner("inspect")
    rec = read_record(args.record, args.format)
    print(describe(rec))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
