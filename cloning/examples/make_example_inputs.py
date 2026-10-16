"""Generate the checked-in example inputs. Deterministic: a fixed seed, no network.

The sequences are invented. They are not drawn from any organism, patient or
database, and they code for nothing that exists.
"""

from __future__ import annotations

import random
from pathlib import Path

from Bio.Seq import Seq
from Bio.SeqRecord import SeqRecord
from Bio import SeqIO

HERE = Path(__file__).parent
SEED = 20261017
BAD = ["GAATTC", "GGATCC", "AAGCTT", "GTCGAC", "CTGCAG"]  # keep sites where we put them
STOPS = {"TAA", "TAG", "TGA"}


def clean_random(rng: random.Random, n: int) -> str:
    while True:
        s = "".join(rng.choice("ACGT") for _ in range(n))
        if not any(b in s for b in BAD):
            return s


def coding(rng: random.Random, aa: int) -> str:
    codons = [
        c
        for c in (a + b + d for a in "ACGT" for b in "ACGT" for d in "ACGT")
        if c not in STOPS
    ]
    while True:
        body = "".join(rng.choice(codons) for _ in range(aa))
        s = "ATG" + body + "TAA"
        if not any(b in s for b in BAD):
            return s


def main() -> None:
    rng = random.Random(SEED)

    # Insert: a made-up gene with short flanks.
    cds = coding(rng, 199)
    insert = clean_random(rng, 60) + cds + clean_random(rng, 57)
    insert_rec = SeqRecord(Seq(insert), id="DEMO-INSERT", description="made-up insert with one invented gene")
    SeqIO.write(insert_rec, HERE / "demo_insert.fasta", "fasta")

    # Vector: a made-up circular backbone with one EcoRI site, one BamHI, one HindIII.
    marker = coding(rng, 149)
    backbone = (
        clean_random(rng, 400)
        + marker
        + clean_random(rng, 300)
        + "GAATTC"
        + clean_random(rng, 120)
        + "GGATCC"
        + clean_random(rng, 200)
        + "AAGCTT"
        + clean_random(rng, 400)
    )
    vec = SeqRecord(Seq(backbone), id="DEMO-VECTOR", name="demo_vector",
                    description="made-up circular backbone, one EcoRI site")
    vec.annotations["molecule_type"] = "DNA"
    vec.annotations["topology"] = "circular"
    vec.annotations["date"] = "01-JAN-1970"
    SeqIO.write(vec, HERE / "demo_vector.gb", "genbank")
    print("wrote demo_insert.fasta and demo_vector.gb")


if __name__ == "__main__":
    main()
