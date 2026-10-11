//! CLI for synth — thin wrapper over the library.

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use synth::{
    DesignSpecOptions, IntendedBsl, IntendedFunction, OptimizeOptions, OptimizeRequest,
    SeekFrame, default_avoid_patterns, enforce_synthesis_gate, format_fasta, optimize_and_handoff,
    optimize_fasta_records, parse_bloc_verdict, parse_fasta, parse_seek_frame, resolve_bloc_chassis,
    resolve_host, to_design_spec_json, to_seek_frame_json,
};

#[derive(Parser, Debug)]
#[command(name = "synth", version, about = "Optimise sequence for synthesis and host")]
struct Args {
    /// Input Seek frame JSON or FASTA
    input: PathBuf,

    /// Output path (defaults to <input>_opt.<ext>)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Target host / chassis (e_coli_k12, yeast, human, …)
    #[arg(long, default_value = "e_coli_k12")]
    host: String,

    /// Patterns to avoid (repeatable). Default: BsaI / BsmBI sites.
    #[arg(long)]
    avoid: Vec<String>,

    /// Minimum GC fraction
    #[arg(long, default_value_t = 0.3)]
    gc_min: f64,

    /// Maximum GC fraction
    #[arg(long, default_value_t = 0.7)]
    gc_max: f64,

    /// Write bloc design_spec JSON (spec_version 0.1.0) for `bloc screen`
    #[arg(long)]
    design_spec: Option<PathBuf>,

    /// bloc screen verdict JSON; when set, enforce RELEASE before success exit
    #[arg(long)]
    verdict: Option<PathBuf>,

    /// Designer.human field on emitted design_spec
    #[arg(long, default_value = "wetware")]
    designer_human: String,

    /// Designer.agent field on emitted design_spec
    #[arg(long, default_value = "synth")]
    designer_agent: String,
}

fn main() {
    if let Err(code) = run() {
        std::process::exit(code);
    }
}

fn run() -> Result<(), i32> {
    let args = Args::parse();
    let text = fs::read_to_string(&args.input).map_err(|e| {
        eprintln!("read input: {e}");
        1
    })?;
    let is_json = args
        .input
        .extension()
        .map(|e| e == "json" || e == "seek")
        .unwrap_or(false)
        || text.trim_start().starts_with('{');

    let species = resolve_host(&args.host);
    let avoid = if args.avoid.is_empty() {
        default_avoid_patterns()
    } else {
        args.avoid
    };
    let opts = OptimizeOptions {
        host: species.clone(),
        avoid,
        gc_min: args.gc_min,
        gc_max: args.gc_max,
    };

    if is_json {
        let frame: SeekFrame = parse_seek_frame(&text).map_err(|e| {
            eprintln!("parse Seek frame: {e}");
            1
        })?;
        let spec_opts = DesignSpecOptions {
            designer_agent: args.designer_agent.clone(),
            designer_human: args.designer_human.clone(),
            model: None,
            session_id: None,
            chassis: resolve_bloc_chassis(&args.host),
            intended_function: IntendedFunction::Reporter,
            intended_bsl: IntendedBsl::Bsl1,
            not_for_synthesis: true,
        };
        let request = OptimizeRequest {
            display_host: args.host.clone(),
            optimize: opts,
            design_spec: spec_opts,
        };
        let handoff = optimize_and_handoff(frame, &request).map_err(|e| {
            eprintln!("{e}");
            2
        })?;

        let out = args.output.unwrap_or_else(|| {
            let mut p = args.input.clone();
            let stem = p.file_stem().unwrap().to_string_lossy();
            p.set_file_name(format!("{}_opt.json", stem));
            p
        });
        fs::write(&out, to_seek_frame_json(&handoff.frame_result.frame).unwrap()).map_err(|e| {
            eprintln!("write: {e}");
            1
        })?;
        println!("Wrote optimised Seek frame to {}", out.display());

        if let Some(spec_path) = args.design_spec {
            fs::write(
                &spec_path,
                to_design_spec_json(&handoff.design_spec).unwrap(),
            )
            .map_err(|e| {
                eprintln!("write design_spec: {e}");
                1
            })?;
            println!("Wrote bloc design_spec to {}", spec_path.display());
        } else {
            println!("Next: bloc design_spec → bloc screen before any synthesis path.");
        }

        println!(
            "GC={:.3}. Route through bloc screen; only RELEASE clears synthesis.",
            handoff.frame_result.gc
        );

        if let Some(verdict_path) = args.verdict {
            let verdict_text = fs::read_to_string(&verdict_path).map_err(|e| {
                eprintln!("read verdict: {e}");
                1
            })?;
            let verdict = parse_bloc_verdict(&verdict_text).map_err(|e| {
                eprintln!("parse verdict: {e}");
                1
            })?;
            if let Err(gate) = enforce_synthesis_gate(&verdict) {
                eprintln!("{gate}");
                return Err(gate.exit_code());
            }
            println!(
                "bloc gate: {} for construct {}",
                verdict.policy.as_str(),
                verdict.construct_id
            );
        }
    } else {
        let mut records = parse_fasta(&text);
        if records.is_empty() {
            eprintln!("no sequences in FASTA");
            return Err(1);
        }
        optimize_fasta_records(&mut records, &opts).map_err(|e| {
            eprintln!("optimise: {e}");
            1
        })?;
        let out = args.output.unwrap_or_else(|| {
            let mut p = args.input.clone();
            let stem = p.file_stem().unwrap().to_string_lossy();
            p.set_file_name(format!("{}_opt.fasta", stem));
            p
        });
        fs::write(&out, format_fasta(&records)).map_err(|e| {
            eprintln!("write: {e}");
            1
        })?;
        println!(
            "Wrote {} optimised sequence(s) to {}",
            records.len(),
            out.display()
        );
        println!("Prefer Seek frames in Wetware sessions. Route through bloc before synthesis.");
    }
    Ok(())
}
