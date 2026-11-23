use anyhow::Result;
use clap::{Parser, Subcommand};
use jensen_anything::{report, Gate, Run, SATIRE_LINE};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "jensen-anything",
    version,
    about = "Pivot any company's story to AI infrastructure and prove the fundamentals did not change.",
    after_help = "SATIRE, NOT FINANCIAL ADVICE. Unaffiliated and unendorsed."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Copy the pitch and financials byte for byte and lock their hashes
    Record(Inputs),
    /// Rewrite the story around AI infrastructure; leave the numbers alone
    Pivot { dir: PathBuf },
    /// Prove the numbers did not change while the story did
    Check { dir: PathBuf },
    /// Write the rescue report and card, if every check passes
    Lift { dir: PathBuf },
    /// Record, pivot, check and lift in one go
    Run(Inputs),
}

#[derive(clap::Args)]
struct Inputs {
    /// Plain-text or Markdown pitch or README
    pitch: PathBuf,
    /// CSV: period,revenue,cost_of_revenue,operating_expenses,enterprise_value
    #[arg(short, long)]
    financials: PathBuf,
    /// Directory for this rescue
    #[arg(short, long, default_value = "jensen-run")]
    out: PathBuf,
    /// Replace an existing recording in the output directory
    #[arg(long)]
    force: bool,
}

fn main() -> ExitCode {
    match execute(Cli::parse().command) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn execute(command: Command) -> Result<ExitCode> {
    match command {
        Command::Record(inputs) => {
            record(&inputs)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Pivot { dir } => {
            pivot(&Run::new(dir))?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Check { dir } => check(&Run::new(dir)),
        Command::Lift { dir } => {
            lift(&Run::new(dir))?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Run(inputs) => {
            let run = record(&inputs)?;
            pivot(&run)?;
            let code = check(&run)?;
            if code == ExitCode::SUCCESS {
                lift(&run)?;
            }
            Ok(code)
        }
    }
}

fn record(inputs: &Inputs) -> Result<Run> {
    let run = Run::new(&inputs.out);
    run.record(&inputs.pitch, &inputs.financials, inputs.force)?;
    println!("record  locked inputs in {}", display(&inputs.out));
    Ok(run)
}

fn pivot(run: &Run) -> Result<()> {
    let replacements = run.pivot()?;
    println!("pivot   {replacements} phrases moved to AI infrastructure");
    Ok(())
}

fn check(run: &Run) -> Result<ExitCode> {
    let gates = run.check()?;
    for Gate {
        name,
        passed,
        detail,
    } in &gates
    {
        let mark = if *passed { "pass" } else { "FAIL" };
        println!("check   {mark}  {name} ({detail})");
    }
    if gates.iter().all(|gate| gate.passed) {
        Ok(ExitCode::SUCCESS)
    } else {
        eprintln!("check failed: nothing was lifted. {SATIRE_LINE}.");
        Ok(ExitCode::FAILURE)
    }
}

fn lift(run: &Run) -> Result<()> {
    let rescue = run.lift()?;
    println!("lift    wrote {}", display(&run.report_path()));
    println!("lift    wrote {}\n", display(&run.card_path()));
    print!("{}", report::terminal(&rescue));
    Ok(())
}

fn display(path: &Path) -> String {
    path.display().to_string()
}
