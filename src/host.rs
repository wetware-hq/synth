//! Host / chassis name normalisation and default synthesis constraints.

/// Map CLI or user-facing host names to internal codon-table keys.
///
/// Examples: `e_coli_k12` → `e_coli`, `yeast` → `s_cerevisiae`, `human` → `h_sapiens`.
pub fn resolve_host(h: &str) -> String {
    let key = h.to_lowercase().replace([' ', '-'], "_");
    match key.as_str() {
        "e_coli_k12" | "e_coli_b" | "ecoli" | "e_coli" | "cell_free" | "other_bsl1"
        | "other" | "none" => "e_coli".into(),
        "yeast" | "s_cerevisiae" => "s_cerevisiae".into(),
        "human" | "h_sapiens" => "h_sapiens".into(),
        other => other.to_string(),
    }
}

/// Default Type IIS / Golden Gate sites to avoid (BsaI and BsmBI).
pub fn default_avoid_patterns() -> Vec<String> {
    vec![
        "GGTCTC".into(),
        "GAGACC".into(),
        "CGTCTC".into(),
        "GAGACG".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_common_hosts() {
        assert_eq!(resolve_host("e_coli_k12"), "e_coli");
        assert_eq!(resolve_host("Yeast"), "s_cerevisiae");
        assert_eq!(resolve_host("human"), "h_sapiens");
    }
}
