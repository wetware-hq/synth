"""Step 5: design primers with primer3-py, optionally with Gibson homology tails."""

from __future__ import annotations

import argparse

import primer3

from synthlab.common import gc_percent, print_banner, read_record, table, write_json
from synthlab.vectorutil import homology_arms, linearise


def design_core_primers(template: str, opt_size: int = 20, product_min: int | None = None) -> dict:
    """primer3 global design across the whole template, settings fixed in code."""
    template = template.upper()
    product_min = product_min or max(100, len(template) - 40)
    seq_args = {
        "SEQUENCE_ID": "template",
        "SEQUENCE_TEMPLATE": template,
    }
    global_args = {
        "PRIMER_TASK": "generic",
        "PRIMER_PICK_LEFT_PRIMER": 1,
        "PRIMER_PICK_RIGHT_PRIMER": 1,
        "PRIMER_PICK_INTERNAL_OLIGO": 0,
        "PRIMER_OPT_SIZE": opt_size,
        "PRIMER_MIN_SIZE": 18,
        "PRIMER_MAX_SIZE": 25,
        "PRIMER_OPT_TM": 60.0,
        "PRIMER_MIN_TM": 57.0,
        "PRIMER_MAX_TM": 63.0,
        "PRIMER_MIN_GC": 35.0,
        "PRIMER_MAX_GC": 65.0,
        "PRIMER_MAX_POLY_X": 4,
        "PRIMER_SALT_MONOVALENT": 50.0,
        "PRIMER_DNA_CONC": 50.0,
        "PRIMER_NUM_RETURN": 1,
        "PRIMER_PRODUCT_SIZE_RANGE": [[product_min, len(template)]],
    }
    res = primer3.bindings.design_primers(seq_args, global_args)
    if not res.get("PRIMER_PAIR_NUM_RETURNED"):
        raise SystemExit(
            "error: primer3 found no acceptable pair for this template "
            f"({res.get('PRIMER_LEFT_EXPLAIN', '')} | {res.get('PRIMER_RIGHT_EXPLAIN', '')})"
        )
    return {
        "left": {
            "sequence": res["PRIMER_LEFT_0_SEQUENCE"],
            "tm": round(res["PRIMER_LEFT_0_TM"], 2),
            "start": res["PRIMER_LEFT_0"][0] + 1,
            "length": res["PRIMER_LEFT_0"][1],
        },
        "right": {
            "sequence": res["PRIMER_RIGHT_0_SEQUENCE"],
            "tm": round(res["PRIMER_RIGHT_0_TM"], 2),
            "start": res["PRIMER_RIGHT_0"][0] + 1,
            "length": res["PRIMER_RIGHT_0"][1],
        },
        "product_size": res["PRIMER_PAIR_0_PRODUCT_SIZE"],
    }


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="synthlab.primers",
        description="Design a primer pair for a template, with optional Gibson tails.",
    )
    p.add_argument("template", help="insert record to amplify")
    p.add_argument("--vector", default=None, help="circular vector record, for Gibson tails")
    p.add_argument("--enzyme", default=None, help="enzyme that opens the vector once")
    p.add_argument("--homology", type=int, default=30, help="tail length in bases")
    p.add_argument("--out", required=True, help="JSON file to write")
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    print_banner("primers")
    template = str(read_record(args.template).seq)
    core = design_core_primers(template)

    tails = {"left": "", "right": ""}
    if args.vector:
        if not args.enzyme:
            raise SystemExit("error: --vector needs --enzyme so the vector can be opened")
        linear_vector = linearise(str(read_record(args.vector).seq), args.enzyme)
        upstream, downstream = homology_arms(linear_vector, args.homology)
        from Bio.Seq import Seq

        tails["left"] = upstream
        tails["right"] = str(Seq(downstream).reverse_complement())

    full_left = tails["left"] + core["left"]["sequence"]
    full_right = tails["right"] + core["right"]["sequence"]
    payload = {
        "template": args.template,
        "vector": args.vector,
        "enzyme": args.enzyme,
        "homology_length": args.homology if args.vector else 0,
        "core": core,
        "forward_primer": full_left,
        "reverse_primer": full_right,
    }
    write_json(payload, args.out)

    rows = [
        [
            "forward",
            str(len(full_left)),
            f"{core['left']['tm']}",
            f"{gc_percent(core['left']['sequence'])}",
            str(len(tails["left"])),
        ],
        [
            "reverse",
            str(len(full_right)),
            f"{core['right']['tm']}",
            f"{gc_percent(core['right']['sequence'])}",
            str(len(tails["right"])),
        ],
    ]
    print(table(["primer", "total_length", "annealing_tm_c", "annealing_gc_pct", "tail_length"], rows))
    print()
    print(f"forward: 5'-{full_left}-3'")
    print(f"reverse: 5'-{full_right}-3'")
    print()
    print(
        "In plain English: these two short pieces of DNA mark the start and end of the "
        f"region to copy. The copied piece will be {core['product_size']} bases long"
        + (
            f", and each primer carries {args.homology} extra bases matching the vector so "
            "the pieces can be joined without any cut-and-paste enzyme."
            if args.vector
            else "."
        )
    )
    print(f"Written: {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
