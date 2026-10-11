//! FASTA parsing and formatting.

use crate::optimize::{OptimizeError, OptimizeOptions, optimize_cds};

/// One FASTA record (header id and raw sequence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastaRecord {
    pub id: String,
    pub sequence: String,
}

/// Parse FASTA text into records (first token of each header line is the id).
pub fn parse_fasta(text: &str) -> Vec<FastaRecord> {
    let mut out = Vec::new();
    let mut id = String::new();
    let mut seq = String::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix('>') {
            if !id.is_empty() {
                out.push(FastaRecord {
                    id: id.clone(),
                    sequence: seq.clone(),
                });
            }
            id = rest.split_whitespace().next().unwrap_or("seq").to_string();
            seq.clear();
        } else {
            seq.push_str(line.trim());
        }
    }
    if !id.is_empty() {
        out.push(FastaRecord { id, sequence: seq });
    }
    out
}

/// Format records as FASTA text (trailing newline).
pub fn format_fasta(records: &[FastaRecord]) -> String {
    let mut buf = String::new();
    for rec in records {
        buf.push('>');
        buf.push_str(&rec.id);
        buf.push('\n');
        buf.push_str(&rec.sequence);
        buf.push('\n');
    }
    buf
}

/// Optimise every record in place; ids gain a `_opt` suffix.
pub fn optimize_fasta_records(
    records: &mut [FastaRecord],
    opts: &OptimizeOptions,
) -> Result<(), OptimizeError> {
    for rec in records.iter_mut() {
        rec.sequence = optimize_cds(&rec.sequence, opts)?;
        rec.id = format!("{}_opt", rec.id);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_record() {
        let records = parse_fasta(">gene1\nATGC\nGGCC\n");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "gene1");
        assert_eq!(records[0].sequence, "ATGCGGCC");
    }

    #[test]
    fn round_trip_format() {
        let records = vec![
            FastaRecord {
                id: "a".into(),
                sequence: "ATG".into(),
            },
            FastaRecord {
                id: "b".into(),
                sequence: "TAA".into(),
            },
        ];
        let parsed = parse_fasta(&format_fasta(&records));
        assert_eq!(parsed, records);
    }
}
