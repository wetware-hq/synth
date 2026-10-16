"""Step 4: restriction digest."""

from __future__ import annotations

import argparse

from Bio.Restriction import CommOnly, RestrictionBatch
from Bio.Seq import Seq

from synthlab.common import print_banner, read_record, table, write_json


def make_batch(names: list[str]) -> RestrictionBatch:
    known = {str(e) for e in CommOnly}
    unknown = [n for n in names if n not in known]
    if unknown:
        raise SystemExit(f"error: unknown enzyme(s): {', '.join(unknown)}")
    return RestrictionBatch(names)


def digest(seq: str, names: list[str], circular: bool) -> dict:
    batch = make_batch(names)
    sites = batch.search(Seq(seq.upper()), linear=not circular)
    per_enzyme = {str(e): sorted(pos) for e, pos in sites.items()}
    cuts = sorted({p for positions in per_enzyme.values() for p in positions})
    if not cuts:
        fragments = [len(seq)]
    elif circular:
        fragments = [
            (cuts[(i + 1) % len(cuts)] - c) % len(seq) or len(seq)
            for i, c in enumerate(cuts)
        ]
    else:
        edges = [0, *cuts, len(seq)]
        fragments = [edges[i + 1] - edges[i] for i in range(len(edges) - 1)]
    return {
        "enzymes": per_enzyme,
        "cut_positions": cuts,
        "fragment_lengths": sorted(fragments, reverse=True),
        "topology": "circular" if circular else "linear",
    }


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="synthlab.digest",
        description="Cut a sequence with named restriction enzymes and report the pieces.",
    )
    p.add_argument("record")
    p.add_argument("--enzymes", required=True, help="comma-separated, e.g. EcoRI,BamHI")
    p.add_argument("--circular", action="store_true", help="treat the sequence as a closed loop")
    p.add_argument("--out", default=None, help="optional JSON file to write")
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    print_banner("digest")
    rec = read_record(args.record)
    circular = args.circular or rec.annotations.get("topology") == "circular"
    names = [n.strip() for n in args.enzymes.split(",") if n.strip()]
    result = digest(str(rec.seq), names, circular)
    rows = [
        [enz, str(len(pos)), ", ".join(str(p) for p in pos) or "(none)"]
        for enz, pos in sorted(result["enzymes"].items())
    ]
    print(table(["enzyme", "cuts", "positions_1_based"], rows))
    print()
    print("Fragment lengths (bp): " + ", ".join(str(f) for f in result["fragment_lengths"]))
    print()
    singles = [e for e, p in result["enzymes"].items() if len(p) == 1]
    print(
        f"In plain English: these enzymes cut the {result['topology']} molecule in "
        f"{len(result['cut_positions'])} places, leaving "
        f"{len(result['fragment_lengths'])} pieces. "
        + (
            f"Cutting exactly once: {', '.join(sorted(singles))}. Those are the useful "
            "places to open the molecule and drop something in."
            if singles
            else "No enzyme here cuts exactly once."
        )
    )
    if args.out:
        write_json(result, args.out)
        print(f"Written: {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
