//! Standard genetic code and codon usage tables (CoCoPUTs / HIVE-CUTs).
//!
//! Relative frequencies are per amino acid (synonymous codons sum to 1.0 within each family).
//! Source: FDA DNA HIVE CoCoPUTs, genomic RefSeq counts — see `data/cocoputs/` and crate README.

use crate::codon_tables::{AaCodons, table_for_host};

pub use crate::codon_tables::CodonUsage;

/// Relative usage of one synonymous codon within its amino-acid family (0.0–1.0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodonFrequency {
    pub codon: &'static str,
    pub rel_freq: f32,
}

impl From<CodonUsage> for CodonFrequency {
    fn from(u: CodonUsage) -> Self {
        Self {
            codon: u.codon,
            rel_freq: u.rel_freq,
        }
    }
}

/// NCBI standard code (table 1) codon → amino acid (one-letter; `*` stop).
pub fn translate_codon(codon: &str) -> Option<char> {
    Some(match codon {
        "TTT" | "TTC" => 'F',
        "TTA" | "TTG" | "CTT" | "CTC" | "CTA" | "CTG" => 'L',
        "ATT" | "ATC" | "ATA" => 'I',
        "ATG" => 'M',
        "GTT" | "GTC" | "GTA" | "GTG" => 'V',
        "TCT" | "TCC" | "TCA" | "TCG" | "AGT" | "AGC" => 'S',
        "CCT" | "CCC" | "CCA" | "CCG" => 'P',
        "ACT" | "ACC" | "ACA" | "ACG" => 'T',
        "GCT" | "GCC" | "GCA" | "GCG" => 'A',
        "TAT" | "TAC" => 'Y',
        "TAA" | "TAG" | "TGA" => '*',
        "CAT" | "CAC" => 'H',
        "CAA" | "CAG" => 'Q',
        "AAT" | "AAC" => 'N',
        "AAA" | "AAG" => 'K',
        "GAT" | "GAC" => 'D',
        "GAA" | "GAG" => 'E',
        "TGT" | "TGC" => 'C',
        "TGG" => 'W',
        "CGT" | "CGC" | "CGA" | "CGG" | "AGA" | "AGG" => 'R',
        "GGT" | "GGC" | "GGA" | "GGG" => 'G',
        _ => return None,
    })
}

fn aa_entry(aa: char, host: &str) -> Option<&'static AaCodons> {
    table_for_host(host)
        .iter()
        .find(|entry| entry.aa == aa)
}

/// Preferred (highest relative frequency) synonymous codon for a host.
pub fn preferred_codon(aa: char, host: &str) -> Option<&'static str> {
    aa_entry(aa, host).and_then(|e| e.codons.first().map(|c| c.codon))
}

/// Synonymous codons for an amino acid, ordered by descending CoCoPUTs relative frequency.
pub fn synonymous_codons(aa: char, host: &str) -> &'static [CodonUsage] {
    aa_entry(aa, host)
        .map(|e| e.codons)
        .unwrap_or(&[])
}

/// Relative frequency of a codon within its amino-acid family for the given host key.
pub fn codon_relative_frequency(codon: &str, host: &str) -> Option<f32> {
    let aa = translate_codon(codon)?;
    aa_entry(aa, host).and_then(|e| {
        e.codons
            .iter()
            .find(|c| c.codon == codon)
            .map(|c| c.rel_freq)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const AA_STOPS: &str = "ACDEFGHIKLMNPQRSTVWY*";

    fn assert_table_complete(host: &str) {
        for aa in AA_STOPS.chars() {
            let codons = synonymous_codons(aa, host);
            assert!(!codons.is_empty(), "missing codons for {aa} host {host}");
            let sum: f32 = codons.iter().map(|c| c.rel_freq).sum();
            assert!(
                (sum - 1.0).abs() < 1e-5,
                "freq sum for {aa} host {host}: {sum}"
            );
            assert_eq!(codons[0].codon, preferred_codon(aa, host).unwrap());
            for c in codons {
                assert_eq!(translate_codon(c.codon), Some(aa));
            }
        }
    }

    #[test]
    fn tables_complete_for_all_hosts() {
        assert_table_complete("e_coli");
        assert_table_complete("s_cerevisiae");
        assert_table_complete("h_sapiens");
    }

    #[test]
    fn yeast_leucine_top_is_ttg() {
        assert_eq!(preferred_codon('L', "s_cerevisiae"), Some("TTG"));
    }

    #[test]
    fn human_leucine_top_is_ctg() {
        assert_eq!(preferred_codon('L', "h_sapiens"), Some("CTG"));
    }

    #[test]
    fn relative_frequency_lookup() {
        let f = codon_relative_frequency("CTG", "e_coli").unwrap();
        assert!(f > 0.0 && f <= 1.0);
    }
}
