# synth card

```
ENV: molecular cloning bench, terminal only; two stations
  cloning/  seven python commands over a plain-text sequence record (Biopython, primer3-py, pydna)
  src/      synth binary: codon-opt CDS for synthesis, hands DesignSpec to bloc
SCOPE: design specifications only, made-up inputs, research not treatment; NO ordering/synthesis path
```

## invariants (never break)

```
text in, text out; no GUI, no interactive editor
no network at run time; no on-disk cache; codon tables compiled in
deterministic: pinned deps, fixed record dates, sorted JSON keys, checked-in examples
  two demo.sh runs byte-identical and equal to demo_transcript.txt
no jurisdiction/site value in core code -> site-profile data file, cited
every command prints the scope banner in its header
```

## cloning station

```
PYTHONPATH=cloning; run from cloning/; venv at cloning/.venv
inspect  record.gb|fa                              -> length, topology, features
annotate record --features TSV --out gb            -> annotated GenBank
orfs     record --min-aa N --out json              -> frames + translations
digest   record --enzymes A,B,C --out json         -> cuts + fragments
primers  insert --vector gb --enzyme E --homology N -> primer pair (+Gibson tails)
assemble --template --primers --vector --enzyme    -> simulated PCR joined into vector
spec     construct.gb --json-out json              -> construct spec + plain-words summary
tests: cd cloning && python -m pytest   (29, incl. failure paths and two re-run checks)
```

## adding a command

```
one file in cloning/synthlab/, one job; wrap a mature library, do not reimplement biology
argparse + shared header/banner from common.py
plain-text stdout; optional --out / --json-out (sorted keys, no clock, no random)
wrong result = error, not a guess: validate inputs, fail loud with the reason
add tests in cloning/tests/test_commands.py: happy path + each failure path + re-run determinism
if it is a release gate, implement the SY-GATE-* check id, do not fold it into another command
```

## gates NOT built

```
SY-GATE-HAZARD   screen of the sequence before export
SY-GATE-HUMANREF target vs normal human reference
SY-GATE-INTEGRITY round-trip/frame/reproducibility as release condition
SY-GATE-IDENTITY two-identifier match person+specimen; signed sequencing/tissue-type artefacts
SY-GATE-SIGNOFF  two named signatures, consent sheet, plain-words summary beside every card
nothing leaving this bench is signed off until these exist
```

## codon station

```
ROLE: codon-opt CDS for synthesis; NOT release authority (bloc owns RELEASE|HOLD|ESCALATE)
PIPE: SeekFrame(phase=filled,kind=DNA) → synth → SeekFrame + optional DesignSpec → bloc screen → BlocVerdict → enforce_synthesis_gate?
```

## invoke_when

```
DNA SeekFrame.phase == filled
seq.nonempty && len(normalise(seq)) % 3 == 0
need host-tuned codons OR Type IIS motif avoidance OR GC band nudge
bloc handoff needed → Seek path (not FASTA-only)
```

## do_not_invoke_when

```
kind ∈ {RNA, AA, XNA}  → seek.filler / Seek tools first
phase != filled || seq empty
need safety verdict → run bloc screen; never infer from synth_optimised feature
de novo AA design → unsupported
```

## input

```
SeekFrame JSON (deny_unknown_fields root):
  id, kind: DNA|RNA|AA|XNA, topo: linear|circular, seq, features[], want[], forbid[], phase: spec|filled|kill|tube
CLI JSON detect: ext .json|.seek OR body starts with '{'
FASTA: >id lines; CLI only; no --design-spec on FASTA path

host string → resolve_host (codons), resolve_bloc_chassis (DesignSpec.chassis)
  e_coli_k12|ecoli|e_coli → e_coli / e_coli_k12
  e_coli_b → e_coli / e_coli_b
  yeast|s_cerevisiae → s_cerevisiae / yeast
  cell_free → e_coli / cell_free
  other_bsl1 → e_coli / other_bsl1
  none → e_coli / none
  human|h_sapiens → h_sapiens / other
  other → e_coli / other
  else → resolve_host: lowercased key returned as-is; synonymous_codons uses E. coli table unless key is s_cerevisiae|h_sapiens / other

OptimizeOptions:
  host: String (resolved codon key)
  avoid: Vec<String> default GGTCTC,GAGACC,CGTCTC,GAGACG
  gc_min: f64 default 0.3
  gc_max: f64 default 0.7
```

## codon_tables

```
source: FDA HIVE CoCoPUTs (HIVE-CUTs) — compile-time src/codon_tables.rs; NO runtime fetch/cache
vendored: data/cocoputs/*.tsv retrieved 2026-10-11; genomic RefSeq counts; per-aa rel_freq sums to 1
regen: python3 data/cocoputs/generate_tables.py
host key → taxid / file:
  e_coli* → 511145 taxid_511145_ecoli_k12_mg1655_genomic.tsv
  s_cerevisiae|yeast → 4932 taxid_4932_s_cerevisiae_genomic.tsv
  h_sapiens|human → 9606 taxid_9606_h_sapiens_genomic.tsv
refresh: prefer CoCoPUTs bulk downloads; CGI ionTaxidCollapseExt fallback (see data/cocoputs/README.md)
if CoCoPUTs unavailable: Edinburgh-Genome-Foundry/codon-usage-tables (Kazusa/GenBank 160.0, 2007) — document clearly
synonymous_codons → &'static [CodonUsage] (freq-ordered); preferred_codon = first entry
```

## algorithm

```
normalise: strip ws; upper; U→T
per-codon: synonymous_codons(aa,host) preferred-first; pick first where prefix+ codon ∉ avoid substrings
  fail → OptimizeError::PatternConflict(offset)
  bad codon → OptimizeError::UnknownCodon(offset)
  len%3!=0 → OptimizeError::BadLength
GC: if gc∉[gc_min,gc_max] single pass codon swaps toward band; else return step-1 seq (may stay out of band)
translation preserved (table 1)
```

## output

```
SeekFrame: seq replaced; phase filled; features += {
  type: "synth_optimised",
  host: <display_host>,
  species: <OptimizeOptions.host>,
  gc: <float>
}
to_seek_frame_json → pretty JSON +\n

DesignSpec (spec_version "0.1.0"):
  construct_id = frame.id
  fragments: each features[].type==CDS (slice start..end 1-based) OR one cds/dna full seq
CLI --design-spec defaults:
  designer.agent=synth designer.human=wetware
  intended_function=reporter intended_bsl=BSL-1 not_for_synthesis=true
  chassis=resolve_bloc_chassis(--host)

FASTA: ids → {id}_opt; format_fasta
```

## library API (order)

```
parse_seek_frame(str) → SeekFrame | serde_json::Error
validate_frame_for_optimisation(&SeekFrame) → Result<(), FrameError>
optimize_seek_frame(SeekFrame, display_host: &str, &OptimizeOptions) → FrameOptimizeResult | FrameError
optimize_and_handoff(SeekFrame, &OptimizeRequest) → SynthHandoff | PipelineError
  OptimizeRequest { display_host, optimize: OptimizeOptions, design_spec: DesignSpecOptions }
parse_fasta → optimize_fasta_records → format_fasta
design_spec_from_seek_frame / to_design_spec_json
parse_bloc_verdict → enforce_synthesis_gate / gate_synthesis
helpers: optimize_cds, gc_content, translate_codon, preferred_codon, synonymous_codons, codon_relative_frequency, CodonUsage, default_avoid_patterns
crate feature cli (default): binary synth; default-features=false → lib only
```

## errors

```
OptimizeError::BadLength | UnknownCodon(usize) | PatternConflict(usize)
FrameError::NotDna(FrameKind) | WrongPhase(FramePhase) | EmptySequence | Optimize(...)
DesignSpecError::NotDna | EmptySequence | NoFragments
SynthesisGateError { policy: BlocPolicy, construct_id }  // HOLD|ESCALATE only
serde_json::Error on parse
```

## CLI

```
synth <input> [-o PATH] [--host HOST] [--avoid PAT]... [--gc-min F] [--gc-max F]
  [--design-spec PATH] [--verdict PATH] [--designer-human S] [--designer-agent S]
defaults: host=e_coli_k12 gc_min=0.3 gc_max=0.7 designer_human=wetware designer_agent=synth
output default: <stem>_opt.json | <stem>_opt.fasta
```

## exit_codes

```
0  ok
1  io | seek json parse | empty fasta | fasta optimise | verdict read/parse
2  frame validate | frame optimise (stderr has FrameError|PipelineError)
10 --verdict & BlocPolicy::Hold     (matches bloc; SynthesisGateError.exit_code)
20 --verdict & BlocPolicy::Escalate
enforce_synthesis_gate: Ok only BlocPolicy::Release
```

## pipeline_rules

```
1 SeekFrame required for DesignSpec handoff
2 after synth → bloc screen on DesignSpec before synthesis/order
3 if BlocVerdict file exists → enforce_synthesis_gate before proceed; HOLD/ESCALATE → stop
4 synth_optimised + optimised seq ≠ safety clearance
```

## do_not_assume

```
GC always ∈ [0.30,0.70] after opt (best-effort single pass)
CLI not_for_synthesis=true ⇒ cleared for ordering (false)
synth runs bloc or emits BlocVerdict
human → bloc chassis enum (maps Other)
FASTA alone satisfies bloc path
avoid list removes all restriction sites globally
```
