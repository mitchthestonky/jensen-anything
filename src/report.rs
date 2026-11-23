//! The three outputs of a rescue: Markdown report, SVG card, terminal card.
//! Each one carries the satire line, so a screenshot cannot pass as analysis.

use crate::valuation::{density, money, multiple, percent};
use crate::{Rescue, SATIRE_LINE};

const NAME_LIMIT: usize = 40;
const CARD_WIDTH: usize = 60;

/// The first Markdown h1, cleaned of control characters and shortened.
pub fn company_name(pitch: &str) -> String {
    let raw = pitch
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .unwrap_or("Unnamed Company");
    let clean: String = raw
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string();
    let clean = if clean.is_empty() {
        "Unnamed Company".to_string()
    } else {
        clean
    };
    if clean.chars().count() > NAME_LIMIT {
        let short: String = clean.chars().take(NAME_LIMIT - 3).collect();
        format!("{}...", short.trim_end())
    } else {
        clean
    }
}

pub fn markdown(rescue: &Rescue) -> String {
    let v = &rescue.valuation;
    let name = escape_markdown(&rescue.company);
    let mut out = format!(
        "> **{SATIRE_LINE}.** This is a joke report from a parody tool. It is not analysis, \
         a valuation, or a recommendation to do anything.\n\n# Valuation Rescue Report: {name}\n\n\
         **Fundamentals changed: 0 bytes. Implied multiple: {} to {}.**\n\n\
         | Measure | Before | After | Change |\n| --- | ---: | ---: | ---: |\n",
        multiple(v.multiple_before()),
        multiple(v.multiple_after()),
    );
    for (label, value) in [
        ("Trailing revenue", v.revenue),
        ("Trailing cost of revenue", v.cost_of_revenue),
        ("Trailing operating expenses", v.operating_expenses),
        ("Trailing operating income", v.operating_income),
    ] {
        let shown = money(value);
        out.push_str(&format!("| {label} | {shown} | {shown} | 0 |\n"));
    }
    out.push_str(&format!(
        "| AI terms per 1,000 words | {} | {} | {} phrases pivoted |\n\
         | EV / trailing revenue | {} | {} | {} |\n\
         | Implied enterprise value | {} | {} | +{} |\n\n",
        density(v.density_before.parts_per_million()),
        density(v.density_after.parts_per_million()),
        rescue.replacements,
        multiple(v.multiple_before()),
        multiple(v.multiple_after()),
        percent(v.uplift_basis_points),
        money(v.enterprise_value_before),
        money(v.enterprise_value_after),
        money(v.value_conjured()),
    ));
    out.push_str(&format!(
        "## How the number was computed\n\n\
         - Trailing totals sum the last {} period(s) of your CSV, in file order.\n\
         - AI density is AI terms per million words, rounded down: {} of {} words before, \
         {} of {} after.\n\
         - Uplift is one percent per extra AI term per 1,000 words: {}.\n\
         - Implied value is the latest enterprise value times that uplift, rounded half up. \
         Nothing about the business changed. That is the joke.\n\n\
         ## Gates\n\n",
        v.trailing_periods,
        v.density_before.ai_terms,
        v.density_before.words,
        v.density_after.ai_terms,
        v.density_after.words,
        percent(v.uplift_basis_points),
    ));
    for gate in &rescue.gates {
        let mark = if gate.passed { "PASS" } else { "FAIL" };
        out.push_str(&format!("- {mark}: {} ({})\n", gate.name, gate.detail));
    }
    out.push_str(&format!(
        "\nFinancials SHA-256 before and after: `{}` ({} bytes)\n\n**{SATIRE_LINE}.**\n",
        rescue.financials_sha256, rescue.financials_bytes
    ));
    out
}

pub fn svg(rescue: &Rescue) -> String {
    let v = &rescue.valuation;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="720" height="380" viewBox="0 0 720 380" role="img" aria-label="Satirical valuation rescue card for {name}. {satire}.">
<rect width="720" height="380" rx="18" fill="#07110a"/>
<rect x="1.5" y="1.5" width="717" height="377" rx="17" fill="none" stroke="#39ff14" stroke-width="3"/>
<rect x="0" y="0" width="720" height="44" rx="18" fill="#39ff14"/><rect x="0" y="24" width="720" height="20" fill="#39ff14"/>
<text x="360" y="30" text-anchor="middle" font-family="Menlo,Consolas,monospace" font-size="20" font-weight="700" fill="#07110a">{satire}</text>
<text x="36" y="92" font-family="Helvetica,Arial,sans-serif" font-size="30" font-weight="700" fill="#f2fff0">{name}</text>
<text x="36" y="122" font-family="Menlo,Consolas,monospace" font-size="15" fill="#8fbf88">valuation rescue · fundamentals changed: 0 bytes</text>
<text x="36" y="182" font-family="Menlo,Consolas,monospace" font-size="15" fill="#8fbf88">EV / REVENUE</text>
<text x="36" y="226" font-family="Helvetica,Arial,sans-serif" font-size="36" font-weight="700" fill="#f2fff0">{before} <tspan fill="#39ff14">→ {after}</tspan></text>
<text x="36" y="274" font-family="Menlo,Consolas,monospace" font-size="15" fill="#8fbf88">IMPLIED VALUE</text>
<text x="36" y="306" font-family="Helvetica,Arial,sans-serif" font-size="24" font-weight="700" fill="#f2fff0">{ev_before} <tspan fill="#39ff14">→ {ev_after}</tspan></text>
<text x="36" y="352" font-family="Menlo,Consolas,monospace" font-size="13" fill="#8fbf88">Parody output. Not analysis, not advice, not affiliated with anyone.</text>
</svg>
"##,
        name = escape_xml(&rescue.company),
        satire = SATIRE_LINE,
        before = escape_xml(&short_multiple(v.multiple_before())),
        after = escape_xml(&short_multiple(v.multiple_after())),
        ev_before = money(v.enterprise_value_before),
        ev_after = money(v.enterprise_value_after),
    )
}

pub fn terminal(rescue: &Rescue) -> String {
    let v = &rescue.valuation;
    let rows = [
        center(SATIRE_LINE),
        String::new(),
        format!(" {}", rescue.company),
        String::new(),
        row("Fundamentals changed", "0 bytes"),
        row("Gates passed", &format!("{0}/{0}", rescue.gates.len())),
        row("Phrases pivoted", &rescue.replacements.to_string()),
        row("Trailing revenue", &money(v.revenue)),
        row(
            "EV / revenue",
            &format!(
                "{} -> {}",
                short_multiple(v.multiple_before()),
                short_multiple(v.multiple_after())
            ),
        ),
        row("Implied value", &money(v.enterprise_value_after)),
        row("Value conjured", &format!("+{}", money(v.value_conjured()))),
        String::new(),
        center(SATIRE_LINE),
    ];
    let bar = "─".repeat(CARD_WIDTH);
    let mut out = format!("┌{bar}┐\n");
    for line in rows {
        let padding = CARD_WIDTH.saturating_sub(line.chars().count());
        out.push_str(&format!("│{line}{}│\n", " ".repeat(padding)));
    }
    out.push_str(&format!("└{bar}┘\n"));
    out
}

fn row(label: &str, value: &str) -> String {
    let gap = (CARD_WIDTH - 2).saturating_sub(label.chars().count() + value.chars().count());
    format!(" {label}{}{value} ", " ".repeat(gap))
}

fn center(text: &str) -> String {
    let left = CARD_WIDTH.saturating_sub(text.chars().count()) / 2;
    format!("{}{text}", " ".repeat(left))
}

fn short_multiple(value: Option<i128>) -> String {
    match value {
        Some(_) => multiple(value),
        None => "n/a".to_string(),
    }
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if "\\`*_[]<>|#".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
