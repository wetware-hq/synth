# CoCoPUTs source tables (HIVE-CUTs)

Codon counts retrieved from the FDA DNA HIVE **CoCoPUTs** database ([HIVE-CUTs](https://dnahive.fda.gov/dna.cgi?cmd=cuts_main)).

| File | TaxID | Organism | Filter |
|------|-------|----------|--------|
| `taxid_511145_ecoli_k12_mg1655_genomic.tsv` | 511145 | *Escherichia coli* K-12 substr. MG1655 | genomic (not organelle) |
| `taxid_4932_s_cerevisiae_genomic.tsv` | 4932 | *Saccharomyces cerevisiae* | genomic |
| `taxid_9606_h_sapiens_genomic.tsv` | 9606 | *Homo sapiens* | genomic |

Retrieval: `dna.cgi?cmd=ionTaxidCollapseExt` with `fileSource=Refseq_species.tsv`, `filterInColName=["Organelle"]`, `filterIn=["genomic"]`, `searchDeep=true`, `raw=1` (bulk download tab was not used; see pull request).

Retrieved: **2026-10-11**.

Regenerate `src/codon_tables.rs` with:

```bash
python3 data/cocoputs/generate_tables.py
```
