"""Step 6: simulate the PCR and the Gibson assembly with pydna."""

from __future__ import annotations

import argparse
import warnings

warnings.filterwarnings("ignore")

from pydna.amplify import pcr  # noqa: E402
from pydna.assembly2 import Assembly, gibson_overlap  # noqa: E402
from pydna.dseqrecord import Dseqrecord  # noqa: E402
from Bio.Seq import Seq  # noqa: E402
from Bio.SeqRecord import SeqRecord  # noqa: E402
from Bio.SeqFeature import SeqFeature, SimpleLocation  # noqa: E402

from synthlab.common import print_banner, read_json, read_record, table, write_record  # noqa: E402
from synthlab.vectorutil import linearise  # noqa: E402


def simulate_pcr(template_seq: str, forward: str, reverse: str) -> Dseqrecord:
    amplicon = pcr(forward, reverse, Dseqrecord(template_seq, circular=False))
    return Dseqrecord(str(amplicon.seq), circular=False)


def simulate_gibson(linear_vector: str, insert: str, limit: int) -> Dseqrecord:
    frags = [Dseqrecord(linear_vector, circular=False), Dseqrecord(insert, circular=False)]
    asm = Assembly(frags, limit=limit, algorithm=gibson_overlap, use_all_fragments=True)
    products = asm.assemble_circular()
    if not products:
        raise SystemExit(
            "error: no circular product; the homology arms do not overlap the vector ends"
        )
    return products[0]


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="synthlab.assemble",
        description="Copy the insert by PCR, then join it into the opened vector (Gibson).",
    )
    p.add_argument("--template", required=True, help="insert record")
    p.add_argument("--primers", required=True, help="JSON written by synthlab.primers")
    p.add_argument("--vector", required=True, help="circular vector record")
    p.add_argument("--enzyme", required=True, help="enzyme that opens the vector once")
    p.add_argument("--limit", type=int, default=25, help="shortest overlap counted as a join")
    p.add_argument("--out", required=True, help="GenBank file for the finished construct")
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    print_banner("assemble")
    template = str(read_record(args.template).seq)
    primers = read_json(args.primers)
    vector_rec = read_record(args.vector)
    linear_vector = linearise(str(vector_rec.seq), args.enzyme)

    amplicon = simulate_pcr(template, primers["forward_primer"], primers["reverse_primer"])
    product = simulate_gibson(linear_vector, str(amplicon.seq), args.limit)

    seq = str(product.seq).upper()
    product_rec = SeqRecord(
        Seq(seq),
        id="SYNTH-DEMO-CONSTRUCT",
        name="construct",
        description="demo construct: made-up insert in made-up vector",
    )
    product_rec.annotations["topology"] = "circular"

    # Label the piece that was dropped in, so the specification can show it.
    homology = int(primers["homology_length"])
    core = str(amplicon.seq)[homology : len(str(amplicon.seq)) - homology] if homology else str(amplicon.seq)
    start = seq.find(core)
    if start >= 0:
        product_rec.features.append(
            SeqFeature(
                SimpleLocation(start, start + len(core), strand=1),
                type="misc_feature",
                qualifiers={"label": ["inserted_fragment"], "note": ["copied insert, joined by homology"]},
            )
        )
    out = write_record(product_rec, args.out)

    rows = [
        ["template (insert)", str(len(template))],
        ["pcr product", str(len(str(amplicon.seq)))],
        ["opened vector", str(len(linear_vector))],
        ["circular construct", str(len(seq))],
    ]
    print(table(["piece", "length_bp"], rows))
    print()
    expected = len(linear_vector) + len(str(amplicon.seq)) - 2 * primers["homology_length"]
    print(f"Expected construct length from the arithmetic: {expected} bp")
    print(f"Observed construct length: {len(seq)} bp")
    print()
    print(
        "In plain English: the copy step produced the insert with matching ends, and the "
        "join step closed it into the vector as one circle. Nothing was ordered or made; "
        "this is a simulation on paper."
    )
    print(f"Written: {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
