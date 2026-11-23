//! Record a company's story and numbers, pivot the story to AI infrastructure,
//! prove the numbers did not move, and lift a satirical valuation report.

pub mod financials;
pub mod pivot;
pub mod report;
pub mod valuation;

use anyhow::{bail, Context, Result};
use financials::Financials;
use pivot::Density;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use valuation::Valuation;

pub const SATIRE_LINE: &str = "SATIRE, NOT FINANCIAL ADVICE";
const LOCK_FORMAT: &str = "jensen-lock/1";

/// The files of one rescue, all inside a single directory.
#[derive(Debug, Clone)]
pub struct Run {
    root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub name: &'static str,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rescue {
    pub company: String,
    pub valuation: Valuation,
    pub replacements: u64,
    pub financials_sha256: String,
    pub financials_bytes: usize,
    pub gates: Vec<Gate>,
}

struct Lock {
    pitch_sha256: String,
    financials_sha256: String,
}

impl Run {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub fn report_path(&self) -> PathBuf {
        self.path("REPORT.md")
    }

    pub fn card_path(&self) -> PathBuf {
        self.path("card.svg")
    }

    /// Step 1: copy the inputs byte for byte and lock their hashes.
    pub fn record(&self, pitch: &Path, financials: &Path, force: bool) -> Result<()> {
        let pitch_bytes =
            fs::read(pitch).with_context(|| format!("cannot read {}", pitch.display()))?;
        let csv_bytes = fs::read(financials)
            .with_context(|| format!("cannot read {}", financials.display()))?;
        std::str::from_utf8(&pitch_bytes).context("pitch is not valid UTF-8")?;
        Financials::parse(&csv_bytes)?;

        if self.path("record.lock").exists() && !force {
            bail!(
                "{} already holds a recording; pass --force to replace it",
                self.root.display()
            );
        }
        for stale in [
            "record.lock",
            "pivoted/pitch.md",
            "pivoted/financials.csv",
            "REPORT.md",
            "card.svg",
        ] {
            match fs::remove_file(self.path(stale)) {
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                    return Err(error).with_context(|| format!("cannot remove {stale}"));
                }
                _ => {}
            }
        }

        fs::create_dir_all(self.path("original"))
            .with_context(|| format!("cannot create {}", self.root.display()))?;
        fs::write(self.path("original/pitch.md"), &pitch_bytes)?;
        fs::write(self.path("original/financials.csv"), &csv_bytes)?;
        let lock = format!(
            "format={LOCK_FORMAT}\npitch_sha256={}\nfinancials_sha256={}\n",
            sha256(&pitch_bytes),
            sha256(&csv_bytes)
        );
        fs::write(self.path("record.lock"), lock)?;
        Ok(())
    }

    /// Step 2: rewrite the story and copy the financials untouched.
    pub fn pivot(&self) -> Result<u64> {
        self.lock()?;
        let original = self.read_text("original/pitch.md")?;
        let pivoted = pivot::pivot(&original);
        fs::create_dir_all(self.path("pivoted"))?;
        fs::write(self.path("pivoted/pitch.md"), &pivoted.text)?;
        fs::copy(
            self.path("original/financials.csv"),
            self.path("pivoted/financials.csv"),
        )?;
        Ok(pivoted.replacements)
    }

    /// Step 3: run every gate. Never stops early, so a failure shows them all.
    pub fn check(&self) -> Result<Vec<Gate>> {
        let lock = self.lock()?;
        let pitch = self.read_bytes("original/pitch.md")?;
        let csv = self.read_bytes("original/financials.csv")?;
        let pivoted_pitch = self.read_bytes("pivoted/pitch.md")?;
        let pivoted_csv = self.read_bytes("pivoted/financials.csv")?;

        let original_text = String::from_utf8_lossy(&pitch);
        let pivoted_text = String::from_utf8_lossy(&pivoted_pitch);
        let before = pivot::numbers(&original_text);
        let after = pivot::numbers(&pivoted_text);
        let expected = pivot::pivot(&original_text);

        Ok(vec![
            gate(
                "recorded pitch intact",
                sha256(&pitch) == lock.pitch_sha256,
                format!("{} bytes", pitch.len()),
            ),
            gate(
                "recorded financials intact",
                sha256(&csv) == lock.financials_sha256,
                format!("{} bytes", csv.len()),
            ),
            gate(
                "financials byte-identical",
                csv == pivoted_csv,
                format!("{} differing bytes", differing_bytes(&csv, &pivoted_csv)),
            ),
            gate(
                "narrative numbers unchanged",
                before == after,
                format!("{} of {} in order", matching(&before, &after), before.len()),
            ),
            gate(
                "narrative actually changed",
                pitch != pivoted_pitch,
                format!("{} phrases pivoted", expected.replacements),
            ),
            gate(
                "pivot reproducible",
                expected.text.as_bytes() == pivoted_pitch,
                "re-derived from the recording".to_string(),
            ),
        ])
    }

    /// Step 4: refuse unless every gate passes, then write the report and card.
    pub fn lift(&self) -> Result<Rescue> {
        let gates = self.check()?;
        if let Some(failed) = gates.iter().find(|gate| !gate.passed) {
            bail!("check failed at {:?}: {}", failed.name, failed.detail);
        }
        let original = self.read_text("original/pitch.md")?;
        let pivoted = pivot::pivot(&original);
        let csv = self.read_bytes("pivoted/financials.csv")?;
        let financials = Financials::parse(&csv)?;
        let valuation = Valuation::compute(
            &financials,
            Density::of(&original),
            Density::of(&pivoted.text),
        );
        let rescue = Rescue {
            company: report::company_name(&original),
            valuation,
            replacements: pivoted.replacements,
            financials_sha256: sha256(&csv),
            financials_bytes: csv.len(),
            gates,
        };
        fs::write(self.report_path(), report::markdown(&rescue))?;
        fs::write(self.card_path(), report::svg(&rescue))?;
        Ok(rescue)
    }

    fn lock(&self) -> Result<Lock> {
        let path = self.path("record.lock");
        let text = fs::read_to_string(&path)
            .with_context(|| format!("no recording at {}; run `record` first", path.display()))?;
        let field = |key: &str| {
            text.lines()
                .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
                .map(str::to_string)
                .with_context(|| format!("record.lock is missing {key}"))
        };
        if field("format")? != LOCK_FORMAT {
            bail!("record.lock has an unknown format");
        }
        Ok(Lock {
            pitch_sha256: field("pitch_sha256")?,
            financials_sha256: field("financials_sha256")?,
        })
    }

    fn read_bytes(&self, relative: &str) -> Result<Vec<u8>> {
        fs::read(self.path(relative)).with_context(|| {
            format!(
                "missing {relative} in {}; run the earlier steps",
                self.root.display()
            )
        })
    }

    fn read_text(&self, relative: &str) -> Result<String> {
        String::from_utf8(self.read_bytes(relative)?)
            .with_context(|| format!("{relative} is not valid UTF-8"))
    }
}

fn gate(name: &'static str, passed: bool, detail: String) -> Gate {
    Gate {
        name,
        passed,
        detail,
    }
}

fn differing_bytes(left: &[u8], right: &[u8]) -> usize {
    let shared = left.iter().zip(right).filter(|(a, b)| a != b).count();
    shared + left.len().abs_diff(right.len())
}

fn matching(left: &[&str], right: &[&str]) -> usize {
    left.iter().zip(right).filter(|(a, b)| a == b).count()
}

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
