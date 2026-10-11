//! [bloc](https://github.com/wetware-hq/bloc) design specification (`spec_version` `0.1.0`).

use crate::seek_frame::{FrameKind, SeekFrame};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// bloc `chassis` enum (`schemas/design_spec.schema.json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlocChassis {
    None,
    #[serde(rename = "e_coli_k12")]
    EColiK12,
    #[serde(rename = "e_coli_b")]
    EColiB,
    Yeast,
    CellFree,
    OtherBsl1,
    Other,
}

/// bloc `intended_function` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntendedFunction {
    Reporter,
    Housekeeping,
    Metabolic,
    Binding,
    Nuclease,
    Polymerase,
    ReverseTranscriptase,
    ProgrammableNucleaseSystem,
    Unknown,
}

/// bloc `intended_bsl` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntendedBsl {
    #[serde(rename = "BSL-1")]
    Bsl1,
    #[serde(rename = "BSL-2")]
    Bsl2,
    #[serde(rename = "BSL-3")]
    Bsl3,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentRole {
    Cds,
    Promoter,
    Rbs,
    Terminator,
    RepeatArray,
    Spacer,
    Oligo,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FragmentAlphabet {
    Dna,
    Rna,
    Aa,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FragmentCoords {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignFragment {
    pub id: String,
    pub role: FragmentRole,
    pub alphabet: FragmentAlphabet,
    pub sequence: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coords: Option<FragmentCoords>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Designer {
    pub agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub human: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignSpec {
    pub spec_version: String,
    pub construct_id: String,
    pub designer: Designer,
    pub chassis: BlocChassis,
    pub intended_function: IntendedFunction,
    pub intended_bsl: IntendedBsl,
    pub not_for_synthesis: bool,
    pub fragments: Vec<DesignFragment>,
}

/// Options for turning an optimised Seek frame into a bloc design specification.
#[derive(Debug, Clone)]
pub struct DesignSpecOptions {
    pub designer_agent: String,
    pub designer_human: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub chassis: BlocChassis,
    pub intended_function: IntendedFunction,
    pub intended_bsl: IntendedBsl,
    /// When `true`, bloc screen will not issue a synthesis clearance.
    pub not_for_synthesis: bool,
}

impl Default for DesignSpecOptions {
    fn default() -> Self {
        Self {
            designer_agent: "synth".into(),
            designer_human: "wetware".into(),
            model: None,
            session_id: None,
            chassis: BlocChassis::EColiK12,
            intended_function: IntendedFunction::Reporter,
            intended_bsl: IntendedBsl::Bsl1,
            not_for_synthesis: true,
        }
    }
}

/// Errors building a design specification from a Seek frame.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DesignSpecError {
    #[error("design_spec requires a DNA frame (got {0})")]
    NotDna(FrameKind),
    #[error("empty sequence on frame `{0}`")]
    EmptySequence(String),
    #[error("no fragments could be derived from frame features")]
    NoFragments,
}

/// Map a host / chassis string to bloc `chassis` (`schemas/design_spec.schema.json`).
///
/// `human` / `h_sapiens` are not bloc chassis enum values; they map to [`BlocChassis::Other`].
pub fn resolve_bloc_chassis(host: &str) -> BlocChassis {
    let key = host.to_lowercase().replace([' ', '-'], "_");
    match key.as_str() {
        "none" => BlocChassis::None,
        "e_coli_k12" | "ecoli" | "e_coli" => BlocChassis::EColiK12,
        "e_coli_b" => BlocChassis::EColiB,
        "yeast" | "s_cerevisiae" => BlocChassis::Yeast,
        "cell_free" => BlocChassis::CellFree,
        "other_bsl1" => BlocChassis::OtherBsl1,
        "other" | "human" | "h_sapiens" => BlocChassis::Other,
        _ => BlocChassis::Other,
    }
}

/// Build a bloc design specification from an optimised Seek DNA frame.
///
/// CDS features (type `CDS`) become `fragments[]` entries; if none are present the full
/// `seq` is emitted as a single `cds` / `dna` fragment (construct id = frame id).
pub fn design_spec_from_seek_frame(
    frame: &SeekFrame,
    opts: &DesignSpecOptions,
) -> Result<DesignSpec, DesignSpecError> {
    if frame.kind != FrameKind::Dna {
        return Err(DesignSpecError::NotDna(frame.kind));
    }
    if frame.seq.is_empty() {
        return Err(DesignSpecError::EmptySequence(frame.id.clone()));
    }

    let fragments = fragments_from_features(frame)?;
    Ok(DesignSpec {
        spec_version: "0.1.0".into(),
        construct_id: frame.id.clone(),
        designer: Designer {
            agent: opts.designer_agent.clone(),
            model: opts.model.clone(),
            human: opts.designer_human.clone(),
            session_id: opts.session_id.clone(),
        },
        chassis: opts.chassis,
        intended_function: opts.intended_function,
        intended_bsl: opts.intended_bsl,
        not_for_synthesis: opts.not_for_synthesis,
        fragments,
    })
}

fn fragments_from_features(frame: &SeekFrame) -> Result<Vec<DesignFragment>, DesignSpecError> {
    let mut out = Vec::new();
    for (idx, feat) in frame.features.iter().enumerate() {
        if feat.get("type").and_then(|t| t.as_str()) != Some("CDS") {
            continue;
        }
        let id = feat
            .get("ref")
            .and_then(|r| r.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| format!("cds{}", idx + 1));
        let coords = match (feat.get("start"), feat.get("end")) {
            (Some(s), Some(e)) => {
                let start = s.as_u64().unwrap_or(1) as u32;
                let end = e.as_u64().unwrap_or(frame.seq.len() as u64) as u32;
                Some(FragmentCoords { start, end })
            }
            _ => None,
        };
        let sequence = if let Some(c) = &coords {
            let start = (c.start as usize).saturating_sub(1);
            let end = c.end as usize;
            frame
                .seq
                .get(start..end.min(frame.seq.len()))
                .unwrap_or(&frame.seq)
                .to_string()
        } else {
            frame.seq.clone()
        };
        out.push(DesignFragment {
            id,
            role: FragmentRole::Cds,
            alphabet: FragmentAlphabet::Dna,
            sequence,
            coords,
        });
    }
    if out.is_empty() {
        out.push(DesignFragment {
            id: format!("{}_cds", frame.id),
            role: FragmentRole::Cds,
            alphabet: FragmentAlphabet::Dna,
            sequence: frame.seq.clone(),
            coords: None,
        });
    }
    if out.is_empty() {
        return Err(DesignSpecError::NoFragments);
    }
    Ok(out)
}

/// Serialise a design specification to pretty JSON with trailing newline.
pub fn to_design_spec_json(spec: &DesignSpec) -> Result<String, serde_json::Error> {
    Ok(serde_json::to_string_pretty(spec)? + "\n")
}
