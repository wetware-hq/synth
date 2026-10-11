//! Codon optimisation for synthesis and host.
//!
//! Method: most-frequent synonymous codon (local CAI maximisation),
//! with fallback to next synonymous codons to avoid forbidden patterns
//! and to keep GC in range where possible.

use crate::codon::{synonymous_codons, translate_codon};
use crate::host::{default_avoid_patterns, resolve_host};
use thiserror::Error;

/// Options controlling codon optimisation.
#[derive(Debug, Clone)]
pub struct OptimizeOptions {
    /// Internal host key for codon tables (use [`resolve_host`] on user input).
    pub host: String,
    /// Substrings that must not appear in the optimised sequence.
    pub avoid: Vec<String>,
    /// Minimum GC fraction (0.0–1.0).
    pub gc_min: f64,
    /// Maximum GC fraction (0.0–1.0).
    pub gc_max: f64,
}

impl OptimizeOptions {
    /// Build options with default avoid patterns and GC bounds for a resolved host key.
    pub fn for_host(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            avoid: default_avoid_patterns(),
            gc_min: 0.3,
            gc_max: 0.7,
        }
    }

    /// Build options from a user-facing host name (normalised via [`resolve_host`]).
    pub fn from_chassis(chassis: &str) -> Self {
        Self::for_host(resolve_host(chassis))
    }
}

impl Default for OptimizeOptions {
    fn default() -> Self {
        Self::for_host("e_coli")
    }
}

/// Errors returned by [`optimize_cds`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum OptimizeError {
    #[error("sequence length not divisible by 3")]
    BadLength,
    #[error("unknown codon at position {0}")]
    UnknownCodon(usize),
    #[error("no synonymous codon avoids patterns at position {0}")]
    PatternConflict(usize),
}

/// Optimise a coding sequence (DNA; length must be a multiple of 3).
///
/// Whitespace is stripped and letters are uppercased; `U` is treated as `T`.
/// Translation is preserved under the standard genetic code.
pub fn optimize_cds(seq: &str, opts: &OptimizeOptions) -> Result<String, OptimizeError> {
    let avoid_refs: Vec<&str> = opts.avoid.iter().map(|s| s.as_str()).collect();
    optimize_cds_with_refs(seq, &opts.host, &avoid_refs, opts.gc_min, opts.gc_max)
}

/// GC fraction of a DNA sequence (G and C over total length).
pub fn gc_content(seq: &str) -> f64 {
    if seq.is_empty() {
        return 0.0;
    }
    let gc = seq.chars().filter(|c| *c == 'G' || *c == 'C').count();
    gc as f64 / seq.len() as f64
}

fn optimize_cds_with_refs(
    seq: &str,
    host: &str,
    avoid: &[&str],
    gc_min: f64,
    gc_max: f64,
) -> Result<String, OptimizeError> {
    let seq = normalise(seq);
    if seq.len() % 3 != 0 {
        return Err(OptimizeError::BadLength);
    }

    let mut out = String::with_capacity(seq.len());
    let n = seq.len() / 3;

    for i in 0..n {
        let codon = &seq[i * 3..i * 3 + 3];
        let aa = translate_codon(codon).ok_or(OptimizeError::UnknownCodon(i * 3))?;
        let candidates = synonymous_codons(aa, host);
        if candidates.is_empty() {
            return Err(OptimizeError::UnknownCodon(i * 3));
        }

        let chosen = choose_codon(candidates, &out, avoid)
            .ok_or(OptimizeError::PatternConflict(i * 3))?;
        out.push_str(chosen);
    }

    if let Some(adjusted) = adjust_gc(&out, host, avoid, gc_min, gc_max) {
        return Ok(adjusted);
    }
    Ok(out)
}

fn normalise(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .map(|c| if c == 'U' { 'T' } else { c })
        .collect()
}

fn choose_codon<'a>(
    candidates: &'a [crate::CodonUsage],
    prefix: &str,
    avoid: &[&str],
) -> Option<&'a str> {
    for c in candidates {
        let trial = format!("{prefix}{}", c.codon);
        if !contains_any(&trial, avoid) {
            return Some(c.codon);
        }
    }
    None
}

fn contains_any(seq: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| seq.contains(p))
}

fn adjust_gc(
    seq: &str,
    host: &str,
    avoid: &[&str],
    gc_min: f64,
    gc_max: f64,
) -> Option<String> {
    let gc = gc_content(seq);
    if gc >= gc_min && gc <= gc_max {
        return None;
    }
    let want_higher = gc < gc_min;
    let mut codons: Vec<String> = (0..seq.len() / 3)
        .map(|i| seq[i * 3..i * 3 + 3].to_string())
        .collect();

    for i in 0..codons.len() {
        let aa = translate_codon(&codons[i])?;
        let alts = synonymous_codons(aa, host);
        for alt in alts {
            if alt.codon == codons[i] {
                continue;
            }
            let alt_gc = alt.codon.chars().filter(|c| *c == 'G' || *c == 'C').count();
            let cur_gc = codons[i].chars().filter(|c| *c == 'G' || *c == 'C').count();
            let improves = if want_higher {
                alt_gc > cur_gc
            } else {
                alt_gc < cur_gc
            };
            if !improves {
                continue;
            }
            let old = codons[i].clone();
            codons[i] = alt.codon.to_string();
            let trial: String = codons.iter().cloned().collect();
            if !contains_any(&trial, avoid) {
                let new_gc = gc_content(&trial);
                if new_gc >= gc_min && new_gc <= gc_max {
                    return Some(trial);
                }
            } else {
                codons[i] = old;
            }
        }
    }
    let final_seq: String = codons.into_iter().collect();
    let final_gc = gc_content(&final_seq);
    if final_gc >= gc_min && final_gc <= gc_max {
        Some(final_seq)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn translate_cds(seq: &str) -> Option<String> {
        let seq = normalise(seq);
        if seq.len() % 3 != 0 {
            return None;
        }
        let mut aa = String::new();
        for i in 0..seq.len() / 3 {
            let codon = &seq[i * 3..i * 3 + 3];
            aa.push(translate_codon(codon)?);
        }
        Some(aa)
    }

    #[test]
    fn preserves_translation() {
        let input = "ATGAAATTT";
        let opts = OptimizeOptions::default();
        let out = optimize_cds(input, &opts).unwrap();
        assert_eq!(translate_cds(&out), translate_cds(input));
    }

    #[test]
    fn rejects_bad_length() {
        let opts = OptimizeOptions::default();
        assert_eq!(
            optimize_cds("ATGAA", &opts).unwrap_err(),
            OptimizeError::BadLength
        );
    }

    #[test]
    fn normalises_input() {
        let opts = OptimizeOptions::default();
        let out = optimize_cds("atg aaa", &opts).unwrap();
        assert_eq!(out.len(), 6);
    }

    #[test]
    fn gc_content_empty() {
        assert_eq!(gc_content(""), 0.0);
    }
}
