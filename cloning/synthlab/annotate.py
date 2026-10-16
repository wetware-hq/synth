"""Step 2: annotate features onto a record and list them."""

from __future__ import annotations

import argparse
import csv
from pathlib import Path

from Bio.SeqFeature import SeqFeature, SimpleLocation

from synthlab.common import print_banner, read_record, table, write_record


def read_feature_table(path: str | Path) -> list[dict]:
    p = Path(path)
    if not p.exists():
        raise SystemExit(f"error: no such file: {p}")
    rows = []
    with p.open(newline="") as fh:
        reader = csv.DictReader(filter(lambda l: not l.startswith("#"), fh), delimiter="\t")
        required = {"label", "type", "start", "end", "strand"}
        for row in reader:
            missing = required - set(row)
            if missing:
                raise SystemExit(f"error: feature table missing columns: {sorted(missing)}")
            rows.append(row)
    if not rows:
        raise SystemExit(f"error: no feature rows in {p}")
    return rows


def apply_features(rec, rows: list[dict]):
    """Add features given as 1-based inclusive coordinates."""
    for row in rows:
        start = int(row["start"]) - 1
        end = int(row["end"])
        if start < 0 or end > len(rec.seq) or start >= end:
            raise SystemExit(
                f"error: feature {row['label']} coordinates fall outside the sequence"
            )
        strand = 1 if str(row["strand"]).strip() in {"1", "+", "+1"} else -1
        feat = SeqFeature(
            SimpleLocation(start, end, strand=strand),
            type=row["type"],
            qualifiers={"label": [row["label"]], "note": [row.get("note", "")]},
        )
        rec.features.append(feat)
    rec.features.sort(key=lambda f: (int(f.location.start), int(f.location.end)))
    return rec


def feature_rows(rec) -> list[list[str]]:
    rows = []
    for feat in rec.features:
        label = feat.qualifiers.get("label", ["(unlabelled)"])[0]
        rows.append(
            [
                label,
                feat.type,
                str(int(feat.location.start) + 1),
                str(int(feat.location.end)),
                "+" if feat.location.strand == 1 else "-",
                str(int(feat.location.end) - int(feat.location.start)),
            ]
        )
    return rows


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="synthlab.annotate",
        description="Add labelled regions to a record from a tab-separated table, then list them.",
    )
    p.add_argument("record")
    p.add_argument("--features", required=True, help="tab-separated file: label type start end strand note")
    p.add_argument("--out", required=True, help="GenBank file to write")
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    print_banner("annotate")
    rec = read_record(args.record)
    rec = apply_features(rec, read_feature_table(args.features))
    out = write_record(rec, args.out)
    print(table(["label", "type", "start", "end", "strand", "length_bp"], feature_rows(rec)))
    print()
    print(
        f"In plain English: {len(rec.features)} regions of this DNA are now labelled, "
        f"and the labelled copy was written to {out}."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
