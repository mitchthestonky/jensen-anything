//! Strict parser for the financials CSV. Whole currency units only, so no
//! floating point ever touches a number.

use anyhow::{bail, Context, Result};

pub const HEADER: &str = "period,revenue,cost_of_revenue,operating_expenses,enterprise_value";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Period {
    pub name: String,
    pub revenue: u64,
    pub cost_of_revenue: u64,
    pub operating_expenses: u64,
    pub enterprise_value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Financials {
    pub periods: Vec<Period>,
}

impl Financials {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let text = std::str::from_utf8(bytes).context("financials CSV is not valid UTF-8")?;
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut lines = text
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty());
        let (_, header) = lines.next().context("financials CSV is empty")?;
        if header.trim() != HEADER {
            bail!("financials CSV header must be exactly: {HEADER}");
        }

        let mut periods: Vec<Period> = Vec::new();
        for (index, line) in lines {
            let number = index + 1;
            let fields: Vec<&str> = line.split(',').map(str::trim).collect();
            let [name, revenue, cost, opex, ev] = fields.as_slice() else {
                bail!("line {number}: expected 5 columns, found {}", fields.len());
            };
            if name.is_empty() {
                bail!("line {number}: period name is empty");
            }
            if periods.iter().any(|period| period.name == *name) {
                bail!("line {number}: duplicate period {name:?}");
            }
            periods.push(Period {
                name: (*name).to_string(),
                revenue: amount(revenue, "revenue", number)?,
                cost_of_revenue: amount(cost, "cost_of_revenue", number)?,
                operating_expenses: amount(opex, "operating_expenses", number)?,
                enterprise_value: amount(ev, "enterprise_value", number)?,
            });
        }

        let Some(last) = periods.last() else {
            bail!("financials CSV has no data rows");
        };
        if last.enterprise_value == 0 {
            bail!("the latest period needs a positive enterprise_value");
        }
        Ok(Self { periods })
    }

    /// The most recent periods used for trailing totals: up to four rows.
    pub fn trailing(&self) -> &[Period] {
        &self.periods[self.periods.len().saturating_sub(4)..]
    }

    pub fn current_enterprise_value(&self) -> u64 {
        self.periods
            .last()
            .map_or(0, |period| period.enterprise_value)
    }
}

fn amount(raw: &str, column: &str, line: usize) -> Result<u64> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("line {line}: {column} must be a whole non-negative number, got {raw:?}");
    }
    raw.parse()
        .with_context(|| format!("line {line}: {column} is too large"))
}
