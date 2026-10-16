"""Shared vector helpers: open a circular vector at one unique enzyme site."""

from __future__ import annotations

from Bio.Restriction import RestrictionBatch
from Bio.Seq import Seq


def linearise(seq: str, enzyme: str) -> str:
    """Return the vector sequence rotated so it begins at a single cut site.

    The enzyme must cut exactly once; anything else is an error rather than a
    guess, because a guess here silently builds the wrong molecule.
    """
    seq = seq.upper()
    batch = RestrictionBatch([enzyme])
    hits = batch.search(Seq(seq), linear=False)
    positions = sorted(next(iter(hits.values())))
    if len(positions) != 1:
        raise SystemExit(
            f"error: {enzyme} cuts the vector {len(positions)} times; "
            "a single cut is required to open it"
        )
    cut = positions[0] - 1  # Biopython reports 1-based cut positions
    return seq[cut:] + seq[:cut]


def homology_arms(linear_vector: str, length: int) -> tuple[str, str]:
    """(upstream arm, downstream arm) flanking the place the insert will sit."""
    if length * 2 > len(linear_vector):
        raise SystemExit("error: homology length is too long for this vector")
    return linear_vector[-length:], linear_vector[:length]
