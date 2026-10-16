"""Step 3: find open reading frames and translate them."""

from __future__ import annotations

import argparse

from Bio.Seq import Seq

from synthlab.common import print_banner, read_record, table, write_json

STOPS = {"TAA", "TAG", "TGA"}


def find_orfs(seq: str, min_aa: int = 50, both_strands: bool = True) -> list[dict]:
    """Plain ATG-to-stop scan in all frames, NCBI standard code (table 1)."""
    seq = seq.upper()
    found: list[dict] = []
    strands = [1, -1] if both_strands else [1]
    for strand in strands:
        work = seq if strand == 1 else str(Seq(seq).reverse_complement())
        for frame in range(3):
            i = frame
            while i < len(work) - 2:
                if work[i : i + 3] == "ATG":
                    j = i
                    while j < len(work) - 2:
                        codon = work[j : j + 3]
                        if codon in STOPS:
                            nt = work[i : j + 3]
                            aa = str(Seq(nt).translate(table=1, to_stop=True))
                            if len(aa) >= min_aa:
                                if strand == 1:
                                    start, end = i, j + 3
                                else:
                                    start, end = len(work) - (j + 3), len(work) - i
                                found.append(
                                    {
                                        "start": start + 1,
                                        "end": end,
                                        "strand": "+" if strand == 1 else "-",
                                        "frame": frame + 1,
                                        "length_nt": len(nt),
                                        "length_aa": len(aa),
                                        "protein": aa,
                                    }
                                )
                            i = j + 3
                            break
                        j += 3
                    else:
                        break
                else:
                    i += 3
    found.sort(key=lambda o: (-o["length_aa"], o["start"], o["strand"]))
    return found


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="synthlab.orfs",
        description="Find open reading frames (ATG to stop) and translate them.",
    )
    p.add_argument("record")
    p.add_argument("--min-aa", type=int, default=50, help="shortest protein to report")
    p.add_argument("--forward-only", action="store_true")
    p.add_argument("--out", default=None, help="optional JSON file to write")
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    print_banner("orfs")
    rec = read_record(args.record)
    orfs = find_orfs(str(rec.seq), args.min_aa, not args.forward_only)
    if not orfs:
        print(f"No open reading frame of at least {args.min_aa} amino acids was found.")
        return 0
    rows = [
        [str(i + 1), o["strand"], str(o["frame"]), str(o["start"]), str(o["end"]), str(o["length_nt"]), str(o["length_aa"])]
        for i, o in enumerate(orfs)
    ]
    print(table(["n", "strand", "frame", "start", "end", "length_nt", "length_aa"], rows))
    print()
    best = orfs[0]
    print("Longest protein (one letter per amino acid):")
    protein = best["protein"]
    for i in range(0, len(protein), 60):
        print("  " + protein[i : i + 60])
    print()
    print(
        f"In plain English: {len(orfs)} stretches of this DNA read as complete protein "
        f"recipes. The longest is {best['length_aa']} amino acids, starting at base "
        f"{best['start']} on the {best['strand']} strand."
    )
    if args.out:
        write_json({"min_aa": args.min_aa, "orfs": orfs}, args.out)
        print(f"Written: {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
