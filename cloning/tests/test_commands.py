"""One test file, every command. No network, no clock, no randomness."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from synthlab import annotate, assemble, common, digest, inspect as inspect_cmd, orfs, primers, spec
from synthlab.vectorutil import homology_arms, linearise

EXAMPLES = Path(__file__).resolve().parents[1] / "examples"
INSERT = EXAMPLES / "demo_insert.fasta"
VECTOR = EXAMPLES / "demo_vector.gb"
FEATURES = EXAMPLES / "demo_features.tsv"
BANNER_MARK = "research and not treatment"


# --- common -----------------------------------------------------------------

def test_banner_carries_the_scope_line():
    assert BANNER_MARK in common.banner("x")
    assert "No ordering or synthesis path" in common.banner("x")


def test_gc_percent():
    assert common.gc_percent("GGCC") == 100.0
    assert common.gc_percent("ATAT") == 0.0
    assert common.gc_percent("") == 0.0


def test_read_record_rejects_a_missing_file(tmp_path):
    with pytest.raises(SystemExit):
        common.read_record(tmp_path / "nope.fasta")


def test_written_genbank_has_no_wall_clock_date(tmp_path):
    rec = common.read_record(INSERT)
    out = common.write_record(rec, tmp_path / "x.gb")
    assert common.FIXED_DATE in out.read_text()


# --- step 1: inspect --------------------------------------------------------

def test_inspect_fasta(capsys):
    assert inspect_cmd.main([str(INSERT)]) == 0
    out = capsys.readouterr().out
    assert BANNER_MARK in out
    assert "length_bp" in out and "720" in out


def test_inspect_genbank_sees_circular(capsys):
    assert inspect_cmd.main([str(VECTOR)]) == 0
    assert "circular" in capsys.readouterr().out


def test_inspect_rejects_unknown_extension(tmp_path):
    p = tmp_path / "x.weird"
    p.write_text(">a\nACGT\n")
    with pytest.raises(SystemExit):
        inspect_cmd.main([str(p)])


# --- step 2: annotate -------------------------------------------------------

def test_annotate_adds_and_lists_features(tmp_path, capsys):
    out = tmp_path / "annotated.gb"
    assert annotate.main([str(INSERT), "--features", str(FEATURES), "--out", str(out)]) == 0
    text = capsys.readouterr().out
    assert "demo_gene" in text and "upstream_flank" in text
    rec = common.read_record(out)
    assert len(rec.features) == 3


def test_annotate_refuses_out_of_range_coordinates(tmp_path):
    ft = tmp_path / "bad.tsv"
    ft.write_text("label\ttype\tstart\tend\tstrand\tnote\nx\tCDS\t1\t999999\t1\t\n")
    with pytest.raises(SystemExit):
        annotate.main([str(INSERT), "--features", str(ft), "--out", str(tmp_path / "o.gb")])


# --- step 3: orfs -----------------------------------------------------------

def test_orfs_finds_the_planted_gene(tmp_path, capsys):
    out = tmp_path / "orfs.json"
    assert orfs.main([str(INSERT), "--min-aa", "100", "--out", str(out)]) == 0
    data = json.loads(out.read_text())
    assert data["orfs"][0]["length_aa"] == 200
    assert data["orfs"][0]["start"] == 61
    assert "protein" in data["orfs"][0]
    assert BANNER_MARK in capsys.readouterr().out


def test_orf_translation_matches_the_dna():
    found = orfs.find_orfs("ATG" + "GCT" * 60 + "TAA", min_aa=10)
    assert found[0]["protein"] == "M" + "A" * 60
    assert found[0]["length_nt"] == 186


def test_orfs_reports_nothing_when_nothing_is_long_enough(capsys):
    assert orfs.main([str(INSERT), "--min-aa", "5000"]) == 0
    assert "No open reading frame" in capsys.readouterr().out


# --- step 4: digest ---------------------------------------------------------

def test_digest_finds_one_site_per_enzyme(tmp_path, capsys):
    out = tmp_path / "digest.json"
    assert digest.main([str(VECTOR), "--enzymes", "EcoRI,BamHI,HindIII", "--out", str(out)]) == 0
    data = json.loads(out.read_text())
    assert [len(v) for v in data["enzymes"].values()] == [1, 1, 1]
    assert sum(data["fragment_lengths"]) == 1891
    assert data["topology"] == "circular"
    assert BANNER_MARK in capsys.readouterr().out


def test_digest_rejects_an_unknown_enzyme():
    with pytest.raises(SystemExit):
        digest.main([str(VECTOR), "--enzymes", "NotAnEnzyme"])


# --- vector helpers ---------------------------------------------------------

def test_linearise_rotates_without_losing_bases():
    vec = str(common.read_record(VECTOR).seq)
    lin = linearise(vec, "EcoRI")
    assert len(lin) == len(vec)
    assert sorted(lin) == sorted(vec)
    assert lin + lin[: len(lin)] != ""  # rotation, not reverse
    assert lin in vec + vec


def test_linearise_refuses_an_enzyme_that_does_not_cut_once():
    vec = str(common.read_record(VECTOR).seq)
    with pytest.raises(SystemExit):
        linearise(vec, "AluI")


def test_homology_arms_lengths():
    up, down = homology_arms("A" * 50 + "C" * 50, 20)
    assert up == "C" * 20 and down == "A" * 20


# --- step 5: primers --------------------------------------------------------

def test_primers_without_tails(tmp_path, capsys):
    out = tmp_path / "p.json"
    assert primers.main([str(INSERT), "--out", str(out)]) == 0
    data = json.loads(out.read_text())
    assert data["homology_length"] == 0
    assert data["forward_primer"] == data["core"]["left"]["sequence"]
    assert 57.0 <= data["core"]["left"]["tm"] <= 63.0
    assert BANNER_MARK in capsys.readouterr().out


def test_primers_with_gibson_tails(tmp_path):
    out = tmp_path / "p.json"
    assert primers.main(
        [str(INSERT), "--vector", str(VECTOR), "--enzyme", "EcoRI", "--homology", "30", "--out", str(out)]
    ) == 0
    data = json.loads(out.read_text())
    lin = linearise(str(common.read_record(VECTOR).seq), "EcoRI")
    up, _ = homology_arms(lin, 30)
    assert data["forward_primer"].startswith(up)
    assert len(data["forward_primer"]) == 30 + data["core"]["left"]["length"]


def test_primers_needs_an_enzyme_with_a_vector(tmp_path):
    with pytest.raises(SystemExit):
        primers.main([str(INSERT), "--vector", str(VECTOR), "--out", str(tmp_path / "p.json")])


def test_primer_design_is_repeatable():
    template = str(common.read_record(INSERT).seq)
    assert primers.design_core_primers(template) == primers.design_core_primers(template)


# --- step 6: assemble -------------------------------------------------------

@pytest.fixture
def built(tmp_path):
    pj = tmp_path / "p.json"
    primers.main(
        [str(INSERT), "--vector", str(VECTOR), "--enzyme", "EcoRI", "--homology", "30", "--out", str(pj)]
    )
    gb = tmp_path / "construct.gb"
    assemble.main(
        [
            "--template", str(INSERT),
            "--primers", str(pj),
            "--vector", str(VECTOR),
            "--enzyme", "EcoRI",
            "--out", str(gb),
        ]
    )
    return gb


def test_assembly_length_is_the_arithmetic(built, capsys):
    rec = common.read_record(built)
    assert len(rec.seq) == 2573
    assert rec.annotations["topology"] == "circular"


def test_assembly_labels_the_inserted_piece(built):
    rec = common.read_record(built)
    labels = [f.qualifiers["label"][0] for f in rec.features]
    assert "inserted_fragment" in labels


def test_assembly_keeps_the_planted_gene(built):
    rec = common.read_record(built)
    found = orfs.find_orfs(str(rec.seq), min_aa=200)
    assert any(o["length_aa"] == 200 for o in found)


def test_assembly_is_byte_identical_on_a_rerun(tmp_path, built):
    second = tmp_path / "again.gb"
    assemble.main(
        [
            "--template", str(INSERT),
            "--primers", str(tmp_path / "p.json"),
            "--vector", str(VECTOR),
            "--enzyme", "EcoRI",
            "--out", str(second),
        ]
    )
    assert second.read_bytes() == built.read_bytes()


def test_assembly_fails_loudly_without_overlap(tmp_path):
    pj = tmp_path / "p.json"
    primers.main([str(INSERT), "--out", str(pj)])  # no tails, so nothing to join
    with pytest.raises(SystemExit):
        assemble.main(
            [
                "--template", str(INSERT),
                "--primers", str(pj),
                "--vector", str(VECTOR),
                "--enzyme", "EcoRI",
                "--out", str(tmp_path / "x.gb"),
            ]
        )


# --- step 7: spec -----------------------------------------------------------

def test_spec_prints_every_section(built, tmp_path, capsys):
    js = tmp_path / "spec.json"
    assert spec.main([str(built), "--enzymes", "EcoRI,BamHI,HindIII", "--min-aa", "100", "--json-out", str(js)]) == 0
    out = capsys.readouterr().out
    for section in ["CONSTRUCT SPECIFICATION", "FEATURES", "OPEN READING FRAMES", "TEMPLATE SEQUENCE", "IN PLAIN ENGLISH"]:
        assert section in out
    assert BANNER_MARK in out
    data = json.loads(js.read_text())
    assert data["length_bp"] == 2573
    assert data["template_sequence"] == str(common.read_record(built).seq)
    assert data["status"].startswith("design specification only")


def test_spec_says_nothing_was_ordered(built, capsys):
    spec.main([str(built)])
    assert "Nothing here has been made, ordered, or given to anyone." in capsys.readouterr().out


def test_spec_json_is_stable(built, tmp_path):
    a, b = tmp_path / "a.json", tmp_path / "b.json"
    spec.main([str(built), "--json-out", str(a)])
    spec.main([str(built), "--json-out", str(b)])
    assert a.read_bytes() == b.read_bytes()
