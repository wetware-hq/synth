"""Step 7: emit the construct specification with a plain-English summary."""

from __future__ import annotations

import argparse

from synthlab.annotate import feature_rows
from synthlab.common import (
    BANNER_LINES,
    gc_percent,
    print_banner,
    read_record,
    table,
    wrapped_sequence,
    write_json,
)
from synthlab.digest import digest
from synthlab.orfs import find_orfs


def build_spec(rec, enzymes: list[str], min_aa: int) -> dict:
    seq = str(rec.seq)
    circular = rec.annotations.get("topology") == "circular"
    orfs = find_orfs(seq, min_aa=min_aa)
    return {
        "construct_id": rec.id,
        "description": rec.description,
        "topology": "circular" if circular else "linear",
        "length_bp": len(seq),
        "gc_percent": gc_percent(seq),
        "features": [
            {
                "label": r[0],
                "type": r[1],
                "start": int(r[2]),
                "end": int(r[3]),
                "strand": r[4],
                "length_bp": int(r[5]),
            }
            for r in feature_rows(rec)
        ],
        "open_reading_frames": [
            {k: v for k, v in o.items() if k != "protein"} for o in orfs
        ],
        "longest_protein": orfs[0]["protein"] if orfs else None,
        "restriction": digest(seq, enzymes, circular) if enzymes else None,
        "template_sequence": seq,
        "status": "design specification only; research and not treatment",
    }


def render(spec: dict) -> str:
    out: list[str] = []
    out.append("CONSTRUCT SPECIFICATION")
    out.append("")
    out.append(
        table(
            ["field", "value"],
            [
                ["construct_id", spec["construct_id"]],
                ["description", spec["description"] or "(none)"],
                ["topology", spec["topology"]],
                ["length_bp", str(spec["length_bp"])],
                ["gc_percent", str(spec["gc_percent"])],
                ["features", str(len(spec["features"]))],
                ["open_reading_frames", str(len(spec["open_reading_frames"]))],
                ["status", spec["status"]],
            ],
        )
    )
    out.append("")
    out.append("FEATURES")
    if spec["features"]:
        out.append(
            table(
                ["label", "type", "start", "end", "strand", "length_bp"],
                [
                    [f["label"], f["type"], str(f["start"]), str(f["end"]), f["strand"], str(f["length_bp"])]
                    for f in spec["features"]
                ],
            )
        )
    else:
        out.append("(none recorded)")
    out.append("")
    out.append("OPEN READING FRAMES")
    if spec["open_reading_frames"]:
        out.append(
            table(
                ["strand", "frame", "start", "end", "length_nt", "length_aa"],
                [
                    [o["strand"], str(o["frame"]), str(o["start"]), str(o["end"]), str(o["length_nt"]), str(o["length_aa"])]
                    for o in spec["open_reading_frames"]
                ],
            )
        )
    else:
        out.append("(none at the requested minimum length)")
    if spec["restriction"]:
        out.append("")
        out.append("RESTRICTION SITES")
        out.append(
            table(
                ["enzyme", "cuts", "positions_1_based"],
                [
                    [enz, str(len(pos)), ", ".join(str(p) for p in pos) or "(none)"]
                    for enz, pos in sorted(spec["restriction"]["enzymes"].items())
                ],
            )
        )
    out.append("")
    out.append("TEMPLATE SEQUENCE (5' to 3')")
    out.append(wrapped_sequence(spec["template_sequence"]))
    out.append("")
    out.append("IN PLAIN ENGLISH")
    protein = spec["longest_protein"]
    out.append(
        f"This is a design on paper for a {spec['topology']} piece of DNA "
        f"{spec['length_bp']} letters long. "
        + (
            f"It carries a recipe for one protein {len(protein)} amino acids long. "
            if protein
            else "It carries no complete protein recipe at the length we looked for. "
        )
        + f"{len(spec['features'])} parts of it are labelled with what they do."
    )
    out.append(
        "Nothing here has been made, ordered, or given to anyone. The inputs are made up, "
        "the work is for research, and it is not treatment."
    )
    out.append("")
    out.append("\n".join("# " + line for line in BANNER_LINES))
    return "\n".join(out)


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="synthlab.spec",
        description="Print the finished construct specification in words a non-specialist can read.",
    )
    p.add_argument("record")
    p.add_argument("--enzymes", default="EcoRI,BamHI,HindIII")
    p.add_argument("--min-aa", type=int, default=50)
    p.add_argument("--json-out", default=None)
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    print_banner("spec")
    rec = read_record(args.record)
    enzymes = [e.strip() for e in args.enzymes.split(",") if e.strip()]
    spec = build_spec(rec, enzymes, args.min_aa)
    print(render(spec))
    if args.json_out:
        write_json(spec, args.json_out)
        print(f"Written: {args.json_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
