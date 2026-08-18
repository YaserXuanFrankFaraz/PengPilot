use chrono::Datelike as _;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanUsage {
    /// "Max (5x)", "Pro" — from the credential's subscription metadata.
    pub plan_label: Option<String>,
    pub windows: Vec<PlanWindow>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanWindow {
    pub label: String,
    /// Percent of the window already used, 0–100.
    pub percent: f64,
    /// Unix seconds when the window resets.
    pub resets_at: Option<i64>,
}

/// "87.7k", "1.0M" — the compact token count the context row shows.
pub fn format_tokens(tokens: u64) -> String {
    if tokens >= 999_500 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}k", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

/// "Resets in 49 min" close in, "Resets Thu 7:59 PM" further out — the CLI's
/// own phrasing for the same rows.
pub fn reset_label(resets_at: i64, now: i64) -> String {
    let delta = resets_at - now;
    if delta <= 0 {
        return tr!("usage.resets_soon");
    }
    let minutes = (delta + 59) / 60;
    if minutes < 60 {
        return tr!("usage.resets_in_minutes", count = minutes);
    }
    if minutes < 24 * 60 {
        let hours = minutes / 60;
        return match minutes % 60 {
            0 => tr!("usage.resets_in_hours", count = hours),
            remainder => tr!(
                "usage.resets_in_hours_minutes",
                hours = hours,
                minutes = remainder
            ),
        };
    }
    use chrono::TimeZone as _;
    match chrono::Local.timestamp_opt(resets_at, 0) {
        chrono::LocalResult::Single(date) if crate::i18n::uses_east_asian_date_format() => tr!(
            "usage.resets_date",
            date = format!(
                "{}月{}日 {}",
                date.month(),
                date.day(),
                date.format("%H:%M")
            )
        ),
        chrono::LocalResult::Single(date) => tr!(
            "usage.resets_date",
            date = date.format("%a %-I:%M %p").to_string()
        ),
        _ => tr!("usage.resets_soon"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_token_counts_use_one_decimal() {
        assert_eq!(format_tokens(950), "950");
        assert_eq!(format_tokens(87_650), "87.7k");
        assert_eq!(format_tokens(999_600), "1.0M");
        assert_eq!(format_tokens(1_000_000), "1.0M");
    }

    #[test]
    fn reset_labels_stay_relative_until_a_day_out() {
        let now = 1_700_000_000;
        assert_eq!(reset_label(now + 49 * 60, now), "Resets in 49 min");
        assert_eq!(
            reset_label(now + 3 * 3600 + 120, now),
            "Resets in 3 hr 2 min"
        );
        assert_eq!(reset_label(now - 5, now), "Resets soon");
        let far = reset_label(now + 3 * 24 * 3600, now);
        assert!(far.starts_with("Resets ") && !far.contains(" in "), "{far}");
    }
}
