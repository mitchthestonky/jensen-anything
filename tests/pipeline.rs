use jensen_anything::{sha256, Run, SATIRE_LINE};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const CSV: &str =
    "period,revenue,cost_of_revenue,operating_expenses,enterprise_value\nQ1,100,40,30,1000\n";
const PITCH: &str = "# Test Co\n\nOur customers love our product in 2024.\n";

fn bin(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jensen-anything"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn inputs(dir: &TempDir, pitch: &str, csv: &str) -> (PathBuf, PathBuf, PathBuf) {
    let p = dir.path().join("pitch.md");
    let c = dir.path().join("financials.csv");
    fs::write(&p, pitch).unwrap();
    fs::write(&c, csv).unwrap();
    (p, c, dir.path().join("run"))
}

fn run_all(p: &Path, c: &Path, out: &Path) -> Output {
    bin(&[
        "run",
        p.to_str().unwrap(),
        "-f",
        c.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ])
}

fn rescued() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let (p, c, out) = inputs(&dir, PITCH, CSV);
    assert!(run_all(&p, &c, &out).status.success());
    (dir, out)
}

fn assert_check_fails(out: &Path, gate: &str) {
    let output = bin(&["check", out.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stdout(&output).contains(&format!("FAIL  {gate}")),
        "{}",
        stdout(&output)
    );
    assert!(!bin(&["lift", out.to_str().unwrap()]).status.success());
}

#[test]
fn arithmetic_matches_a_hand_calculation() {
    // Before: 0 AI terms in 9 words. After: 2 in 10, so 200,000 ppm,
    // 20,000 basis points, and 1,000 * 3 = 3,000.
    let (_dir, out) = rescued();
    let report = fs::read_to_string(out.join("REPORT.md")).unwrap();
    assert!(
        report.contains("| EV / trailing revenue | 10.00x | 30.00x | +200.00% |"),
        "{report}"
    );
    assert!(report.contains("| Implied enterprise value | 1,000 | 3,000 | +2,000 |"));
    assert!(report.contains("| Trailing operating income | 30 | 30 | 0 |"));
}

#[test]
fn financials_stay_byte_identical_and_story_changes() {
    let (_dir, out) = rescued();
    assert_eq!(
        fs::read(out.join("pivoted/financials.csv")).unwrap(),
        CSV.as_bytes()
    );
    let story = fs::read_to_string(out.join("pivoted/pitch.md")).unwrap();
    assert_eq!(
        story,
        "# Test Co\n\nOur inference endpoints love our model in 2024.\n"
    );
    let report = fs::read_to_string(out.join("REPORT.md")).unwrap();
    assert!(report.contains(&sha256(CSV.as_bytes())));
}

#[test]
fn every_output_carries_the_satire_line() {
    let dir = TempDir::new().unwrap();
    let (p, c, out) = inputs(&dir, PITCH, CSV);
    let output = run_all(&p, &c, &out);
    assert!(stdout(&output).contains(SATIRE_LINE));
    let report = fs::read_to_string(out.join("REPORT.md")).unwrap();
    assert!(report.starts_with(&format!("> **{SATIRE_LINE}.**")));
    assert!(report.trim_end().ends_with(&format!("**{SATIRE_LINE}.**")));
    assert!(fs::read_to_string(out.join("card.svg"))
        .unwrap()
        .contains(SATIRE_LINE));
}

#[test]
fn runs_are_reproducible() {
    let (_a, first) = rescued();
    let (_b, second) = rescued();
    for file in ["REPORT.md", "card.svg", "pivoted/pitch.md", "record.lock"] {
        assert_eq!(
            fs::read(first.join(file)).unwrap(),
            fs::read(second.join(file)).unwrap()
        );
    }
}

#[test]
fn one_changed_financial_byte_fails() {
    let (_dir, out) = rescued();
    fs::write(
        out.join("pivoted/financials.csv"),
        CSV.replace("100,", "101,"),
    )
    .unwrap();
    assert_check_fails(&out, "financials byte-identical");
}

#[test]
fn a_changed_narrative_number_fails() {
    let (_dir, out) = rescued();
    let path = out.join("pivoted/pitch.md");
    let story = fs::read_to_string(&path).unwrap().replace("2024", "2025");
    fs::write(path, story).unwrap();
    assert_check_fails(&out, "narrative numbers unchanged");
}

#[test]
fn a_hand_edited_story_fails() {
    let (_dir, out) = rescued();
    let path = out.join("pivoted/pitch.md");
    let story = fs::read_to_string(&path).unwrap().replace("love", "adore");
    fs::write(path, story).unwrap();
    assert_check_fails(&out, "pivot reproducible");
}

#[test]
fn a_tampered_recording_fails() {
    let (_dir, out) = rescued();
    fs::write(
        out.join("original/financials.csv"),
        CSV.replace("1000", "9000"),
    )
    .unwrap();
    assert_check_fails(&out, "recorded financials intact");
}

#[test]
fn a_story_with_nothing_to_pivot_is_not_lifted() {
    let dir = TempDir::new().unwrap();
    let (p, c, out) = inputs(&dir, "# Zzz\n\nHello there.\n", CSV);
    let output = run_all(&p, &c, &out);
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("FAIL  narrative actually changed"));
    assert!(!out.join("REPORT.md").exists());
}

#[test]
fn pre_revenue_has_no_multiple_and_does_not_crash() {
    let dir = TempDir::new().unwrap();
    let csv = "period,revenue,cost_of_revenue,operating_expenses,enterprise_value\nQ1,0,0,5,1000\n";
    let (p, c, out) = inputs(&dir, PITCH, csv);
    assert!(run_all(&p, &c, &out).status.success());
    let report = fs::read_to_string(out.join("REPORT.md")).unwrap();
    assert!(report.contains("n/a (pre-revenue)"));
    assert!(report.contains("| Trailing operating income | -5 | -5 | 0 |"));
}

#[test]
fn bad_input_is_rejected_before_writing() {
    let header = "period,revenue,cost_of_revenue,operating_expenses,enterprise_value\n";
    for csv in [
        "nope\n".to_string(),
        header.to_string(),
        format!("{header}Q1,1.5,0,0,10\n"),
        format!("{header}Q1,-1,0,0,10\n"),
        format!("{header}Q1,1,0,0\n"),
        format!("{header}Q1,1,0,0,0\n"),
        format!("{header}Q1,1,0,0,10\nQ1,1,0,0,10\n"),
        format!("{header}Q1,99999999999999999999,0,0,10\n"),
    ] {
        let dir = TempDir::new().unwrap();
        let (p, c, out) = inputs(&dir, PITCH, &csv);
        assert_eq!(run_all(&p, &c, &out).status.code(), Some(2), "{csv}");
        assert!(!out.exists());
    }
}

#[test]
fn an_existing_recording_needs_force_and_clears_stale_output() {
    let dir = TempDir::new().unwrap();
    let (p, c, out) = inputs(&dir, PITCH, CSV);
    assert!(run_all(&p, &c, &out).status.success());
    assert_eq!(run_all(&p, &c, &out).status.code(), Some(2));
    let run = Run::new(&out);
    run.record(&p, &c, true).unwrap();
    assert!(!out.join("REPORT.md").exists());
    assert!(!out.join("pivoted/pitch.md").exists());
}

#[test]
fn hostile_company_names_are_escaped() {
    let dir = TempDir::new().unwrap();
    let pitch = "# <script>x</script> \u{1b}[31m|Co\n\nOur customers.\n";
    let (p, c, out) = inputs(&dir, pitch, CSV);
    let output = run_all(&p, &c, &out);
    assert!(output.status.success());
    assert!(!stdout(&output).contains('\u{1b}'));
    assert!(!fs::read_to_string(out.join("card.svg"))
        .unwrap()
        .contains("<script>"));
    assert!(!fs::read_to_string(out.join("REPORT.md"))
        .unwrap()
        .contains("<script>"));
}

#[test]
fn every_fixture_rescues_with_zero_changed_financial_bytes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let mut count = 0;
    for entry in fs::read_dir(root).unwrap() {
        let company = entry.unwrap().path();
        let dir = TempDir::new().unwrap();
        let out = dir.path().join("run");
        let csv = company.join("financials.csv");
        let output = run_all(&company.join("pitch.md"), &csv, &out);
        assert!(output.status.success(), "{}", company.display());
        assert_eq!(
            fs::read(&csv).unwrap(),
            fs::read(out.join("pivoted/financials.csv")).unwrap()
        );
        count += 1;
    }
    assert_eq!(count, 5);
}

#[test]
fn readme_results_match_the_tool() {
    use jensen_anything::valuation::{money, multiple};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    for entry in fs::read_dir(root.join("fixtures")).unwrap() {
        let company = entry.unwrap().path();
        let dir = TempDir::new().unwrap();
        let run = Run::new(dir.path().join("run"));
        run.record(
            &company.join("pitch.md"),
            &company.join("financials.csv"),
            false,
        )
        .unwrap();
        run.pivot().unwrap();
        let rescue = run.lift().unwrap();
        let v = &rescue.valuation;
        let short = |value: Option<i128>| value.map_or("n/a".to_string(), |_| multiple(value));
        let row = format!(
            "| {} | {} | 0 | {} | **{}** | +{} |",
            rescue.company,
            money(v.revenue),
            short(v.multiple_before()),
            short(v.multiple_after()),
            money(v.value_conjured()),
        );
        assert!(readme.contains(&row), "README is missing: {row}");
    }
}
