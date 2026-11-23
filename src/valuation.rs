//! Exact integer valuation arithmetic.

use crate::financials::Financials;
use crate::pivot::Density;

/// One percent of uplift for each additional AI term per thousand words.
const PPM_PER_BASIS_POINT: u64 = 10;
const BASIS: i128 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Valuation {
    pub trailing_periods: usize,
    pub revenue: i128,
    pub cost_of_revenue: i128,
    pub operating_expenses: i128,
    pub operating_income: i128,
    pub enterprise_value_before: i128,
    pub enterprise_value_after: i128,
    pub density_before: Density,
    pub density_after: Density,
    pub uplift_basis_points: u64,
}

impl Valuation {
    pub fn compute(financials: &Financials, before: Density, after: Density) -> Self {
        let trailing = financials.trailing();
        let sum = |pick: fn(&crate::financials::Period) -> u64| -> i128 {
            trailing.iter().map(|p| i128::from(pick(p))).sum()
        };
        let revenue = sum(|p| p.revenue);
        let cost_of_revenue = sum(|p| p.cost_of_revenue);
        let operating_expenses = sum(|p| p.operating_expenses);
        let uplift_basis_points = after
            .parts_per_million()
            .saturating_sub(before.parts_per_million())
            / PPM_PER_BASIS_POINT;
        let enterprise_value_before = i128::from(financials.current_enterprise_value());
        let enterprise_value_after = divide_rounded(
            enterprise_value_before * (BASIS + i128::from(uplift_basis_points)),
            BASIS,
        );
        Self {
            trailing_periods: trailing.len(),
            revenue,
            cost_of_revenue,
            operating_expenses,
            operating_income: revenue - cost_of_revenue - operating_expenses,
            enterprise_value_before,
            enterprise_value_after,
            density_before: before,
            density_after: after,
            uplift_basis_points,
        }
    }

    pub fn multiple_before(&self) -> Option<i128> {
        self.multiple(self.enterprise_value_before)
    }

    pub fn multiple_after(&self) -> Option<i128> {
        self.multiple(self.enterprise_value_after)
    }

    pub fn value_conjured(&self) -> i128 {
        self.enterprise_value_after - self.enterprise_value_before
    }

    /// EV / revenue in hundredths, rounded half up. `None` before revenue.
    fn multiple(&self, enterprise_value: i128) -> Option<i128> {
        (self.revenue > 0).then(|| divide_rounded(enterprise_value * 100, self.revenue))
    }
}

fn divide_rounded(numerator: i128, denominator: i128) -> i128 {
    (numerator * 2 + denominator) / (denominator * 2)
}

pub fn money(value: i128) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    if value < 0 {
        format!("-{grouped}")
    } else {
        grouped
    }
}

pub fn multiple(hundredths: Option<i128>) -> String {
    match hundredths {
        Some(value) => format!("{}.{:02}x", value / 100, value % 100),
        None => "n/a (pre-revenue)".to_string(),
    }
}

pub fn percent(basis_points: u64) -> String {
    format!("+{}.{:02}%", basis_points / 100, basis_points % 100)
}

pub fn density(ppm: u64) -> String {
    format!("{}.{:03}", ppm / 1_000, ppm % 1_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_exactly() {
        assert_eq!(money(-1_234_567), "-1,234,567");
        assert_eq!(money(999), "999");
        assert_eq!(multiple(Some(1205)), "12.05x");
        assert_eq!(percent(12_345), "+123.45%");
        assert_eq!(density(1_005), "1.005");
    }

    #[test]
    fn rounds_half_up() {
        assert_eq!(divide_rounded(5, 2), 3);
        assert_eq!(divide_rounded(4, 3), 1);
    }
}
