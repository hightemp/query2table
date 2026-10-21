//! The usage journal: what the app spent, call by call, for the Statistics section. Rows outlive
//! their runs (no cascade), so deleting history does not erase spending. A row may stand for
//! several calls: runs made before the journal are imported from their summaries, and calls
//! older than a year are folded into daily totals.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;

use super::repository::Repository;

/// Setting with the time the journal started; runs made before it are imported once.
pub const JOURNAL_SINCE: &str = "usage_journal_since";

/// One or more calls of one kind: `llm`, `search` (provider brave/serper, model = search mode)
/// or `download` (pages, PDFs, images).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageEvent {
    /// Unix seconds.
    pub at: i64,
    pub run_id: Option<String>,
    pub run_type: String,
    pub kind: String,
    pub provider: String,
    /// The model that answered.
    pub model: String,
    /// The model asked for, which manual rates name.
    pub requested_model: String,
    pub calls: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub reasoning_tokens: i64,
    pub cached_tokens: i64,
    pub cache_write_tokens: i64,
    pub reported_usd: f64,
    pub estimated_usd: f64,
    /// Calls whose cost is unknown (no rate for the model).
    pub unpriced_calls: i64,
    pub bytes_sent: i64,
    pub bytes_received: i64,
}

/// A run as the statistics name it, kept after the run is deleted.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageRun {
    pub run_id: String,
    pub run_type: String,
    pub title: String,
    pub started_at: i64,
}

pub(crate) const SCHEMA: [&str; 4] = [
    "CREATE TABLE IF NOT EXISTS usage_events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        at INTEGER NOT NULL,
        run_id TEXT,
        run_type TEXT NOT NULL DEFAULT '',
        kind TEXT NOT NULL,
        provider TEXT NOT NULL DEFAULT '',
        model TEXT NOT NULL DEFAULT '',
        requested_model TEXT NOT NULL DEFAULT '',
        calls INTEGER NOT NULL DEFAULT 1,
        prompt_tokens INTEGER NOT NULL DEFAULT 0,
        completion_tokens INTEGER NOT NULL DEFAULT 0,
        reasoning_tokens INTEGER NOT NULL DEFAULT 0,
        cached_tokens INTEGER NOT NULL DEFAULT 0,
        cache_write_tokens INTEGER NOT NULL DEFAULT 0,
        reported_usd REAL NOT NULL DEFAULT 0,
        estimated_usd REAL NOT NULL DEFAULT 0,
        unpriced_calls INTEGER NOT NULL DEFAULT 0,
        bytes_sent INTEGER NOT NULL DEFAULT 0,
        bytes_received INTEGER NOT NULL DEFAULT 0
    )",
    "CREATE INDEX IF NOT EXISTS idx_usage_events_at ON usage_events(at)",
    "CREATE TABLE IF NOT EXISTS usage_runs (
        run_id TEXT PRIMARY KEY,
        run_type TEXT NOT NULL DEFAULT '',
        title TEXT NOT NULL DEFAULT '',
        started_at INTEGER NOT NULL DEFAULT 0
    )",
    // The statistics show a run under its current name; a deleted run keeps the last one.
    "CREATE TRIGGER IF NOT EXISTS usage_runs_keep_title BEFORE DELETE ON runs BEGIN
        UPDATE usage_runs SET title = COALESCE(NULLIF(OLD.title, ''), OLD.query) WHERE run_id = OLD.id;
    END",
];

const COLUMNS: &str = "at, run_id, run_type, kind, provider, model, requested_model, calls, prompt_tokens, completion_tokens, \
    reasoning_tokens, cached_tokens, cache_write_tokens, reported_usd, estimated_usd, unpriced_calls, bytes_sent, bytes_received";

/// Creates the journal and, on its first start, imports the runs made before it.
pub(crate) async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    for statement in SCHEMA {
        sqlx::query(statement).execute(pool).await?;
    }
    let now: i64 = sqlx::query_scalar("SELECT unixepoch()").fetch_one(pool).await?;
    let started: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?").bind(JOURNAL_SINCE).fetch_optional(pool).await?;
    if started.is_none() {
        let mut tx = pool.begin().await?;
        import_runs(&mut tx, now).await?;
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?)").bind(JOURNAL_SINCE).bind(now.to_string()).execute(&mut *tx).await?;
        tx.commit().await?;
    }
    fold(pool, now - FOLD_AFTER_SECS).await
}

/// Calls older than this are kept as daily totals.
const FOLD_AFTER_SECS: i64 = 365 * 86_400;

/// Folds the rows before `before` into one row per local day, run, kind, provider and model,
/// dated by the day's first call.
async fn fold(pool: &SqlitePool, before: i64) -> Result<(), sqlx::Error> {
    let sums = "SUM(calls), SUM(prompt_tokens), SUM(completion_tokens), SUM(reasoning_tokens), SUM(cached_tokens), \
        SUM(cache_write_tokens), SUM(reported_usd), SUM(estimated_usd), SUM(unpriced_calls), SUM(bytes_sent), SUM(bytes_received)";
    let mut tx = pool.begin().await?;
    sqlx::query(&format!(
        "CREATE TEMP TABLE usage_folded AS
         SELECT MIN(at) AS at, run_id, run_type, kind, provider, model, requested_model, {sums}
         FROM usage_events WHERE at < ?1
         GROUP BY date(at, 'unixepoch', 'localtime'), run_id, run_type, kind, provider, model, requested_model"
    ))
    .bind(before)
    .execute(&mut *tx)
    .await?;
    sqlx::query("DELETE FROM usage_events WHERE at < ?").bind(before).execute(&mut *tx).await?;
    sqlx::query(&format!("INSERT INTO usage_events ({COLUMNS}) SELECT * FROM usage_folded ORDER BY at, run_id, kind, provider, model"))
        .execute(&mut *tx)
        .await?;
    sqlx::query("DROP TABLE usage_folded").execute(&mut *tx).await?;
    tx.commit().await
}

/// Rows of a saved accounting summary (`BudgetSnapshot`), dated `at`.
fn summary_events(summary: &Value, at: i64, run_id: &str, run_type: &str) -> Vec<UsageEvent> {
    let int = |v: &Value, key: &str| v.get(key).and_then(Value::as_i64).unwrap_or(0);
    let money = |v: &Value, key: &str| v.get(key).and_then(Value::as_f64).unwrap_or(0.0);
    let text = |v: &Value, key: &str| v.get(key).and_then(Value::as_str).unwrap_or("").to_string();
    let base = UsageEvent { at, run_id: Some(run_id.into()), run_type: run_type.into(), ..Default::default() };
    let mut events: Vec<UsageEvent> = summary
        .get("breakdown")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|line| int(line, "calls") > 0)
        .map(|line| {
            let provider = text(line, "provider");
            UsageEvent {
                kind: if matches!(provider.as_str(), "brave" | "serper") { "search" } else { "llm" }.into(),
                provider,
                model: text(line, "model"),
                requested_model: text(line, "requested_model"),
                calls: int(line, "calls"),
                prompt_tokens: int(line, "prompt_tokens"),
                completion_tokens: int(line, "completion_tokens"),
                reported_usd: money(line, "reported_usd"),
                estimated_usd: money(line, "estimated_usd"),
                unpriced_calls: int(line, "unpriced_calls"),
                ..base.clone()
            }
        })
        .collect();
    let downloads = int(summary, "fetch_calls");
    if downloads > 0 {
        events.push(UsageEvent { kind: "download".into(), calls: downloads, ..base });
    }
    events
}

async fn import_runs(tx: &mut sqlx::SqliteConnection, now: i64) -> Result<(), sqlx::Error> {
    let runs: Vec<(String, String, String, i64, Option<String>)> = sqlx::query_as(
        "SELECT id, run_type, COALESCE(NULLIF(title, ''), query), created_at, stats FROM runs WHERE created_at <= ? ORDER BY created_at, id",
    )
    .bind(now)
    .fetch_all(&mut *tx)
    .await?;
    let turns: Vec<(String, i64, String)> = sqlx::query_as(
        "SELECT run_id, created_at, accounting FROM research_turns WHERE accounting IS NOT NULL ORDER BY created_at, turn_index",
    )
    .fetch_all(&mut *tx)
    .await?;
    let mut events = Vec::new();
    for (id, run_type, title, created_at, stats) in &runs {
        insert_run(&mut *tx, &UsageRun { run_id: id.clone(), run_type: run_type.clone(), title: title.clone(), started_at: *created_at }).await?;
        // A research run's summary holds only its last question; each question keeps its own.
        let own_turns: Vec<_> = turns.iter().filter(|(run, _, _)| run == id).collect();
        if !own_turns.is_empty() {
            for (_, at, accounting) in own_turns {
                if let Ok(summary) = serde_json::from_str::<Value>(accounting) {
                    events.extend(summary_events(&summary, *at, id, run_type));
                }
            }
            continue;
        }
        let summary = stats.as_deref().and_then(|s| serde_json::from_str::<Value>(s).ok()).and_then(|s| s.get("accounting").cloned());
        if let Some(summary) = summary {
            events.extend(summary_events(&summary, *created_at, id, run_type));
        }
    }
    events.sort_by_key(|e| e.at);
    for event in &events {
        insert_event(&mut *tx, event).await?;
    }
    Ok(())
}

async fn insert_event(executor: impl sqlx::SqliteExecutor<'_>, e: &UsageEvent) -> Result<(), sqlx::Error> {
    sqlx::query(&format!("INSERT INTO usage_events ({COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"))
        .bind(e.at)
        .bind(&e.run_id)
        .bind(&e.run_type)
        .bind(&e.kind)
        .bind(&e.provider)
        .bind(&e.model)
        .bind(&e.requested_model)
        .bind(e.calls)
        .bind(e.prompt_tokens)
        .bind(e.completion_tokens)
        .bind(e.reasoning_tokens)
        .bind(e.cached_tokens)
        .bind(e.cache_write_tokens)
        .bind(e.reported_usd)
        .bind(e.estimated_usd)
        .bind(e.unpriced_calls)
        .bind(e.bytes_sent)
        .bind(e.bytes_received)
        .execute(executor)
        .await?;
    Ok(())
}

async fn insert_run(executor: impl sqlx::SqliteExecutor<'_>, run: &UsageRun) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO usage_runs (run_id, run_type, title, started_at) VALUES (?, ?, ?, ?) ON CONFLICT(run_id) DO NOTHING",
    )
    .bind(&run.run_id)
    .bind(&run.run_type)
    .bind(&run.title)
    .bind(run.started_at)
    .execute(executor)
    .await?;
    Ok(())
}

impl Repository {
    pub async fn record_usage(&self, events: &[UsageEvent]) -> Result<(), sqlx::Error> {
        let mut tx = self.pool().begin().await?;
        for event in events {
            insert_event(&mut *tx, event).await?;
        }
        tx.commit().await
    }

    /// Names a run for the statistics; a run already named keeps its name and start.
    pub async fn remember_usage_run(&self, run: &UsageRun) -> Result<(), sqlx::Error> {
        insert_run(self.pool(), run).await
    }

    /// Rows with `from <= at < to`, oldest first.
    pub async fn usage_events(&self, from: i64, to: i64) -> Result<Vec<UsageEvent>, sqlx::Error> {
        sqlx::query_as(&format!("SELECT {COLUMNS} FROM usage_events WHERE at >= ? AND at < ? ORDER BY at, id"))
            .bind(from)
            .bind(to)
            .fetch_all(self.pool())
            .await
    }

    pub async fn usage_runs(&self) -> Result<Vec<UsageRun>, sqlx::Error> {
        sqlx::query_as("SELECT run_id, run_type, title, started_at FROM usage_runs ORDER BY run_id").fetch_all(self.pool()).await
    }

    /// Folds the rows before `before` into daily totals (done at every start for calls older
    /// than a year).
    pub async fn fold_usage(&self, before: i64) -> Result<(), sqlx::Error> {
        fold(self.pool(), before).await
    }

    pub async fn clear_usage(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool().begin().await?;
        sqlx::query("DELETE FROM usage_events").execute(&mut *tx).await?;
        sqlx::query("DELETE FROM usage_runs").execute(&mut *tx).await?;
        tx.commit().await
    }
}

/// A statistics period `[from, to)`, its chart buckets (ascending starts, the first at `from`)
/// and the start of the period before it (`[previous_from, from)`). Boundaries come from the
/// frontend, which knows the local calendar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRequest {
    pub from: i64,
    pub to: i64,
    pub buckets: Vec<i64>,
    pub previous_from: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UsageTotals {
    /// Reported plus estimated cost.
    pub cost_usd: f64,
    pub reported_usd: f64,
    pub estimated_usd: f64,
    pub unpriced_calls: i64,
    pub llm_calls: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub reasoning_tokens: i64,
    pub cached_tokens: i64,
    pub cache_write_tokens: i64,
    pub searches: i64,
    pub downloads: i64,
    /// Downloads and search responses.
    pub web_bytes: i64,
    pub model_bytes_sent: i64,
    pub model_bytes_received: i64,
    /// Runs with any usage.
    pub runs: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UsagePoint {
    pub start: i64,
    pub totals: UsageTotals,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelUsage {
    pub provider: String,
    pub model: String,
    pub calls: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub cached_tokens: i64,
    pub cost_usd: f64,
    pub unpriced_calls: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchUsage {
    pub provider: String,
    pub calls: i64,
    pub cost_usd: f64,
    pub bytes: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ModeUsage {
    pub run_type: String,
    pub runs: i64,
    pub llm_calls: i64,
    pub searches: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RunUsage {
    pub run_id: String,
    pub run_type: String,
    /// The run's current name, or its last one when it is gone.
    pub title: String,
    pub started_at: i64,
    pub cost_usd: f64,
    pub llm_calls: i64,
    /// Whether the run can still be opened (not deleted or in the trash).
    pub exists: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UsageReport {
    pub totals: UsageTotals,
    pub previous: UsageTotals,
    pub series: Vec<UsagePoint>,
    pub by_model: Vec<ModelUsage>,
    pub by_search: Vec<SearchUsage>,
    pub by_mode: Vec<ModeUsage>,
    pub top_runs: Vec<RunUsage>,
}

/// Runs listed as the most expensive of a period.
const TOP_RUNS: usize = 10;

/// Money kept to ten decimals, so sums of cents print as written.
fn money(value: f64) -> f64 {
    (value * 1e10).round() / 1e10
}

impl UsageTotals {
    fn add(&mut self, e: &UsageEvent) {
        self.reported_usd += e.reported_usd;
        self.estimated_usd += e.estimated_usd;
        self.unpriced_calls += e.unpriced_calls;
        match e.kind.as_str() {
            "llm" => {
                self.llm_calls += e.calls;
                self.prompt_tokens += e.prompt_tokens;
                self.completion_tokens += e.completion_tokens;
                self.reasoning_tokens += e.reasoning_tokens;
                self.cached_tokens += e.cached_tokens;
                self.cache_write_tokens += e.cache_write_tokens;
                self.model_bytes_sent += e.bytes_sent;
                self.model_bytes_received += e.bytes_received;
            }
            "search" => {
                self.searches += e.calls;
                self.web_bytes += e.bytes_received;
            }
            _ => {
                self.downloads += e.calls;
                self.web_bytes += e.bytes_received;
            }
        }
    }

    fn of<'a>(events: impl Iterator<Item = &'a UsageEvent>) -> Self {
        let mut totals = Self::default();
        let mut runs = std::collections::HashSet::new();
        for event in events {
            totals.add(event);
            runs.extend(event.run_id.as_deref());
        }
        totals.runs = runs.len() as i64;
        totals.reported_usd = money(totals.reported_usd);
        totals.estimated_usd = money(totals.estimated_usd);
        totals.cost_usd = money(totals.reported_usd + totals.estimated_usd);
        totals
    }
}

fn check(request: &UsageRequest) -> Result<(), String> {
    let UsageRequest { from, to, buckets, previous_from } = request;
    if from >= to || previous_from > from {
        return Err("The statistics period is invalid.".into());
    }
    if buckets.first() != Some(from) || buckets.windows(2).any(|w| w[0] >= w[1]) || buckets.last().is_some_and(|last| last >= to) {
        return Err("The statistics period has invalid chart steps.".into());
    }
    Ok(())
}

impl Repository {
    /// The statistics of a period: totals, the period before, a series by the given buckets,
    /// breakdowns and the most expensive runs.
    pub async fn usage_report(&self, request: &UsageRequest) -> Result<UsageReport, String> {
        check(request)?;
        let events = self.usage_events(request.previous_from, request.to).await.map_err(|e| e.to_string())?;
        let (previous, current): (Vec<_>, Vec<_>) = events.into_iter().partition(|e| e.at < request.from);

        let series = request
            .buckets
            .iter()
            .enumerate()
            .map(|(i, start)| {
                let end = request.buckets.get(i + 1).copied().unwrap_or(request.to);
                UsagePoint { start: *start, totals: UsageTotals::of(current.iter().filter(|e| e.at >= *start && e.at < end)) }
            })
            .collect();

        let mut models: std::collections::BTreeMap<(String, String), ModelUsage> = Default::default();
        let mut searches: std::collections::BTreeMap<String, SearchUsage> = Default::default();
        let mut modes: std::collections::BTreeMap<String, (ModeUsage, std::collections::HashSet<String>)> = Default::default();
        let mut runs: std::collections::HashMap<String, RunUsage> = Default::default();
        for e in &current {
            let cost = e.reported_usd + e.estimated_usd;
            if e.kind == "llm" {
                let line = models.entry((e.provider.clone(), e.model.clone())).or_insert_with(|| ModelUsage {
                    provider: e.provider.clone(),
                    model: e.model.clone(),
                    ..Default::default()
                });
                line.calls += e.calls;
                line.prompt_tokens += e.prompt_tokens;
                line.completion_tokens += e.completion_tokens;
                line.cached_tokens += e.cached_tokens;
                line.cost_usd += cost;
                line.unpriced_calls += e.unpriced_calls;
            }
            if e.kind == "search" {
                let line = searches.entry(e.provider.clone()).or_insert_with(|| SearchUsage { provider: e.provider.clone(), ..Default::default() });
                line.calls += e.calls;
                line.cost_usd += cost;
                line.bytes += e.bytes_received;
            }
            let (mode, mode_runs) = modes.entry(e.run_type.clone()).or_insert_with(|| (ModeUsage { run_type: e.run_type.clone(), ..Default::default() }, Default::default()));
            mode.cost_usd += cost;
            match e.kind.as_str() {
                "llm" => mode.llm_calls += e.calls,
                "search" => mode.searches += e.calls,
                _ => {}
            }
            if let Some(run_id) = &e.run_id {
                mode_runs.insert(run_id.clone());
                let run = runs.entry(run_id.clone()).or_insert_with(|| RunUsage { run_id: run_id.clone(), run_type: e.run_type.clone(), ..Default::default() });
                run.cost_usd += cost;
                if e.kind == "llm" {
                    run.llm_calls += e.calls;
                }
            }
        }

        let by_cost = |a: f64, b: f64| b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal);
        let mut by_model: Vec<ModelUsage> = models.into_values().map(|m| ModelUsage { cost_usd: money(m.cost_usd), ..m }).collect();
        by_model.sort_by(|a, b| by_cost(a.cost_usd, b.cost_usd).then(b.calls.cmp(&a.calls)));
        let by_search = searches.into_values().map(|s| SearchUsage { cost_usd: money(s.cost_usd), ..s }).collect();
        let by_mode = modes
            .into_values()
            .map(|(mode, ids)| ModeUsage { runs: ids.len() as i64, cost_usd: money(mode.cost_usd), ..mode })
            .collect();

        let mut top_runs: Vec<RunUsage> = runs.into_values().map(|r| RunUsage { cost_usd: money(r.cost_usd), ..r }).collect();
        top_runs.sort_by(|a, b| by_cost(a.cost_usd, b.cost_usd).then(b.llm_calls.cmp(&a.llm_calls)).then(a.run_id.cmp(&b.run_id)));
        top_runs.truncate(TOP_RUNS);
        for run in &mut top_runs {
            let named: Option<(String, String, i64, bool)> = sqlx::query_as(
                "SELECT COALESCE(NULLIF(r.title, ''), r.query, u.title), u.run_type, u.started_at, r.id IS NOT NULL AND r.deleted_at IS NULL
                 FROM usage_runs u LEFT JOIN runs r ON r.id = u.run_id WHERE u.run_id = ?",
            )
            .bind(&run.run_id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| e.to_string())?;
            if let Some((title, run_type, started_at, exists)) = named {
                (run.title, run.run_type, run.started_at, run.exists) = (title, run_type, started_at, exists);
            }
        }

        Ok(UsageReport {
            totals: UsageTotals::of(current.iter()),
            previous: UsageTotals::of(previous.iter()),
            series,
            by_model,
            by_search,
            by_mode,
            top_runs,
        })
    }
}

/// id, kind, provider, model, requested model, calls, input, output, cached and written tokens.
type UnpricedRow = (i64, String, String, String, String, i64, i64, i64, i64, i64);

impl Repository {
    /// Prices the calls recorded without a price with today's rates: manual model rates (for
    /// the model asked for, at any address of its provider) and the per-1000 search prices.
    /// Returns how many calls got a price. Rows mixing priced and unpriced calls are left as
    /// they are.
    pub async fn reprice_usage(&self, settings: &std::collections::HashMap<String, String>) -> Result<i64, String> {
        use crate::providers::accounting::PriceQuote;
        let rates: std::collections::BTreeMap<String, PriceQuote> = settings
            .get("llm_pricing_overrides")
            .and_then(|value| serde_json::from_str(value).ok())
            .unwrap_or_default();
        let rates: Vec<((String, String), PriceQuote)> = rates
            .into_iter()
            .filter(|(_, quote)| quote.valid())
            .filter_map(|(scope, quote)| {
                let (provider, _endpoint, model): (String, String, String) = serde_json::from_str(&scope).ok()?;
                Some(((provider, model), quote))
            })
            .collect();
        let search_rate = |provider: &str| {
            settings
                .get(&format!("{provider}_price_per_1000"))
                .and_then(|value| value.trim().parse::<f64>().ok())
                .filter(|rate| rate.is_finite() && *rate >= 0.0)
        };
        let rows: Vec<UnpricedRow> = sqlx::query_as(
            "SELECT id, kind, provider, model, requested_model, calls, prompt_tokens, completion_tokens, cached_tokens, cache_write_tokens
             FROM usage_events WHERE calls > 0 AND unpriced_calls = calls AND reported_usd = 0 AND estimated_usd = 0",
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| e.to_string())?;
        let mut priced = 0;
        let mut tx = self.pool().begin().await.map_err(|e| e.to_string())?;
        for (id, kind, provider, model, requested, calls, input, output, cached, written) in rows {
            let cost = match kind.as_str() {
                "llm" => {
                    let wanted = if requested.is_empty() { &model } else { &requested };
                    let tokens = |n: i64| Some(u32::try_from(n.max(0)).unwrap_or(u32::MAX));
                    let usage = crate::providers::llm::types::LlmUsage {
                        prompt_tokens: tokens(input),
                        completion_tokens: tokens(output),
                        cached_prompt_tokens: tokens(cached),
                        cache_write_tokens: tokens(written),
                        ..Default::default()
                    };
                    rates.iter().find(|((p, m), _)| *p == provider && m == wanted).and_then(|(_, quote)| quote.estimate(&usage))
                }
                "search" => search_rate(&provider).map(|rate| rate / 1000.0 * calls as f64),
                _ => None,
            };
            let Some(cost) = cost else { continue };
            sqlx::query("UPDATE usage_events SET estimated_usd = ?, unpriced_calls = 0 WHERE id = ?")
                .bind(money(cost))
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
            priced += calls;
        }
        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(priced)
    }
}

/// Setting with the monthly budget in USD; 0 or empty turns it off.
pub const MONTHLY_BUDGET: &str = "monthly_budget_usd";
/// The warnings already given: "<year>-<month>:<percent>".
const BUDGET_WARNED: &str = "usage_budget_warned";

/// This month's spending against the monthly budget.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MonthlyBudget {
    /// 0 when no budget is set.
    pub limit_usd: f64,
    pub spent_usd: f64,
    /// Unix seconds of the month's first local midnight.
    pub month_start: i64,
}

fn month_start<Tz: chrono::TimeZone>(now: &chrono::DateTime<Tz>) -> chrono::DateTime<Tz> {
    use chrono::Datelike;
    let zone = now.timezone();
    zone.with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .earliest()
        // A midnight skipped by a clock change: the first hour that exists.
        .or_else(|| zone.with_ymd_and_hms(now.year(), now.month(), 1, 1, 0, 0).earliest())
        .unwrap_or_else(|| now.clone())
}

impl Repository {
    pub async fn monthly_budget<Tz: chrono::TimeZone>(&self, now: &chrono::DateTime<Tz>) -> Result<MonthlyBudget, String> {
        let limit: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
            .bind(MONTHLY_BUDGET)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| e.to_string())?;
        let limit_usd = limit.and_then(|v| v.trim().parse::<f64>().ok()).filter(|v| v.is_finite() && *v > 0.0).unwrap_or(0.0);
        let start = month_start(now).timestamp();
        let spent: f64 = sqlx::query_scalar("SELECT CAST(COALESCE(SUM(reported_usd + estimated_usd), 0) AS REAL) FROM usage_events WHERE at >= ?")
            .bind(start)
            .fetch_one(self.pool())
            .await
            .map_err(|e| e.to_string())?;
        Ok(MonthlyBudget { limit_usd, spent_usd: money(spent), month_start: start })
    }

    /// The budget warning due now (80 or 100 percent), each given once a month.
    pub async fn budget_alert<Tz: chrono::TimeZone>(&self, now: &chrono::DateTime<Tz>) -> Result<Option<(u8, MonthlyBudget)>, String>
    where
        Tz::Offset: std::fmt::Display,
    {
        let budget = self.monthly_budget(now).await?;
        if budget.limit_usd <= 0.0 {
            return Ok(None);
        }
        let level = if budget.spent_usd >= budget.limit_usd {
            100
        } else if budget.spent_usd >= budget.limit_usd * 0.8 {
            80
        } else {
            return Ok(None);
        };
        let month = month_start(now).format("%Y-%m").to_string();
        let warned: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
            .bind(BUDGET_WARNED)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| e.to_string())?;
        let already = warned
            .as_deref()
            .and_then(|v| v.split_once(':'))
            .filter(|(m, _)| *m == month)
            .and_then(|(_, level)| level.parse::<u8>().ok())
            .unwrap_or(0);
        if already >= level {
            return Ok(None);
        }
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = unixepoch()")
            .bind(BUDGET_WARNED)
            .bind(format!("{month}:{level}"))
            .execute(self.pool())
            .await
            .map_err(|e| e.to_string())?;
        Ok(Some((level, budget)))
    }
}

impl Repository {
    /// When the first recorded call happened; the start of "All time".
    pub async fn usage_first_at(&self) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar("SELECT MIN(at) FROM usage_events").fetch_one(self.pool()).await
    }
}
