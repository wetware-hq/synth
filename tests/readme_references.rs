//! Structural bar on the README's References section, mirroring the Python test in the
//! sibling repos. A green means only: the README ends with a `## References` section,
//! that section is not empty, and every citation entry (a bullet opening with `- **`)
//! carries a checkable id (DOI, PMID or arXiv). It does not check that a citation is
//! real or that it supports its claim; that is human-verified when the entry is added.

use std::fs;
use std::path::PathBuf;

fn readme() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.md");
    fs::read_to_string(p).expect("README.md is readable")
}

/// The text from the start of the last top-level `## ` header to the end of the file.
fn references_section(text: &str) -> String {
    let mut starts: Vec<usize> = Vec::new();
    if text.starts_with("## ") {
        starts.push(0);
    }
    for (i, _) in text.match_indices("\n## ") {
        starts.push(i + 1);
    }
    let last = *starts.last().expect("README has at least one `## ` header");
    text[last..].to_string()
}

#[test]
fn readme_ends_with_references_header() {
    let text = readme();
    let sec = references_section(&text);
    let first = sec.lines().next().unwrap().trim();
    assert_eq!(
        first, "## References",
        "the README's last `## ` section must be `## References`, found: {first}"
    );
}

#[test]
fn references_section_not_empty() {
    let text = readme();
    let sec = references_section(&text);
    let body: String = sec
        .lines()
        .skip(1)
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !body.trim().is_empty(),
        "the References section has no content below its header"
    );
}

#[test]
fn every_citation_entry_has_a_checkable_id() {
    let text = readme();
    let sec = references_section(&text);
    let mut missing: Vec<String> = Vec::new();
    for line in sec.lines() {
        if line.starts_with("- **") {
            let lower = line.to_lowercase();
            let has_doi = line.contains("10.") && line.contains('/');
            let has_pmid = lower.contains("pmid");
            let has_arxiv = lower.contains("arxiv:");
            if !(has_doi || has_pmid || has_arxiv) {
                missing.push(line.to_string());
            }
        }
    }
    assert!(
        missing.is_empty(),
        "citation entries without a DOI, PMID or arXiv id: {missing:?}"
    );
}
