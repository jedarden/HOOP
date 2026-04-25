//! Cost aggregator for session usage
//!
//! Aggregates token usage from sessions into rolling buckets:
//! - per-project
//! - per-adapter
//! - per-model
//! - per-strand
//! - per-day
//!
//! Calculates costs using pricing from config.

use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use hoop_schema::{ParsedSession, ParsedSessionKind, ParsedSessionTotalUsage, MessageUsage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Model pricing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ModelPricing {
    input_per_million: f64,
    output_per_million: f64,
    #[serde(default)]
    cache_read_per_million: Option<f64>,
    #[serde(default)]
    cache_write_per_million: Option<f64>,
}

impl ModelPricing {
    fn cache_read_per_million(&self) -> f64 {
        self.cache_read_per_million.unwrap_or(0.0)
    }

    fn cache_write_per_million(&self) -> f64 {
        self.cache_write_per_million.unwrap_or(0.0)
    }
}

/// Adapter pricing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
struct AdapterPricing {
    #[serde(default)]
    models: HashMap<String, ModelPricing>,
    #[serde(default)]
    default_model: Option<String>,
}

/// Full pricing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
struct PricingConfigInternal {
    #[serde(default)]
    adapters: HashMap<String, AdapterPricing>,
}

impl Default for PricingConfigInternal {
    fn default() -> Self {
        serde_yaml::from_str(DEFAULT_PRICING_YAML)
            .expect("Default pricing YAML should be valid")
    }
}

/// Cost bucket key for aggregation
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct CostBucketKey {
    /// Date in YYYY-MM-DD format
    date: NaiveDate,
    /// Project name
    project: String,
    /// Adapter name
    adapter: String,
    /// Model name
    model: String,
    /// Strand name (null if not applicable)
    strand: Option<String>,
    /// Fleet (worker/NEEDLE-tagged) or operator (all others)
    classification: String,
}

/// Usage accumulator for aggregation
#[derive(Debug, Clone, Default)]
struct UsageAccumulator {
    input_tokens: i64,
    output_tokens: i64,
    cache_read_tokens: i64,
    cache_write_tokens: i64,
    request_count: i64,
}

impl UsageAccumulator {
    fn add(&mut self, usage: &ParsedSessionTotalUsage) {
        self.input_tokens += usage.input_tokens;
        self.output_tokens += usage.output_tokens;
        self.cache_read_tokens += usage.cache_read_tokens;
        self.cache_write_tokens += usage.cache_write_tokens;
        self.request_count += 1;
    }

    fn to_message_usage(&self) -> MessageUsage {
        MessageUsage {
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            cache_read_tokens: self.cache_read_tokens,
            cache_write_tokens: self.cache_write_tokens,
        }
    }
}

/// Cost aggregator state
#[derive(Debug, Clone)]
pub struct CostAggregator {
    /// Config file path
    config_path: PathBuf,
    /// Pricing configuration
    pricing: PricingConfigInternal,
    /// Cost buckets
    buckets: HashMap<CostBucketKey, UsageAccumulator>,
}

impl CostAggregator {
    /// Create a new cost aggregator
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let pricing = Self::load_pricing(&config_path)?;
        Ok(Self {
            config_path,
            pricing,
            buckets: HashMap::new(),
        })
    }

    /// Load pricing configuration from file
    fn load_pricing(path: &Path) -> Result<PricingConfigInternal> {
        if !path.exists() {
            info!("Pricing config not found at {}, using defaults", path.display());
            return Ok(PricingConfigInternal::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read pricing config from {}", path.display()))?;

        let config: PricingConfigInternal = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse pricing config from {}", path.display()))?;

        Ok(config)
    }

    /// Reload pricing configuration
    pub fn reload_pricing(&mut self) -> Result<()> {
        self.pricing = Self::load_pricing(&self.config_path)?;
        info!("Reloaded pricing configuration from {}", self.config_path.display());
        Ok(())
    }

    /// Aggregate usage from a session
    pub fn aggregate_session(&mut self, session: &ParsedSession) -> Result<()> {
        // Extract project from cwd (use parent dir name as project)
        let project = Self::extract_project(&session.cwd);

        // Extract model from session (for worker sessions, infer from worker metadata)
        let model = Self::extract_model(session);

        // Extract strand from session kind
        let strand = Self::extract_strand(&session.kind);

        // Derive fleet/operator classification from session kind
        let classification = Self::extract_classification(&session.kind);

        // Get date from created_at
        let date = session.created_at.date_naive();

        // Create bucket key
        let key = CostBucketKey {
            date,
            project,
            adapter: session.provider.clone(),
            model,
            strand: strand.clone(),
            classification,
        };

        // Add usage to bucket
        let accumulator = self.buckets.entry(key.clone()).or_default();
        accumulator.add(&session.total_usage);

        debug!(
            "Aggregated session {} into bucket: {} {} {} {}",
            session.id,
            key.date,
            key.adapter,
            key.model,
            strand.as_deref().unwrap_or("none")
        );

        Ok(())
    }

    /// Extract project name from cwd
    fn extract_project(cwd: &str) -> String {
        // Use the last directory component as project name
        PathBuf::from(cwd)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Extract model name from session
    fn extract_model(session: &ParsedSession) -> String {
        // For worker sessions (needle tag), extract from worker name
        if let ParsedSessionKind::Variant0 { worker, .. } = &session.kind {
            return Self::worker_to_model(worker);
        }

        // For other sessions, use adapter-specific default model
        Self::default_model_for_adapter(&session.provider)
    }

    /// Convert worker name to model name
    fn worker_to_model(worker: &str) -> String {
        match worker {
            "alpha" => "opus".to_string(),
            "beta" => "sonnet".to_string(),
            "gamma" => "haiku".to_string(),
            _ => worker.to_string(),
        }
    }

    /// Get default model for an adapter
    fn default_model_for_adapter(adapter: &str) -> String {
        match adapter {
            "claude" => "claude-sonnet-4.6-20250514".to_string(),
            "codex" => "gpt-4-turbo".to_string(),
            "gemini" => "gemini-2.5-flash".to_string(),
            "opencode" => "gpt-4o".to_string(),
            "aider" => "claude-sonnet-4.6-20250514".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Extract strand from session kind
    fn extract_strand(kind: &ParsedSessionKind) -> Option<String> {
        if let ParsedSessionKind::Variant0 { strand, .. } = kind {
            strand.clone()
        } else {
            None
        }
    }

    /// Derive fleet/operator classification from session kind.
    /// Worker sessions (Variant0 = needle-tagged) are "fleet"; all others are "operator".
    fn extract_classification(kind: &ParsedSessionKind) -> String {
        match kind {
            ParsedSessionKind::Variant0 { .. } => "fleet".to_string(),
            _ => "operator".to_string(),
        }
    }

    /// Calculate cost for a bucket using pricing config
    fn calculate_cost(&self, key: &CostBucketKey, usage: &UsageAccumulator) -> f64 {
        // Find pricing for adapter/model
        let adapter_pricing = self.pricing.adapters.get(&key.adapter)
            .or_else(|| {
                // Fallback to claude adapter pricing
                self.pricing.adapters.get("claude")
            });

        if let Some(adapter) = adapter_pricing {
            if let Some(model_pricing) = adapter.models.get(&key.model) {
                return Self::apply_pricing(usage, model_pricing);
            }

            // Try default model
            if let Some(default) = &adapter.default_model {
                if let Some(model_pricing) = adapter.models.get(default) {
                    return Self::apply_pricing(usage, model_pricing);
                }
            }
        }

        // Fallback to Claude Opus pricing
        warn!("No pricing found for {}/{} using fallback", key.adapter, key.model);
        Self::fallback_pricing(usage)
    }

    /// Apply pricing to usage
    fn apply_pricing(usage: &UsageAccumulator, model: &ModelPricing) -> f64 {
        let input_per_m = model.input_per_million / 1_000_000.0;
        let output_per_m = model.output_per_million / 1_000_000.0;
        let cache_read_per_m = model.cache_read_per_million() / 1_000_000.0;
        let cache_write_per_m = model.cache_write_per_million() / 1_000_000.0;

        let input_cost = usage.input_tokens as f64 * input_per_m;
        let output_cost = usage.output_tokens as f64 * output_per_m;
        let cache_read_cost = usage.cache_read_tokens as f64 * cache_read_per_m;
        let cache_write_cost = usage.cache_write_tokens as f64 * cache_write_per_m;

        input_cost + output_cost + cache_read_cost + cache_write_cost
    }

    /// Fallback pricing when no config is available
    fn fallback_pricing(usage: &UsageAccumulator) -> f64 {
        // Conservative Claude Opus pricing
        const INPUT_PER_M: f64 = 15.0 / 1_000_000.0;
        const OUTPUT_PER_M: f64 = 75.0 / 1_000_000.0;
        const CACHE_READ_PER_M: f64 = 0.0 / 1_000_000.0;
        const CACHE_WRITE_PER_M: f64 = 3.75 / 1_000_000.0;

        let input_cost = usage.input_tokens as f64 * INPUT_PER_M;
        let output_cost = usage.output_tokens as f64 * OUTPUT_PER_M;
        let cache_read_cost = usage.cache_read_tokens as f64 * CACHE_READ_PER_M;
        let cache_write_cost = usage.cache_write_tokens as f64 * CACHE_WRITE_PER_M;

        input_cost + output_cost + cache_read_cost + cache_write_cost
    }

    /// Get all cost buckets
    pub fn get_buckets(&self) -> Vec<CostBucket> {
        self.buckets
            .iter()
            .map(|(key, usage)| CostBucket {
                date: key.date.to_string(),
                project: key.project.clone(),
                adapter: key.adapter.clone(),
                model: key.model.clone(),
                strand: key.strand.clone(),
                usage: usage.to_message_usage(),
                request_count: usage.request_count,
                cost_usd: self.calculate_cost(key, usage),
            })
            .collect()
    }

    /// Get buckets filtered by project
    pub fn get_buckets_by_project(&self, project: &str) -> Vec<CostBucket> {
        self.get_buckets()
            .into_iter()
            .filter(|b| b.project == project)
            .collect()
    }

    /// Get buckets filtered by date range
    pub fn get_buckets_by_date_range(&self, start: NaiveDate, end: NaiveDate) -> Vec<CostBucket> {
        self.get_buckets()
            .into_iter()
            .filter(|b| {
                if let Ok(d) = NaiveDate::parse_from_str(&b.date, "%Y-%m-%d") {
                    d >= start && d <= end
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get total cost for a project today
    pub fn cost_today_for_project(&self, project: &str) -> f64 {
        let today = Utc::now().date_naive();
        self.buckets
            .iter()
            .filter(|(key, _)| key.project == project && key.date == today)
            .map(|(key, usage)| self.calculate_cost(key, usage))
            .sum()
    }

    /// Return per-(project, date) cost rollup rows suitable for persisting to fleet.db.
    ///
    /// Aggregates all in-memory buckets by (project, date), summing tokens and cost.
    pub fn get_project_date_rollup(&self) -> Vec<(String, String, f64, i64, i64, i64, i64)> {
        let mut map: HashMap<(String, String), (f64, i64, i64, i64, i64)> = HashMap::new();
        for (key, usage) in &self.buckets {
            let cost = self.calculate_cost(key, usage);
            let entry = map
                .entry((key.project.clone(), key.date.to_string()))
                .or_default();
            entry.0 += cost;
            entry.1 += usage.input_tokens;
            entry.2 += usage.output_tokens;
            entry.3 += usage.cache_read_tokens;
            entry.4 += usage.cache_write_tokens;
        }
        map.into_iter()
            .map(|((project, date), (cost, input, output, cache_read, cache_write))| {
                (project, date, cost, input, output, cache_read, cache_write)
            })
            .collect()
    }

    /// Clear all buckets (e.g., for daily reset)
    pub fn clear(&mut self) {
        self.buckets.clear();
    }

    /// Clear buckets older than a given date
    pub fn clear_before(&mut self, date: NaiveDate) {
        self.buckets.retain(|key, _| key.date >= date);
    }
}

/// Cost bucket for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBucket {
    pub date: String,
    pub project: String,
    pub adapter: String,
    pub model: String,
    pub strand: Option<String>,
    /// Fleet (worker/NEEDLE-tagged) or operator (all others). Derived from session kind at
    /// aggregation time; never mutated after the session is classified.
    pub classification: String,
    pub usage: hoop_schema::MessageUsage,
    pub request_count: i64,
    pub cost_usd: f64,
}

/// Default pricing configuration as YAML
const DEFAULT_PRICING_YAML: &str = r#"
adapters:
  claude:
    models:
      claude-sonnet-4.6-20250514:
        input_per_million: 3.0
        output_per_million: 15.0
        cache_read_per_million: 0.0
        cache_write_per_million: 0.30
      claude-opus-4.7:
        input_per_million: 15.0
        output_per_million: 75.0
        cache_read_per_million: 0.0
        cache_write_per_million: 3.75
      claude-haiku-4.5:
        input_per_million: 0.25
        output_per_million: 1.25
        cache_read_per_million: 0.0
        cache_write_per_million: 0.03
      opus:
        input_per_million: 15.0
        output_per_million: 75.0
        cache_read_per_million: 0.0
        cache_write_per_million: 3.75
      sonnet:
        input_per_million: 3.0
        output_per_million: 15.0
        cache_read_per_million: 0.0
        cache_write_per_million: 0.30
      haiku:
        input_per_million: 0.25
        output_per_million: 1.25
        cache_read_per_million: 0.0
        cache_write_per_million: 0.03
    default_model: sonnet
  codex:
    models:
      gpt-4-turbo:
        input_per_million: 10.0
        output_per_million: 30.0
      gpt-4:
        input_per_million: 30.0
        output_per_million: 60.0
      gpt-3.5-turbo:
        input_per_million: 0.5
        output_per_million: 1.5
    default_model: gpt-4-turbo
  gemini:
    models:
      gemini-2.5-pro:
        input_per_million: 1.25
        output_per_million: 10.0
        cache_read_per_million: 0.0
        cache_write_per_million: 0.0
      gemini-2.5-flash:
        input_per_million: 0.075
        output_per_million: 0.30
        cache_read_per_million: 0.0
        cache_write_per_million: 0.0
      gemini-2.0-flash:
        input_per_million: 0.10
        output_per_million: 0.40
      gemini-1.5-pro:
        input_per_million: 1.25
        output_per_million: 5.0
      gemini-1.5-flash:
        input_per_million: 0.075
        output_per_million: 0.30
    default_model: gemini-2.5-flash
  opencode:
    models:
      gpt-4o:
        input_per_million: 2.50
        output_per_million: 10.0
      gpt-4o-mini:
        input_per_million: 0.15
        output_per_million: 0.60
      o1-preview:
        input_per_million: 15.0
        output_per_million: 60.0
      o1-mini:
        input_per_million: 3.0
        output_per_million: 12.0
    default_model: gpt-4o
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use hoop_schema::ParsedSessionTotalUsage;

    #[test]
    fn test_extract_project() {
        assert_eq!(CostAggregator::extract_project("/home/coding/HOOP"), "HOOP");
        assert_eq!(CostAggregator::extract_project("/home/user/projects/my-project"), "my-project");
    }

    #[test]
    fn test_worker_to_model() {
        assert_eq!(CostAggregator::worker_to_model("alpha"), "opus");
        assert_eq!(CostAggregator::worker_to_model("beta"), "sonnet");
        assert_eq!(CostAggregator::worker_to_model("gamma"), "haiku");
        assert_eq!(CostAggregator::worker_to_model("delta"), "delta");
    }

    #[test]
    fn test_usage_accumulator() {
        let mut acc = UsageAccumulator::default();
        acc.add(&ParsedSessionTotalUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_tokens: 10,
            cache_write_tokens: 5,
        });
        assert_eq!(acc.input_tokens, 100);
        assert_eq!(acc.output_tokens, 50);
        assert_eq!(acc.request_count, 1);
    }

    #[test]
    fn test_default_pricing() {
        let pricing = PricingConfigInternal::default();
        assert!(pricing.adapters.contains_key("claude"));
        assert!(pricing.adapters.contains_key("codex"));
    }
}
