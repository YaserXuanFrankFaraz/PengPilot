use std::time::Duration;

use chrono::{Datelike as _, NaiveDate};
use serde::{Deserialize, Serialize};

/// The windows the daily and per-project views offer, in menu order.
pub const WINDOW_CHOICES: [UsageWindow; 5] = [
    UsageWindow::TrailingDays(7),
    UsageWindow::TrailingDays(30),
    UsageWindow::TrailingDays(90),
    UsageWindow::ThisMonth,
    UsageWindow::LastMonth,
];

/// The fixed window the monthly statement view scans.
pub const MONTHLY_WINDOW: UsageWindow = UsageWindow::Months(12);

/// The stretch of calendar time one scan covers.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UsageWindow {
    /// The trailing N days ending today.
    TrailingDays(u32),
    /// The current calendar month and the N−1 before it.
    Months(u32),
    /// The current calendar month to date.
    ThisMonth,
    /// The previous calendar month, whole.
    LastMonth,
}

impl UsageWindow {
    /// The inclusive `(since, until)` days the window covers, relative to
    /// `today`. Only `LastMonth` ends before today.
    pub fn bounds(self, today: NaiveDate) -> (NaiveDate, NaiveDate) {
        match self {
            UsageWindow::TrailingDays(days) => (
                today - chrono::Days::new(u64::from(days.saturating_sub(1))),
                today,
            ),
            UsageWindow::Months(months) => (
                first_of_month(today)
                    .checked_sub_months(chrono::Months::new(months.saturating_sub(1)))
                    .unwrap_or_else(|| first_of_month(today)),
                today,
            ),
            UsageWindow::ThisMonth => (first_of_month(today), today),
            UsageWindow::LastMonth => {
                let this_first = first_of_month(today);
                (
                    this_first
                        .checked_sub_months(chrono::Months::new(1))
                        .unwrap_or(this_first),
                    this_first.pred_opt().unwrap_or(today),
                )
            }
        }
    }
}

/// The first day of `day`'s calendar month.
pub fn first_of_month(day: NaiveDate) -> NaiveDate {
    day.with_day(1).unwrap_or(day)
}

/// Inclusive day list between the window bounds, oldest first — the chart's
/// x-axis, including days with no activity.
pub fn enumerate_days(since_day: NaiveDate, until_day: NaiveDate) -> Vec<NaiveDate> {
    let mut days = Vec::new();
    let mut cursor = since_day;
    while cursor <= until_day {
        days.push(cursor);
        cursor = cursor + chrono::Days::new(1);
    }
    days
}

/// Inclusive first-of-month list between the bounds' months, oldest first —
/// the statement view's rows, including months with no activity.
pub fn enumerate_months(since_day: NaiveDate, until_day: NaiveDate) -> Vec<NaiveDate> {
    let mut months = Vec::new();
    let mut cursor = first_of_month(since_day);
    let last = first_of_month(until_day);
    while cursor <= last {
        months.push(cursor);
        let Some(next) = cursor.checked_add_months(chrono::Months::new(1)) else {
            break;
        };
        cursor = next;
    }
    months
}

/// Number of days in `first_day`'s month.
pub fn days_in_month(first_day: NaiveDate) -> u32 {
    first_day
        .checked_add_months(chrono::Months::new(1))
        .and_then(|next| next.pred_opt())
        .map(|last| last.day())
        .unwrap_or(31)
}

/// Providers with a transcript scanner. Mirrors T3 Code's coverage, plus Grok.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UsageProvider {
    Claude,
    Codex,
    Grok,
}

impl UsageProvider {
    pub const COUNT: usize = 3;
    pub const ALL: [UsageProvider; Self::COUNT] = [
        UsageProvider::Claude,
        UsageProvider::Codex,
        UsageProvider::Grok,
    ];

    pub fn label(self) -> &'static str {
        match self {
            UsageProvider::Claude => "Claude Code",
            UsageProvider::Codex => "Codex",
            UsageProvider::Grok => "Grok",
        }
    }

    /// Index into the fixed per-provider arrays on [`DaySlice`].
    pub fn index(self) -> usize {
        match self {
            UsageProvider::Claude => 0,
            UsageProvider::Codex => 1,
            UsageProvider::Grok => 2,
        }
    }
}

/// Token counts for one record or bucket. `reasoning` is a subset of `output`
/// and is never added into [`TokenTotals::total`].
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTotals {
    pub uncached_input: u64,
    pub cached_input: u64,
    pub cache_creation: u64,
    pub output: u64,
    pub reasoning: u64,
}

impl TokenTotals {
    pub fn total(&self) -> u64 {
        self.uncached_input + self.cached_input + self.cache_creation + self.output
    }

    pub fn add(&mut self, other: &TokenTotals) {
        self.uncached_input += other.uncached_input;
        self.cached_input += other.cached_input;
        self.cache_creation += other.cache_creation;
        self.output += other.output;
        self.reasoning += other.reasoning;
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PricingStatus {
    /// Fetched from LiteLLM within this scan.
    Fresh,
    /// Served from the on-disk snapshot.
    Cached,
    /// No table at all; every model reports as unpriced.
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSlice {
    pub provider: UsageProvider,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub cost_share: f64,
    pub token_share: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSlice {
    pub provider: UsageProvider,
    pub model: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub cost_share: f64,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDay {
    pub cost_usd: f64,
    pub total_tokens: u64,
}

/// One calendar day with activity, ascending by day in
/// [`UsageHistory::daily`]. `by_provider` is indexed by
/// [`UsageProvider::index`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DaySlice {
    pub day: NaiveDate,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub by_provider: [ProviderDay; UsageProvider::COUNT],
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CostQuality {
    pub provider_reported_share: f64,
    pub model_priced_share: f64,
    pub unpriced_share: f64,
    pub cache_savings_usd: f64,
}

/// One calendar month with activity, keyed by its first day, ascending in
/// [`UsageHistory::months`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthSlice {
    pub first_day: NaiveDate,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub by_provider: [ProviderDay; UsageProvider::COUNT],
    /// Distinct sessions active in the month.
    pub sessions: u64,
    /// Days in the month with any tokens.
    pub active_days: u32,
    /// Cost per model, descending; the render caps how many it shows.
    pub top_models: Vec<(String, f64)>,
}

/// One project's usage within the window, descending by cost in
/// [`UsageHistory::projects`]. `path` is the resolved project root (or the
/// raw working directory when no known root contains it; empty when the
/// transcripts did not record one).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSlice {
    pub path: String,
    pub cost_usd: f64,
    pub total_tokens: u64,
    pub by_provider: [ProviderDay; UsageProvider::COUNT],
    pub sessions: u64,
    pub cost_share: f64,
    pub last_day: Option<NaiveDate>,
    /// Cost per model, descending; the render caps how many it shows.
    pub top_models: Vec<(String, f64)>,
}

/// The fully derived snapshot the Usage page renders. Everything a frame
/// needs is precomputed here so render does no aggregation of its own.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageHistory {
    pub window: UsageWindow,
    pub since_day: NaiveDate,
    pub until_day: NaiveDate,
    pub totals: TokenTotals,
    pub total_tokens: u64,
    pub cost_usd: f64,
    pub records: u64,
    pub sessions: u64,
    /// Sorted by cost descending.
    pub providers: Vec<ProviderSlice>,
    /// Sorted by cost descending, then tokens.
    pub models: Vec<ModelSlice>,
    /// Days with activity, ascending.
    pub daily: Vec<DaySlice>,
    /// Months with activity, ascending.
    pub months: Vec<MonthSlice>,
    /// Projects by cost, descending.
    pub projects: Vec<ProjectSlice>,
    pub quality: CostQuality,
    pub pricing: PricingStatus,
    pub scanned_files: usize,
    pub skipped_files: usize,
    /// Provider roots that exist but could not be read.
    pub errors: Vec<String>,
    #[serde(with = "duration_secs_nanos")]
    pub scan_duration: Duration,
}

impl UsageHistory {
    /// The day's slice, if that day had activity. `daily` is sorted, so this
    /// is a binary search.
    pub fn day(&self, day: NaiveDate) -> Option<&DaySlice> {
        self.daily
            .binary_search_by_key(&day, |slice| slice.day)
            .ok()
            .map(|index| &self.daily[index])
    }

    /// The month's slice by its first day, if that month had activity.
    pub fn month(&self, first_day: NaiveDate) -> Option<&MonthSlice> {
        self.months
            .binary_search_by_key(&first_day, |slice| slice.first_day)
            .ok()
            .map(|index| &self.months[index])
    }
}

mod duration_secs_nanos {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Deserialize, Serialize)]
    struct Repr {
        secs: u64,
        nanos: u32,
    }

    pub fn serialize<S: Serializer>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
        Repr {
            secs: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
        let repr = Repr::deserialize(deserializer)?;
        Ok(Duration::new(repr.secs, repr.nanos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grok_is_the_third_usage_provider() {
        assert_eq!(UsageProvider::COUNT, 3);
        assert_eq!(UsageProvider::Grok.index(), 2);
        assert_eq!(UsageProvider::ALL[2], UsageProvider::Grok);
    }

    #[test]
    fn usage_history_round_trips_scan_duration() {
        let history = UsageHistory {
            window: UsageWindow::TrailingDays(7),
            since_day: NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
            until_day: NaiveDate::from_ymd_opt(2026, 8, 18).unwrap(),
            totals: TokenTotals::default(),
            total_tokens: 0,
            cost_usd: 0.0,
            records: 0,
            sessions: 0,
            providers: Vec::new(),
            models: Vec::new(),
            daily: Vec::new(),
            months: Vec::new(),
            projects: Vec::new(),
            quality: CostQuality::default(),
            pricing: PricingStatus::Unavailable,
            scanned_files: 0,
            skipped_files: 0,
            errors: Vec::new(),
            scan_duration: Duration::from_millis(1250),
        };
        let value = serde_json::to_value(&history).unwrap();
        assert_eq!(value["scanDuration"]["secs"], 1);
        assert_eq!(value["scanDuration"]["nanos"], 250_000_000);
        let restored: UsageHistory = serde_json::from_value(value).unwrap();
        assert_eq!(restored.scan_duration, Duration::from_millis(1250));
        assert_eq!(restored.window, UsageWindow::TrailingDays(7));
    }
}
