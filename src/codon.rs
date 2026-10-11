//! Standard genetic code and preferred-codon tables.
//!
//! Preferred codons maximise local CAI (most frequent synonymous codon per host).

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

/// Preferred (most frequent) codon per amino acid for a host.
pub fn preferred_codon(aa: char, host: &str) -> Option<&'static str> {
    let table = host_table(host);
    table.iter().find(|(k, _)| *k == aa).map(|(_, v)| *v)
}

/// Alternative synonymous codons (preferred first) for pattern avoidance.
pub fn synonymous_codons(aa: char, host: &str) -> Vec<&'static str> {
    let preferred = preferred_codon(aa, host);
    let all = all_codons_for(aa);
    let mut out = Vec::new();
    if let Some(p) = preferred {
        out.push(p);
    }
    for c in all {
        if Some(c) != preferred {
            out.push(c);
        }
    }
    out
}

fn host_table(host: &str) -> &'static [(char, &'static str)] {
    match host {
        "e_coli" | "ecoli" | "e_coli_k12" | "e_coli_b" | "cell_free" | "other_bsl1" | "other"
        | "none" => ECO,
        "yeast" | "s_cerevisiae" => YEAST,
        "human" | "h_sapiens" => HUMAN,
        _ => ECO,
    }
}

fn all_codons_for(aa: char) -> Vec<&'static str> {
    match aa {
        'F' => vec!["TTT", "TTC"],
        'L' => vec!["TTA", "TTG", "CTT", "CTC", "CTA", "CTG"],
        'I' => vec!["ATT", "ATC", "ATA"],
        'M' => vec!["ATG"],
        'V' => vec!["GTT", "GTC", "GTA", "GTG"],
        'S' => vec!["TCT", "TCC", "TCA", "TCG", "AGT", "AGC"],
        'P' => vec!["CCT", "CCC", "CCA", "CCG"],
        'T' => vec!["ACT", "ACC", "ACA", "ACG"],
        'A' => vec!["GCT", "GCC", "GCA", "GCG"],
        'Y' => vec!["TAT", "TAC"],
        '*' => vec!["TAA", "TAG", "TGA"],
        'H' => vec!["CAT", "CAC"],
        'Q' => vec!["CAA", "CAG"],
        'N' => vec!["AAT", "AAC"],
        'K' => vec!["AAA", "AAG"],
        'D' => vec!["GAT", "GAC"],
        'E' => vec!["GAA", "GAG"],
        'C' => vec!["TGT", "TGC"],
        'W' => vec!["TGG"],
        'R' => vec!["CGT", "CGC", "CGA", "CGG", "AGA", "AGG"],
        'G' => vec!["GGT", "GGC", "GGA", "GGG"],
        _ => vec![],
    }
}

// Most frequent codon (CAI method) — E. coli highly expressed (Kazusa-style)
const ECO: &[(char, &str)] = &[
    ('A', "GCG"),
    ('C', "TGC"),
    ('D', "GAT"),
    ('E', "GAA"),
    ('F', "TTT"),
    ('G', "GGC"),
    ('H', "CAT"),
    ('I', "ATT"),
    ('K', "AAA"),
    ('L', "CTG"),
    ('M', "ATG"),
    ('N', "AAC"),
    ('P', "CCG"),
    ('Q', "CAG"),
    ('R', "CGT"),
    ('S', "AGC"),
    ('T', "ACC"),
    ('V', "GTG"),
    ('W', "TGG"),
    ('Y', "TAT"),
    ('*', "TAA"),
];

const YEAST: &[(char, &str)] = &[
    ('A', "GCT"),
    ('C', "TGT"),
    ('D', "GAT"),
    ('E', "GAA"),
    ('F', "TTT"),
    ('G', "GGT"),
    ('H', "CAT"),
    ('I', "ATT"),
    ('K', "AAA"),
    ('L', "TTG"),
    ('M', "ATG"),
    ('N', "AAT"),
    ('P', "CCA"),
    ('Q', "CAA"),
    ('R', "AGA"),
    ('S', "TCT"),
    ('T', "ACT"),
    ('V', "GTT"),
    ('W', "TGG"),
    ('Y', "TAT"),
    ('*', "TAA"),
];

const HUMAN: &[(char, &str)] = &[
    ('A', "GCC"),
    ('C', "TGC"),
    ('D', "GAC"),
    ('E', "GAG"),
    ('F', "TTC"),
    ('G', "GGC"),
    ('H', "CAC"),
    ('I', "ATC"),
    ('K', "AAG"),
    ('L', "CTG"),
    ('M', "ATG"),
    ('N', "AAC"),
    ('P', "CCC"),
    ('Q', "CAG"),
    ('R', "CGC"),
    ('S', "AGC"),
    ('T', "ACC"),
    ('V', "GTG"),
    ('W', "TGG"),
    ('Y', "TAC"),
    ('*', "TGA"),
];
