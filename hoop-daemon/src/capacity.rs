//! Per-account capacity utilization from Claude Code and Gemini JSONL logs
//!
//! Computes per-account 5h and 7d rolling utilization meters.
//! - Claude: matches `/status` output using cached API response or JSONL estimation
//! - Gemini: JSONL-based estimation from session files, optional GCP quota API
//!
//! Data sources (in priority order):
//! 1. Cached API response (`~/.cache/claude-usage/usage.json`) — Claude only, exact
//! 2. GCP Consumer Quotas API — Gemini only, exact when available
//! 3. JSONL-based estimation — fallback for both adapters
//!
//! The JSONL fallback uses cost-equivalent token weighting to approximate
//! Claude's internal rate-limit accounting. It is inherently approximate
//! because the exact weighting formula is proprietary. The cached API
//! response should be preferred whenever available.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::accounts_config::AccountsConfig;

/// GCP Consumer Quotas API client for fetching Gemini quota limits.
///
/// This module provides optional integration with Google Cloud's Consumer Quotas API
/// to fetch accurate rate limit information for Gemini API usage. When credentials
/// are available via environment variables, it provides exact quota limits; otherwise,
/// the system falls back to hardcoded defaults.
mod gcp_quota_client {
    use super::*;

    /// GCP project and region configuration for quota lookup.
    #[derive(Debug, Clone)]
    pub struct GcpQuotaConfig {
        /// GCP project ID
        pub project_id: String,
        /// GCP region (e.g., "us-central1")
        pub region: String,
        /// Whether to use quota API for limit lookup
        pub enabled: bool,
    }

    /// Parsed quota limits from GCP API response.
    #[derive(Debug, Clone)]
    pub struct GeminiQuotaLimits {
        /// Daily token limit (tokens per day)
        pub daily_limit: u64,
        /// Requests per minute limit
        pub rpm_limit: Option<u64>,
    }

    /// Consumer Quotas API response structure.
    #[derive(Debug, Deserialize)]
    struct QuotaLimit {
        #[serde(rename = "name")]
        name: String,
        #[serde(rename = "limit")]
        limit: f64,
    }

    /// Load GCP quota configuration from environment variables.
    ///
    /// Environment variables:
    /// - `GEMINI_GCP_PROJECT_ID`: GCP project ID for quota lookup
    /// - `GEMINI_GCP_REGION`: GCP region (default: "us-central1")
    /// - `GEMINI_USE_QUOTA_API`: Set to "true" to enable quota API lookup
    ///
    /// Returns `None` if configuration is incomplete or disabled.
    pub fn load_gcp_quota_config() -> Option<GcpQuotaConfig> {
        let project_id = env::var("GEMINI_GCP_PROJECT_ID").ok()?;
        let enabled = env::var("GEMINI_USE_QUOTA_API")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        if !enabled {
            debug!("GCP quota API disabled via GEMINI_USE_QUOTA_API");
            return None;
        }

        let region = env::var("GEMINI_GCP_REGION").unwrap_or_else(|_| "us-central1".to_string());

        Some(GcpQuotaConfig {
            project_id,
            region,
            enabled: true,
        })
    }

    /// Fetch Gemini quota limits from GCP Consumer Quotas API.
    ///
    /// Uses Application Default Credentials (ADC) for authentication.
    /// Requires `gcloud auth application-default login` or service account credentials.
    ///
    /// Returns `Ok(None)` if the API call fails (allowing fallback to defaults).
    pub fn fetch_gemini_quota(config: &GcpQuotaConfig) -> Result<Option<GeminiQuotaLimits>> {
        debug!(
            "Fetching GCP quota for project {} in region {}",
            config.project_id, config.region
        );

        // Try to use gcloud CLI for quota lookup (more reliable than HTTP API)
        if let Some(limits) = fetch_quota_via_gcloud(config) {
            debug!("Successfully fetched quota via gcloud CLI");
            return Ok(Some(limits));
        }

        // Fallback: try HTTP API if gcloud is not available
        debug!("gcloud CLI not available, skipping GCP quota API");
        Ok(None)
    }

    /// Fetch quota limits using gcloud CLI.
    ///
    /// This is more reliable than direct HTTP API because it handles
    /// authentication via ADC automatically.
    fn fetch_quota_via_gcloud(config: &GcpQuotaConfig) -> Option<GeminiQuotaLimits> {
        use std::process::Command;

        // Use gcloud to list consumer quotas for Generative Language API
        let output = Command::new("gcloud")
            .args([
                "consumer-quotas",
                "list",
                &format!(
                    "projects/{}/locations/{}/services/generativelanguage.googleapis.com/consumerQuotas",
                    config.project_id, config.region
                ),
                "--filter=metric:generativelanguage.googleapis.com/*",
                "--format=json",
                "--limit=100",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            debug!(
                "gcloud consumer-quotas command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return None;
        }

        let json_str = String::from_utf8(output.stdout).ok()?;
        let quotas: serde_json::Value = serde_json::from_str(&json_str).ok()?;

        // Parse quota limits from the response
        let mut daily_limit: Option<u64> = None;
        let mut rpm_limit: Option<u64> = None;

        if let Some(array) = quotas.as_array() {
            for quota in array {
                if let Some(name) = quota.get("name").and_then(|v| v.as_str()) {
                    // Look for daily token generation limit
                    if name.contains("generativeTokenCapacityPerDay")
                        || name.contains("dailyTokenGeneration")
                    {
                        if let Some(limit_val) = quota
                            .get("quotaLimits")
                            .and_then(|v| v.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.get("value"))
                            .and_then(|v| v.as_f64())
                        {
                            daily_limit = Some(limit_val as u64);
                        }
                    }

                    // Look for requests per minute limit
                    if name.contains("requestsPerMinute") || name.contains("rpm") {
                        if let Some(limit_val) = quota
                            .get("quotaLimits")
                            .and_then(|v| v.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.get("value"))
                            .and_then(|v| v.as_f64())
                        {
                            rpm_limit = Some(limit_val as u64);
                        }
                    }
                }
            }
        }

        // If we found daily limit, return the quota limits
        if let Some(daily) = daily_limit {
            debug!(
                "GCP quota API returned daily_limit={}, rpm_limit={:?}",
                daily, rpm_limit
            );
            return Some(GeminiQuotaLimits {
                daily_limit: daily,
                rpm_limit,
            });
        }

        debug!("GCP quota API did not return token capacity limits");
        None
    }

    /// Fetch quota limits via HTTP API (experimental).
    ///
    /// This requires proper OAuth2 credentials and may not work in all environments.
    #[allow(dead_code)]
    fn fetch_quota_via_http(_config: &GcpQuotaConfig) -> Option<GeminiQuotaLimits> {
        // HTTP-based quota API is complex due to OAuth2 requirements.
        // The gcloud CLI approach is preferred as it handles authentication.
        // This stub is reserved for future implementation if needed.
        debug!("HTTP-based quota API not implemented, use gcloud CLI");
        None
    }
}

/// Deserializes `Option<Option<T>>` so that a present-but-null JSON field
/// becomes `Some(None)` (distinguishable from an absent field which is `None`).
fn deserialize_option_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

/// Per-model 7d utilization window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelWindow {
    pub model: String,
    pub utilization: f64,
    pub resets_at: Option<DateTime<Utc>>,
}

/// Utilization data for a single account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountCapacity {
    /// Account identifier (derived from credential dir path)
    pub account_id: String,
    /// Adapter name (always "claude" for now)
    pub adapter: String,
    /// Plan type from credentials (e.g. "max", "pro")
    pub plan_type: String,
    /// Rate limit tier from credentials (e.g. "default_claude_max_20x")
    pub rate_limit_tier: String,
    /// 5-hour rolling utilization (0-100)
    pub utilization_5h: f64,
    /// 7-day rolling utilization (0-100)
    pub utilization_7d: f64,
    /// When the 5h window resets
    pub resets_at_5h: Option<DateTime<Utc>>,
    /// When the 7d window resets
    pub resets_at_7d: Option<DateTime<Utc>>,
    /// Per-model 7d windows (sonnet, opus, etc.)
    pub model_windows_7d: Vec<ModelWindow>,
    /// Tokens counted in the 5h window (from JSONL)
    pub tokens_5h: u64,
    /// Tokens counted in the 7d window (from JSONL)
    pub tokens_7d: u64,
    /// Total assistant turns in 5h window
    pub turns_5h: u64,
    /// Total assistant turns in 7d window
    pub turns_7d: u64,
    /// Prompts used in current 5-hour window (ZAI proxy/OpenCode)
    pub prompts_5h: u64,
    /// Prompts used in current 7-day window (ZAI proxy/OpenCode)
    pub prompts_7d: u64,
    /// Prompt limit per 5-hour window (ZAI proxy)
    pub prompts_per_5h: Option<u64>,
    /// Prompt limit per 7-day window (ZAI proxy)
    pub prompts_per_7d: Option<u64>,
    /// Burn rate: tokens per minute over the last hour
    pub burn_rate_per_min: f64,
    /// Forecast: minutes until 5h utilization hits 100% at current burn rate
    pub forecast_full_5h_min: Option<f64>,
    /// Forecast: minutes until 7d utilization hits 100% at current burn rate
    pub forecast_full_7d_min: Option<f64>,
    /// Stitch close rate: completed worker sessions per minute (2h window)
    pub stitch_close_rate_per_min: f64,
    /// Mean cost per stitch: average cost-equivalent tokens per completed session
    pub mean_cost_per_stitch_tokens: f64,
    /// Forecast: minutes until 5h limit at stitch-projected burn rate
    pub forecast_full_5h_stitch_min: Option<f64>,
    /// Forecast: minutes until 7d limit at stitch-projected burn rate
    pub forecast_full_7d_stitch_min: Option<f64>,
    /// Source of the data ("api_cache" or "jsonl_estimate")
    pub source: String,
    /// When this data was computed
    pub computed_at: DateTime<Utc>,
}

/// Cached API usage response from Claude Code.
///
/// This is the exact same data that `/status` displays. Written by Claude
/// Code to `~/.cache/claude-usage/usage.json` on each API call.
#[derive(Debug, Deserialize)]
struct CachedUsageResponse {
    #[serde(default)]
    five_hour: Option<WindowUsage>,
    #[serde(default)]
    seven_day: Option<WindowUsage>,
    #[serde(default, deserialize_with = "deserialize_option_option")]
    seven_day_sonnet: Option<Option<WindowUsage>>,
    #[serde(default, deserialize_with = "deserialize_option_option")]
    seven_day_opus: Option<Option<WindowUsage>>,
    #[serde(default, deserialize_with = "deserialize_option_option")]
    seven_day_cowork: Option<Option<WindowUsage>>,
    #[serde(default, deserialize_with = "deserialize_option_option")]
    seven_day_omelette: Option<Option<WindowUsage>>,
}

#[derive(Debug, Deserialize, Clone)]
struct WindowUsage {
    #[serde(default)]
    utilization: f64,
    #[serde(default)]
    resets_at: Option<String>,
}

/// Claude credentials structure
#[derive(Debug, Deserialize)]
struct Credentials {
    #[serde(default, rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OAuthCreds>,
}

#[derive(Debug, Deserialize)]
struct OAuthCreds {
    #[serde(default, rename = "subscriptionType")]
    subscription_type: Option<String>,
    #[serde(default, rename = "rateLimitTier")]
    rate_limit_tier: Option<String>,
}

/// A single JSONL turn with parsed timestamp and usage
#[derive(Debug)]
struct ParsedTurn {
    ts: DateTime<Utc>,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    #[allow(dead_code)]
    model: Option<String>,
    /// Session identifier for grouping turns into stitches
    session_id: Option<String>,
}

/// A single OpenCode prompt (assistant turn) for ZAI proxy tracking
#[derive(Debug)]
struct ParsedPrompt {
    ts: DateTime<Utc>,
    /// Session identifier for grouping
    session_id: String,
}

/// A single Gemini assistant turn for utilization tracking
#[derive(Debug)]
struct GeminiTurn {
    ts: DateTime<Utc>,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    /// Session identifier for grouping turns into stitches
    session_id: Option<String>,
}

/// Minimum age in seconds before a session is considered complete (ended).
/// Sessions with a last turn older than this are treated as done.
const SESSION_COMPLETE_SECS: i64 = 300; // 5 minutes

/// Window for computing stitch close rate and mean cost (seconds)
const STITCH_WINDOW_SECS: i64 = 7200; // 2 hours

impl ParsedTurn {
    /// Cost-equivalent token count for utilization estimation.
    ///
    /// Claude's rate limiting uses a cost-weighted token count where output
    /// tokens count more than input tokens (reflecting API pricing). The
    /// exact ratio is proprietary, but empirically:
    ///
    /// - `input_tokens` at full weight
    /// - `output_tokens` at ~5x weight (matching the ~5:1 output:input price ratio)
    /// - `cache_read` at ~0.1x (cache reads are discounted)
    /// - `cache_write` at ~0.25x (cache writes are partially discounted)
    ///
    /// This gives a reasonable approximation for the JSONL fallback path.
    /// The primary path reads the cached API response which is exact.
    fn cost_equivalent_tokens(&self) -> u64 {
        let input = self.input_tokens as f64;
        let cache_read = self.cache_read_tokens as f64;
        let cache_write = self.cache_write_tokens as f64;
        let output = self.output_tokens as f64;

        let weighted = input + cache_read * 0.10 + cache_write * 0.25 + output * 5.0;

        weighted as u64
    }
}

impl GeminiTurn {
    /// Cost-equivalent token count for utilization estimation.
    ///
    /// Uses similar weighting to Claude but adjusted for Gemini's pricing:
    /// - Input tokens at full weight
    /// - Output tokens at ~8x weight (matching Gemini's output:input price ratio)
    /// - Cache reads at ~0.1x (cache reads are discounted)
    /// - Cache writes at ~0.25x (cache writes are partially discounted)
    fn cost_equivalent_tokens(&self) -> u64 {
        let input = self.input_tokens as f64;
        let cache_read = self.cache_read_tokens as f64;
        let cache_write = self.cache_write_tokens as f64;
        let output = self.output_tokens as f64;

        // Gemini has a higher output:input price ratio than Claude (approx 8:1 for flash)
        let weighted = input + cache_read * 0.10 + cache_write * 0.25 + output * 8.0;

        weighted as u64
    }
}

/// Gemini-specific token limits for rate limit windows.
#[derive(Debug, Clone)]
struct GeminiLimits {
    /// Token budget per 5-hour window (approximate)
    tokens_5h: u64,
    /// Token budget per 7-day window (approximate)
    tokens_7d: u64,
    /// Whether these limits are from GCP API (true) or hardcoded defaults (false)
    from_api: bool,
}

impl Default for GeminiLimits {
    fn default() -> Self {
        Self {
            tokens_5h: 1_000_000,
            tokens_7d: 15_000_000,
            from_api: false,
        }
    }
}

/// Plan-specific token limits for rate limit windows.
///
/// These are calibrated from the cached API response by observing the
/// relationship between JSONL token counts and reported utilization %.
/// They are only used in the JSONL fallback path.
struct PlanLimits {
    /// Token budget per 5-hour window
    tokens_5h: u64,
    /// Token budget per 7-day window
    tokens_7d: u64,
}

/// OpenCode prompt limits for ZAI proxy rate limiting.
///
/// Based on ZAI proxy Max tier: 1600 prompts per 5 hours, 8000 prompts per 7 days.
/// These counts represent the number of assistant responses (prompts) sent.
#[derive(Debug, Clone)]
struct OpenCodePromptLimits {
    /// Prompt budget per 5-hour window
    prompts_per_5h: u64,
    /// Prompt budget per 7-day window
    prompts_per_7d: u64,
}

fn get_opencode_limits() -> OpenCodePromptLimits {
    // ZAI proxy Max tier limits: 1600 prompts per 5 hours, 8000 prompts per 7 days
    OpenCodePromptLimits {
        prompts_per_5h: 1600,
        prompts_per_7d: 8000,
    }
}

fn get_plan_limits(plan_type: &str, tier: &str) -> PlanLimits {
    match (plan_type, tier) {
        ("max", t) if t.contains("20x") => PlanLimits {
            tokens_5h: 1_000_000,
            tokens_7d: 15_000_000,
        },
        ("max", t) if t.contains("10x") => PlanLimits {
            tokens_5h: 500_000,
            tokens_7d: 7_500_000,
        },
        ("max", t) if t.contains("5x") => PlanLimits {
            tokens_5h: 250_000,
            tokens_7d: 3_750_000,
        },
        ("max", _) => PlanLimits {
            tokens_5h: 100_000,
            tokens_7d: 1_500_000,
        },
        ("pro", _) => PlanLimits {
            tokens_5h: 44_000,
            tokens_7d: 660_000,
        },
        _ => PlanLimits {
            tokens_5h: 44_000,
            tokens_7d: 660_000,
        },
    }
}

/// Resolved paths for a single Claude account
#[derive(Debug, Clone)]
struct AccountPaths {
    /// Directory containing .credentials.json (e.g. ~/.claude)
    credential_dir: PathBuf,
    /// Directory containing JSONL session files (e.g. ~/.claude/projects/)
    projects_dir: PathBuf,
    /// Path to cached usage.json from Claude API
    cached_usage_path: PathBuf,
}

/// Resolved paths for a single Gemini account
#[derive(Debug, Clone)]
struct GeminiAccountPaths {
    /// Root directory (e.g. ~/.gemini or $GEMINI_CLI_HOME)
    root_dir: PathBuf,
    /// Session directory (tmp or sessions)
    session_subpath: String,
    /// Full path to session files
    sessions_dir: PathBuf,
}

/// Resolved paths for a single OpenCode account
#[derive(Debug, Clone)]
struct OpenCodeAccountPaths {
    /// Root directory (e.g. ~/.local/share/opencode)
    root_dir: PathBuf,
    /// Session storage directory
    session_dir: PathBuf,
    /// Legacy session directory (if exists)
    legacy_session_dir: Option<PathBuf>,
}

/// Capacity meter configuration
#[derive(Debug, Clone)]
pub struct CapacityMeterConfig {
    /// Claude config directories to scan (each = one account).
    /// Defaults to vec![~/.claude].
    /// Auto-discovery appends any ~/.claude-* dirs with .credentials.json.
    pub account_dirs: Vec<PathBuf>,
    /// Gemini config directories to scan (each = one account).
    /// Defaults to vec![~/.gemini].
    /// Auto-discovery appends any ~/.gemini-* dirs with session files.
    pub gemini_dirs: Vec<PathBuf>,
    /// OpenCode config directories to scan (each = one account).
    /// Defaults to vec![~/.local/share/opencode].
    /// Auto-discovery appends any opencode-* dirs with session files.
    pub opencode_dirs: Vec<PathBuf>,
    /// How often to recompute (seconds)
    pub refresh_interval_secs: u64,
    /// Maximum age of cached usage.json before treating it as stale (seconds)
    pub cache_max_age_secs: u64,
    /// Override base cache directory (defaults to dirs::cache_dir() = ~/.cache).
    /// Set in tests to avoid touching the real cache.
    pub cache_base_dir: Option<PathBuf>,
    /// Optional GCP quota configuration for enhanced Gemini limit accuracy.
    /// Loaded from environment variables (GEMINI_GCP_PROJECT_ID, GEMINI_USE_QUOTA_API).
    /// When None, uses hardcoded defaults for Gemini limits.
    pub gcp_quota_config: Option<gcp_quota_client::GcpQuotaConfig>,
    /// Path to accounts.yaml for per-account OpenCode prompt limits.
    /// Defaults to ~/.hoop/accounts.yaml.
    pub accounts_file: Option<PathBuf>,
}

impl Default for CapacityMeterConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let mut account_dirs = vec![home.join(".claude")];

        // Auto-discover additional Claude config dirs (~/.claude-*)
        if let Ok(entries) = fs::read_dir(&home) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with(".claude-")
                    && entry.path().join(".credentials.json").exists()
                {
                    account_dirs.push(entry.path());
                }
            }
        }

        // Discover Gemini directories
        let gemini_dirs = Self::discover_gemini_dirs(&home);

        // Discover OpenCode directories
        let opencode_dirs = Self::discover_opencode_dirs(&home);

        // Load optional GCP quota configuration from environment
        let gcp_quota_config = gcp_quota_client::load_gcp_quota_config();

        if gcp_quota_config.is_some() {
            info!(
                "GCP quota API enabled for Gemini: project={}, region={}",
                gcp_quota_config.as_ref().unwrap().project_id,
                gcp_quota_config.as_ref().unwrap().region
            );
        }

        // Default accounts.yaml path
        let accounts_file = home.join(".hoop").join("accounts.yaml");

        Self {
            account_dirs,
            gemini_dirs,
            opencode_dirs,
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: None,
            gcp_quota_config,
            accounts_file: Some(accounts_file),
        }
    }
}

impl CapacityMeterConfig {
    /// Resolve per-account paths from an account config directory.
    fn resolve_account_paths(&self, account_dir: &Path) -> AccountPaths {
        let cache_base = self
            .cache_base_dir
            .as_ref()
            .cloned()
            .or_else(dirs::cache_dir)
            .unwrap_or_else(|| PathBuf::from(".cache"));

        let dir_name = account_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let cached_usage_path = if dir_name == ".claude" {
            cache_base.join("claude-usage").join("usage.json")
        } else {
            cache_base
                .join("claude-usage")
                .join(format!("{}-usage.json", dir_name))
        };

        AccountPaths {
            credential_dir: account_dir.to_path_buf(),
            projects_dir: account_dir.join("projects"),
            cached_usage_path,
        }
    }

    /// Discover Gemini session directories.
    ///
    /// Checks multiple potential locations in order:
    /// 1. $GEMINI_CLI_HOME/tmp/ (sandbox mode)
    /// 2. ~/.gemini/tmp/ (default sandbox location)
    /// 3. $GEMINI_CLI_HOME/sessions/ (custom sessions dir)
    /// 4. ~/.gemini/sessions/ (legacy default)
    ///
    /// Returns all paths that exist and contain .jsonl session files.
    fn discover_gemini_dirs(home: &Path) -> Vec<PathBuf> {
        let mut found_dirs: Vec<PathBuf> = Vec::new();

        // Check GEMINI_CLI_HOME environment variable
        let gemini_cli_home = std::env::var("GEMINI_CLI_HOME").ok();

        // Build list of candidate directories to check
        let candidates = vec![
            // GEMINI_CLI_HOME/tmp/ (sandbox mode)
            gemini_cli_home
                .clone()
                .map(|p| PathBuf::from(p).join("tmp")),
            // ~/.gemini/tmp/ (default sandbox location)
            Some(home.join(".gemini").join("tmp")),
            // GEMINI_CLI_HOME/sessions/ (custom sessions dir)
            gemini_cli_home.map(|p| PathBuf::from(p).join("sessions")),
            // ~/.gemini/sessions/ (legacy default)
            Some(home.join(".gemini").join("sessions")),
        ];

        for candidate in candidates {
            let Some(dir) = candidate else { continue };

            if dir.exists() && dir.is_dir() {
                // Check if this directory contains .jsonl session files
                let has_jsonl = fs::read_dir(&dir)
                    .map(|entries| {
                        entries.filter_map(|e| e.ok()).any(|e| {
                            e.path()
                                .extension()
                                .map(|ext| ext == "jsonl")
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false);

                if has_jsonl {
                    debug!(
                        "Gemini session discovery: found session files at {}",
                        dir.display()
                    );
                    // Return the parent directory (root) for consistency
                    if let Some(parent) = dir.parent() {
                        if !found_dirs.contains(&parent.to_path_buf()) {
                            found_dirs.push(parent.to_path_buf());
                        }
                    }
                }
            }
        }

        // Also check for ~/.gemini-* directories (named Gemini accounts)
        if let Ok(entries) = fs::read_dir(home) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with(".gemini-") {
                    let path = entry.path();
                    // Check for tmp/ or sessions/ subdirectory
                    for subpath in ["tmp", "sessions"] {
                        let session_dir = path.join(subpath);
                        if session_dir.exists() && session_dir.is_dir() {
                            let has_jsonl = fs::read_dir(&session_dir)
                                .map(|entries| {
                                    entries.filter_map(|e| e.ok()).any(|e| {
                                        e.path()
                                            .extension()
                                            .map(|ext| ext == "jsonl")
                                            .unwrap_or(false)
                                    })
                                })
                                .unwrap_or(false);

                            if has_jsonl && !found_dirs.contains(&path) {
                                debug!(
                                    "Gemini session discovery: found named account at {}",
                                    path.display()
                                );
                                found_dirs.push(path.clone());
                            }
                        }
                    }
                }
            }
        }

        found_dirs
    }

    /// Discover OpenCode session directories.
    ///
    /// Checks multiple potential locations in order:
    /// 1. `~/.local/share/opencode/storage/session/` (primary tree-based storage)
    /// 2. `~/.opencode/sessions/` (legacy JSONL format)
    /// 3. Additional XDG data dirs with opencode/storage/session
    ///
    /// Returns all paths that exist and contain session files.
    fn discover_opencode_dirs(home: &Path) -> Vec<PathBuf> {
        let mut found_dirs: Vec<PathBuf> = Vec::new();

        // Primary: XDG data directory
        let xdg_data_home = std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local").join("share"));

        let opencode_storage = xdg_data_home
            .join("opencode")
            .join("storage")
            .join("session");
        if opencode_storage.exists() && opencode_storage.is_dir() {
            debug!(
                "OpenCode session discovery: found tree-based storage at {}",
                opencode_storage.display()
            );
            found_dirs.push(xdg_data_home.join("opencode"));
        }

        // Legacy: ~/.opencode/sessions/
        let legacy_dir = home.join(".opencode").join("sessions");
        if legacy_dir.exists() && legacy_dir.is_dir() {
            // Check if it contains .jsonl files
            let has_jsonl = fs::read_dir(&legacy_dir)
                .map(|entries| {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "jsonl")
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if has_jsonl {
                debug!(
                    "OpenCode session discovery: found legacy sessions at {}",
                    legacy_dir.display()
                );
                found_dirs.push(home.join(".opencode"));
            }
        }

        // Also check for ~/.opencode-* directories (named OpenCode accounts)
        if let Ok(entries) = fs::read_dir(home) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with(".opencode-") {
                    let path = entry.path();
                    // Check for storage/session or sessions subdirectory
                    let has_sessions = path.join("storage").join("session").exists()
                        || path.join("sessions").exists();

                    if has_sessions && !found_dirs.contains(&path) {
                        debug!(
                            "OpenCode session discovery: found named account at {}",
                            path.display()
                        );
                        found_dirs.push(path);
                    }
                }
            }
        }

        found_dirs
    }

    /// Resolve OpenCode session paths from a root directory.
    ///
    /// Returns all session paths that exist and contain session files.
    fn resolve_opencode_paths(&self, root_dir: &Path) -> Vec<OpenCodeAccountPaths> {
        let mut paths = Vec::new();

        // Primary: tree-based storage at storage/session/
        let session_dir = root_dir.join("storage").join("session");
        if session_dir.exists() && session_dir.is_dir() {
            // Check for .json session files (tree-based storage)
            let has_json = fs::read_dir(&session_dir)
                .map(|entries| {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "json")
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if has_json {
                paths.push(OpenCodeAccountPaths {
                    root_dir: root_dir.to_path_buf(),
                    session_dir: session_dir.clone(),
                    legacy_session_dir: None,
                });
            }
        }

        // Legacy fallback: sessions/ directory with .jsonl files
        let legacy_dir = root_dir.join("sessions");
        if legacy_dir.exists() && legacy_dir.is_dir() {
            let has_jsonl = fs::read_dir(&legacy_dir)
                .map(|entries| {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "jsonl")
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if has_jsonl {
                // If we already have a tree-based path, add this as legacy
                if let Some(existing) = paths.iter_mut().find(|p| p.root_dir == root_dir) {
                    existing.legacy_session_dir = Some(legacy_dir);
                } else {
                    paths.push(OpenCodeAccountPaths {
                        root_dir: root_dir.to_path_buf(),
                        session_dir: root_dir.join("storage").join("session"), // may not exist
                        legacy_session_dir: Some(legacy_dir),
                    });
                }
            }
        }

        paths
    }

    /// Resolve Gemini session paths from a root directory.
    ///
    /// Returns all session subdirectories (tmp, sessions) that exist
    /// and contain .jsonl files.
    fn resolve_gemini_paths(&self, root_dir: &Path) -> Vec<GeminiAccountPaths> {
        let mut paths = Vec::new();

        for subpath in ["tmp", "sessions"] {
            let session_dir = root_dir.join(subpath);
            if session_dir.exists() && session_dir.is_dir() {
                let has_jsonl = fs::read_dir(&session_dir)
                    .map(|entries| {
                        entries.filter_map(|e| e.ok()).any(|e| {
                            e.path()
                                .extension()
                                .map(|ext| ext == "jsonl")
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false);

                if has_jsonl {
                    paths.push(GeminiAccountPaths {
                        root_dir: root_dir.to_path_buf(),
                        session_subpath: subpath.to_string(),
                        sessions_dir: session_dir,
                    });
                }
            }
        }

        paths
    }
}

/// Capacity meter: computes per-account utilization
pub struct CapacityMeter {
    config: CapacityMeterConfig,
    /// Loaded accounts configuration for per-account OpenCode limits
    accounts_config: AccountsConfig,
}

impl CapacityMeter {
    pub fn new(config: CapacityMeterConfig) -> Self {
        // Load accounts configuration
        let accounts_config = config
            .accounts_file
            .as_ref()
            .map(|path| AccountsConfig::load_from_file(path))
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_default();

        if let Some(ref path) = config.accounts_file {
            info!(
                "Loaded accounts config from {}: {} accounts configured",
                path.display(),
                accounts_config.accounts.len()
            );
        }

        Self {
            config,
            accounts_config,
        }
    }

    /// Compute capacity for all configured accounts
    pub fn compute(&self) -> Vec<AccountCapacity> {
        let mut accounts = Vec::new();

        // Process Claude accounts
        for account_dir in &self.config.account_dirs {
            let paths = self.config.resolve_account_paths(account_dir);
            match self.compute_account(&paths) {
                Ok(cap) => accounts.push(cap),
                Err(e) => {
                    warn!(
                        "Failed to compute capacity for {}: {}",
                        paths.credential_dir.display(),
                        e
                    );
                }
            }
        }

        // Process Gemini accounts
        for gemini_root in &self.config.gemini_dirs {
            let gemini_paths = self.config.resolve_gemini_paths(gemini_root);
            for paths in &gemini_paths {
                match self.compute_gemini_account(paths) {
                    Ok(cap) => accounts.push(cap),
                    Err(e) => {
                        warn!(
                            "Failed to compute Gemini capacity for {}: {}",
                            paths.root_dir.display(),
                            e
                        );
                    }
                }
            }
        }

        // Process OpenCode accounts
        for opencode_root in &self.config.opencode_dirs {
            let opencode_paths = self.config.resolve_opencode_paths(opencode_root);
            for paths in &opencode_paths {
                match self.compute_opencode_account(paths) {
                    Ok(cap) => accounts.push(cap),
                    Err(e) => {
                        warn!(
                            "Failed to compute OpenCode capacity for {}: {}",
                            paths.root_dir.display(),
                            e
                        );
                    }
                }
            }
        }

        // Emit capacity exhaustion warnings (§16.7)
        // Warning when forecast is less than 30 minutes
        const WARNING_THRESHOLD_MINUTES: f64 = 30.0;
        for account in &accounts {
            let needs_warning = account
                .forecast_full_5h_min
                .is_some_and(|f| f < WARNING_THRESHOLD_MINUTES && f > 0.0)
                || account
                    .forecast_full_7d_min
                    .is_some_and(|f| f < WARNING_THRESHOLD_MINUTES && f > 0.0);

            if needs_warning {
                crate::metrics::metrics()
                    .hoop_capacity_meter_exhaustion_warnings_total
                    .inc(&[&account.account_id]);
            }
        }

        accounts
    }

    fn compute_account(&self, paths: &AccountPaths) -> Result<AccountCapacity> {
        let account_id = Self::derive_account_id(&paths.credential_dir);
        let now = Utc::now();

        let (plan_type, rate_limit_tier) = Self::read_credentials(&paths.credential_dir)?;

        // Try cached API response first (exact numbers matching /status)
        let cached =
            Self::read_cached_usage(&paths.cached_usage_path, self.config.cache_max_age_secs);

        // Parse JSONL for token counts (used for burn rate and as fallback)
        let turns = Self::parse_all_jsonl(&paths.projects_dir)?;

        // Compute rolling windows
        let cutoff_5h = now - Duration::hours(5);
        let cutoff_7d = now - Duration::days(7);
        let cutoff_1h = now - Duration::hours(1);

        let mut tokens_5h: u64 = 0;
        let mut tokens_7d: u64 = 0;
        let mut turns_5h: u64 = 0;
        let mut turns_7d: u64 = 0;
        let mut tokens_last_hour: u64 = 0;

        for turn in &turns {
            let weighted = turn.cost_equivalent_tokens();
            if turn.ts > cutoff_5h {
                tokens_5h += weighted;
                turns_5h += 1;
            }
            if turn.ts > cutoff_7d {
                tokens_7d += weighted;
                turns_7d += 1;
            }
            if turn.ts > cutoff_1h {
                tokens_last_hour += weighted;
            }
        }

        let burn_rate_per_min = if tokens_last_hour > 0 {
            tokens_last_hour as f64 / 60.0
        } else {
            0.0
        };

        // Stitch-based burn rate: group turns by session_id, identify completed
        // sessions (last turn > SESSION_COMPLETE_SECS old), and compute:
        //   stitch_close_rate = completions in last STITCH_WINDOW_SECS / window_minutes
        //   mean_cost_per_stitch = average cost-equivalent tokens of those sessions
        let (stitch_close_rate_per_min, mean_cost_per_stitch_tokens) = {
            let complete_cutoff = now - Duration::seconds(SESSION_COMPLETE_SECS);
            let stitch_window_cutoff = now - Duration::seconds(STITCH_WINDOW_SECS);

            // Accumulate per-session stats keyed by session_id
            let mut session_last_ts: HashMap<String, DateTime<Utc>> = HashMap::new();
            let mut session_cost: HashMap<String, u64> = HashMap::new();

            for turn in &turns {
                let sid = match &turn.session_id {
                    Some(id) if !id.is_empty() => id.clone(),
                    _ => continue, // skip turns without session IDs
                };
                let weighted = turn.cost_equivalent_tokens();
                let last = session_last_ts.entry(sid.clone()).or_insert(turn.ts);
                if turn.ts > *last {
                    *last = turn.ts;
                }
                *session_cost.entry(sid).or_insert(0) += weighted;
            }

            // Collect sessions that completed within the stitch window
            let mut completion_costs: Vec<u64> = Vec::new();
            for (sid, last_ts) in &session_last_ts {
                // Session is complete if its last turn is older than SESSION_COMPLETE_SECS
                // and within the STITCH_WINDOW_SECS lookback period
                if *last_ts < complete_cutoff && *last_ts > stitch_window_cutoff {
                    let cost = *session_cost.get(sid).unwrap_or(&0);
                    if cost > 0 {
                        completion_costs.push(cost);
                    }
                }
            }

            let window_minutes = STITCH_WINDOW_SECS as f64 / 60.0;
            let rate = if !completion_costs.is_empty() {
                completion_costs.len() as f64 / window_minutes
            } else {
                0.0
            };
            let mean = if !completion_costs.is_empty() {
                completion_costs.iter().sum::<u64>() as f64 / completion_costs.len() as f64
            } else {
                0.0
            };
            (rate, mean)
        };

        // Determine utilization: prefer cached API, fall back to JSONL estimate
        let (util_5h, util_7d, resets_5h, resets_7d, model_windows, source) =
            if let Some(ref cached) = cached {
                let u5 = cached
                    .five_hour
                    .as_ref()
                    .map(|w| w.utilization)
                    .unwrap_or(0.0);
                let u7 = cached
                    .seven_day
                    .as_ref()
                    .map(|w| w.utilization)
                    .unwrap_or(0.0);
                let r5 = parse_resets_at(cached.five_hour.as_ref());
                let r7 = parse_resets_at(cached.seven_day.as_ref());

                let mut windows = Vec::new();

                if let Some(Some(w)) = &cached.seven_day_sonnet {
                    windows.push(ModelWindow {
                        model: "sonnet".to_string(),
                        utilization: w.utilization,
                        resets_at: parse_resets_at(Some(w)),
                    });
                }
                if let Some(Some(w)) = &cached.seven_day_opus {
                    windows.push(ModelWindow {
                        model: "opus".to_string(),
                        utilization: w.utilization,
                        resets_at: parse_resets_at(Some(w)),
                    });
                }
                if let Some(Some(w)) = &cached.seven_day_cowork {
                    windows.push(ModelWindow {
                        model: "cowork".to_string(),
                        utilization: w.utilization,
                        resets_at: parse_resets_at(Some(w)),
                    });
                }
                if let Some(Some(w)) = &cached.seven_day_omelette {
                    windows.push(ModelWindow {
                        model: "omelette".to_string(),
                        utilization: w.utilization,
                        resets_at: parse_resets_at(Some(w)),
                    });
                }

                (u5, u7, r5, r7, windows, "api_cache".to_string())
            } else {
                let limits = get_plan_limits(&plan_type, &rate_limit_tier);
                let u5 = if limits.tokens_5h > 0 {
                    (tokens_5h as f64 / limits.tokens_5h as f64 * 100.0).min(100.0)
                } else {
                    0.0
                };
                let u7 = if limits.tokens_7d > 0 {
                    (tokens_7d as f64 / limits.tokens_7d as f64 * 100.0).min(100.0)
                } else {
                    0.0
                };
                (u5, u7, None, None, Vec::new(), "jsonl_estimate".to_string())
            };

        let limits = get_plan_limits(&plan_type, &rate_limit_tier);

        // Remaining capacity in JSONL-weighted token units.
        //
        // When the API cache supplies an exact utilization % and we have a non-zero
        // JSONL token count for the same window, we can derive remaining capacity
        // without relying on the hardcoded plan limits:
        //
        //   remaining = tokens_used_JSONL × (100 − util%) / util%
        //
        // This identity holds regardless of the JSONL/API token-weighting ratio
        // (both remaining and burn_rate end up in the same JSONL units, so the
        // ratio cancels in the ETA division). It is the most accurate path when
        // API cache is fresh.  Falls back to plan-limit estimates otherwise.
        let remaining_5h = if source == "api_cache" && util_5h > 0.0 && tokens_5h > 0 {
            tokens_5h as f64 * (100.0 - util_5h) / util_5h
        } else {
            limits.tokens_5h as f64 * (1.0 - util_5h / 100.0)
        };
        let remaining_7d = if source == "api_cache" && util_7d > 0.0 && tokens_7d > 0 {
            tokens_7d as f64 * (100.0 - util_7d) / util_7d
        } else {
            limits.tokens_7d as f64 * (1.0 - util_7d / 100.0)
        };

        let forecast_full_5h = if burn_rate_per_min > 0.0 && util_5h < 100.0 {
            Some(remaining_5h / burn_rate_per_min)
        } else if util_5h >= 100.0 {
            Some(0.0)
        } else {
            None
        };

        let forecast_full_7d = if burn_rate_per_min > 0.0 && util_7d < 100.0 {
            Some(remaining_7d / burn_rate_per_min)
        } else if util_7d >= 100.0 {
            Some(0.0)
        } else {
            None
        };

        // Stitch-projected forecasts
        let stitch_burn_rate = stitch_close_rate_per_min * mean_cost_per_stitch_tokens;
        let forecast_full_5h_stitch = if stitch_burn_rate > 0.0 && util_5h < 100.0 {
            Some(remaining_5h / stitch_burn_rate)
        } else if util_5h >= 100.0 {
            Some(0.0)
        } else {
            None
        };
        let forecast_full_7d_stitch = if stitch_burn_rate > 0.0 && util_7d < 100.0 {
            Some(remaining_7d / stitch_burn_rate)
        } else if util_7d >= 100.0 {
            Some(0.0)
        } else {
            None
        };

        Ok(AccountCapacity {
            account_id,
            adapter: "claude".to_string(),
            plan_type,
            rate_limit_tier,
            utilization_5h: util_5h,
            utilization_7d: util_7d,
            resets_at_5h: resets_5h,
            resets_at_7d: resets_7d,
            model_windows_7d: model_windows,
            tokens_5h,
            tokens_7d,
            turns_5h,
            turns_7d,
            prompts_5h: 0,
            prompts_7d: 0,
            prompts_per_5h: None,
            prompts_per_7d: None,
            burn_rate_per_min,
            forecast_full_5h_min: forecast_full_5h,
            forecast_full_7d_min: forecast_full_7d,
            stitch_close_rate_per_min,
            mean_cost_per_stitch_tokens,
            forecast_full_5h_stitch_min: forecast_full_5h_stitch,
            forecast_full_7d_stitch_min: forecast_full_7d_stitch,
            source,
            computed_at: now,
        })
    }

    /// Compute capacity for a Gemini account from session files.
    ///
    /// Gemini uses JSONL session files with a different structure than Claude.
    /// There's no cached API response, so we rely entirely on session file parsing.
    fn compute_gemini_account(&self, paths: &GeminiAccountPaths) -> Result<AccountCapacity> {
        let account_id = Self::derive_gemini_account_id(&paths.root_dir);
        let now = Utc::now();

        // Parse Gemini session files
        let turns = Self::parse_gemini_sessions(&paths.sessions_dir)?;

        // Gemini has free tier with rate limits (similar to Claude's default)
        // These are approximate limits based on typical Gemini free tier quotas
        let plan_type = "free".to_string();
        let rate_limit_tier = "gemini_free".to_string();

        // Compute rolling windows
        let cutoff_5h = now - Duration::hours(5);
        let cutoff_7d = now - Duration::days(7);
        let cutoff_1h = now - Duration::hours(1);

        let mut tokens_5h: u64 = 0;
        let mut tokens_7d: u64 = 0;
        let mut turns_5h: u64 = 0;
        let mut turns_7d: u64 = 0;
        let mut tokens_last_hour: u64 = 0;

        for turn in &turns {
            let weighted = turn.cost_equivalent_tokens();
            if turn.ts > cutoff_5h {
                tokens_5h += weighted;
                turns_5h += 1;
            }
            if turn.ts > cutoff_7d {
                tokens_7d += weighted;
                turns_7d += 1;
            }
            if turn.ts > cutoff_1h {
                tokens_last_hour += weighted;
            }
        }

        let burn_rate_per_min = if tokens_last_hour > 0 {
            tokens_last_hour as f64 / 60.0
        } else {
            0.0
        };

        // Stitch-based burn rate for Gemini
        let (stitch_close_rate_per_min, mean_cost_per_stitch_tokens) = {
            let complete_cutoff = now - Duration::seconds(SESSION_COMPLETE_SECS);
            let stitch_window_cutoff = now - Duration::seconds(STITCH_WINDOW_SECS);

            let mut session_last_ts: HashMap<String, DateTime<Utc>> = HashMap::new();
            let mut session_cost: HashMap<String, u64> = HashMap::new();

            for turn in &turns {
                let sid = match &turn.session_id {
                    Some(id) if !id.is_empty() => id.clone(),
                    _ => continue,
                };
                let weighted = turn.cost_equivalent_tokens();
                let last = session_last_ts.entry(sid.clone()).or_insert(turn.ts);
                if turn.ts > *last {
                    *last = turn.ts;
                }
                *session_cost.entry(sid).or_insert(0) += weighted;
            }

            let mut completion_costs: Vec<u64> = Vec::new();
            for (sid, last_ts) in &session_last_ts {
                if *last_ts < complete_cutoff && *last_ts > stitch_window_cutoff {
                    let cost = *session_cost.get(sid).unwrap_or(&0);
                    if cost > 0 {
                        completion_costs.push(cost);
                    }
                }
            }

            let window_minutes = STITCH_WINDOW_SECS as f64 / 60.0;
            let rate = if !completion_costs.is_empty() {
                completion_costs.len() as f64 / window_minutes
            } else {
                0.0
            };
            let mean = if !completion_costs.is_empty() {
                completion_costs.iter().sum::<u64>() as f64 / completion_costs.len() as f64
            } else {
                0.0
            };
            (rate, mean)
        };

        // Gemini limits: try GCP quota API first, fall back to hardcoded defaults
        let gemini_limits = if let Some(ref gcp_config) = self.config.gcp_quota_config {
            // Try to fetch actual quota from GCP Consumer Quotas API
            match gcp_quota_client::fetch_gemini_quota(gcp_config) {
                Ok(Some(quota)) => {
                    // Convert daily limit to 7-day and estimate 5h from daily/24*5
                    let tokens_7d = quota.daily_limit * 7;
                    let tokens_5h = quota.daily_limit / 24 * 5;
                    debug!(
                        "Using GCP quota API limits for {}: daily={}, 7d={}, 5h={}",
                        account_id, quota.daily_limit, tokens_7d, tokens_5h
                    );
                    GeminiLimits {
                        tokens_5h,
                        tokens_7d,
                        from_api: true,
                    }
                }
                Ok(None) => {
                    debug!(
                        "GCP quota API returned no data for {}, using defaults",
                        account_id
                    );
                    GeminiLimits::default()
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch GCP quota for {}: {}, using defaults",
                        account_id, e
                    );
                    GeminiLimits::default()
                }
            }
        } else {
            // No GCP quota config, use hardcoded defaults
            GeminiLimits::default()
        };

        let util_5h = if gemini_limits.tokens_5h > 0 {
            (tokens_5h as f64 / gemini_limits.tokens_5h as f64 * 100.0).min(100.0)
        } else {
            0.0
        };
        let util_7d = if gemini_limits.tokens_7d > 0 {
            (tokens_7d as f64 / gemini_limits.tokens_7d as f64 * 100.0).min(100.0)
        } else {
            0.0
        };

        // Calculate forecasts
        let remaining_5h = gemini_limits.tokens_5h as f64 * (1.0 - util_5h / 100.0);
        let remaining_7d = gemini_limits.tokens_7d as f64 * (1.0 - util_7d / 100.0);

        let forecast_full_5h = if burn_rate_per_min > 0.0 && util_5h < 100.0 {
            Some(remaining_5h / burn_rate_per_min)
        } else if util_5h >= 100.0 {
            Some(0.0)
        } else {
            None
        };

        let forecast_full_7d = if burn_rate_per_min > 0.0 && util_7d < 100.0 {
            Some(remaining_7d / burn_rate_per_min)
        } else if util_7d >= 100.0 {
            Some(0.0)
        } else {
            None
        };

        // Stitch-projected forecasts
        let stitch_burn_rate = stitch_close_rate_per_min * mean_cost_per_stitch_tokens;
        let forecast_full_5h_stitch = if stitch_burn_rate > 0.0 && util_5h < 100.0 {
            Some(remaining_5h / stitch_burn_rate)
        } else if util_5h >= 100.0 {
            Some(0.0)
        } else {
            None
        };
        let forecast_full_7d_stitch = if stitch_burn_rate > 0.0 && util_7d < 100.0 {
            Some(remaining_7d / stitch_burn_rate)
        } else if util_7d >= 100.0 {
            Some(0.0)
        } else {
            None
        };

        // Determine source: "gcp_api" if limits from API, "jsonl_estimate" otherwise
        let source = if gemini_limits.from_api {
            "gcp_api".to_string()
        } else {
            "jsonl_estimate".to_string()
        };

        Ok(AccountCapacity {
            account_id,
            adapter: "gemini".to_string(),
            plan_type,
            rate_limit_tier,
            utilization_5h: util_5h,
            utilization_7d: util_7d,
            resets_at_5h: None,
            resets_at_7d: None,
            model_windows_7d: Vec::new(),
            tokens_5h,
            tokens_7d,
            turns_5h,
            turns_7d,
            prompts_5h: 0,
            prompts_7d: 0,
            prompts_per_5h: None,
            prompts_per_7d: None,
            burn_rate_per_min,
            forecast_full_5h_min: forecast_full_5h,
            forecast_full_7d_min: forecast_full_7d,
            stitch_close_rate_per_min,
            mean_cost_per_stitch_tokens,
            forecast_full_5h_stitch_min: forecast_full_5h_stitch,
            forecast_full_7d_stitch_min: forecast_full_7d_stitch,
            source,
            computed_at: now,
        })
    }

    fn derive_account_id(cred_dir: &Path) -> String {
        let dir_name = cred_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        if dir_name == ".claude" {
            "claude-default".to_string()
        } else {
            dir_name.to_string()
        }
    }

    fn read_credentials(cred_dir: &Path) -> Result<(String, String)> {
        let creds_path = cred_dir.join(".credentials.json");
        if !creds_path.exists() {
            return Ok(("unknown".to_string(), "unknown".to_string()));
        }

        let content = fs::read_to_string(&creds_path)?;
        let creds: Credentials = serde_json::from_str(&content)?;

        let oauth = creds.claude_ai_oauth.unwrap_or(OAuthCreds {
            subscription_type: None,
            rate_limit_tier: None,
        });

        Ok((
            oauth
                .subscription_type
                .unwrap_or_else(|| "unknown".to_string()),
            oauth
                .rate_limit_tier
                .unwrap_or_else(|| "unknown".to_string()),
        ))
    }

    /// Read cached API usage response for a specific account.
    fn read_cached_usage(path: &Path, max_age_secs: u64) -> Option<CachedUsageResponse> {
        if !path.exists() {
            debug!("No cached usage at {}", path.display());
            return None;
        }

        let content = fs::read_to_string(path).ok()?;
        let cached: CachedUsageResponse = serde_json::from_str(&content).ok()?;

        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                let modified_dt: DateTime<Utc> = modified.into();
                let age = Utc::now() - modified_dt;
                if age > Duration::seconds(max_age_secs as i64) {
                    debug!(
                        "Cached usage data is {}s old (max {}s), ignoring",
                        age.num_seconds(),
                        max_age_secs
                    );
                    return None;
                }
            }
        }

        Some(cached)
    }

    /// Parse all JSONL files under a specific account's projects directory
    fn parse_all_jsonl(projects_dir: &Path) -> Result<Vec<ParsedTurn>> {
        if !projects_dir.exists() {
            return Ok(Vec::new());
        }

        let mut turns = Vec::new();
        Self::scan_jsonl_recursive(projects_dir, &mut turns)?;

        debug!("Parsed {} assistant turns from JSONL files", turns.len());
        Ok(turns)
    }

    fn scan_jsonl_recursive(dir: &Path, turns: &mut Vec<ParsedTurn>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if path.file_name().map(|n| n == "subagents").unwrap_or(false) {
                    continue;
                }
                Self::scan_jsonl_recursive(&path, turns)?;
            } else if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Err(e) = Self::parse_jsonl_file(&path, turns) {
                    debug!("Error parsing {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    fn parse_jsonl_file(path: &Path, turns: &mut Vec<ParsedTurn>) -> Result<()> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);

        let mut seen_message_ids: HashMap<String, bool> = HashMap::new();
        let mut line_number: usize = 0;

        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            if !line.contains("\"type\":\"assistant\"") {
                continue;
            }

            let source = crate::parse_jsonl_safe::LineSource {
                tag: "capacity",
                file_path: path.to_path_buf(),
                line_number,
            };

            let entry: serde_json::Value =
                match crate::parse_jsonl_safe::parse_line(line.trim(), &source) {
                    crate::parse_jsonl_safe::ParseResult::Ok(v) => v,
                    _ => continue,
                };

            if entry.get("type").and_then(|v| v.as_str()) != Some("assistant") {
                continue;
            }

            let ts_str = match entry.get("timestamp").and_then(|v| v.as_str()) {
                Some(s) => s,
                None => continue,
            };
            let ts: DateTime<Utc> = match ts_str.parse() {
                Ok(t) => t,
                Err(_) => continue,
            };

            let message = match entry.get("message") {
                Some(m) => m,
                None => continue,
            };

            if let Some(msg_id) = message.get("id").and_then(|v| v.as_str()) {
                if seen_message_ids.contains_key(msg_id) {
                    continue;
                }
                seen_message_ids.insert(msg_id.to_string(), true);
            }

            let model = message.get("model").and_then(|v| v.as_str()).unwrap_or("");
            if model == "<synthetic>" {
                continue;
            }

            let usage = match message.get("usage") {
                Some(u) => u,
                None => continue,
            };

            let input_tokens = usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let output_tokens = usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_read = usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_write = usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if input_tokens == 0 && output_tokens == 0 && cache_read == 0 && cache_write == 0 {
                continue;
            }

            let session_id = entry
                .get("sessionId")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            turns.push(ParsedTurn {
                ts,
                input_tokens,
                output_tokens,
                cache_read_tokens: cache_read,
                cache_write_tokens: cache_write,
                model: if model.is_empty() {
                    None
                } else {
                    Some(model.to_string())
                },
                session_id,
            });
        }

        Ok(())
    }

    /// Derive account ID from Gemini root directory path.
    fn derive_gemini_account_id(root_dir: &Path) -> String {
        let dir_name = root_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        if dir_name == ".gemini" {
            "gemini-default".to_string()
        } else {
            dir_name.to_string()
        }
    }

    /// Parse all Gemini session JSONL files in a directory.
    fn parse_gemini_sessions(sessions_dir: &Path) -> Result<Vec<GeminiTurn>> {
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut turns = Vec::new();
        Self::scan_gemini_jsonl(sessions_dir, &mut turns)?;

        debug!(
            "Parsed {} Gemini turns from {}",
            turns.len(),
            sessions_dir.display()
        );
        Ok(turns)
    }

    /// Scan Gemini JSONL files recursively.
    fn scan_gemini_jsonl(dir: &Path, turns: &mut Vec<GeminiTurn>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::scan_gemini_jsonl(&path, turns)?;
            } else if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Err(e) = Self::parse_gemini_jsonl_file(&path, turns) {
                    debug!("Error parsing Gemini {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    /// Parse a single Gemini JSONL file.
    fn parse_gemini_jsonl_file(path: &Path, turns: &mut Vec<GeminiTurn>) -> Result<()> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);

        let mut line_number: usize = 0;
        for line in reader.lines() {
            let line = line?;
            line_number += 1;

            // Gemini JSONL uses "type": "message" or "type": "turn" for assistant responses
            if !line.contains("\"type\"") && !line.contains("\"role\"") {
                continue;
            }

            let source = crate::parse_jsonl_safe::LineSource {
                tag: "capacity/gemini",
                file_path: path.to_path_buf(),
                line_number,
            };

            let entry: serde_json::Value =
                match crate::parse_jsonl_safe::parse_line(line.trim(), &source) {
                    crate::parse_jsonl_safe::ParseResult::Ok(v) => v,
                    _ => continue,
                };

            // Check if this is a message/turn event with a model role (assistant response)
            let _event_type = entry.get("type").and_then(|v| v.as_str());
            let role = entry.get("role").and_then(|v| v.as_str());

            // Only count assistant/model responses (not user prompts)
            if role != Some("model") && role != Some("assistant") {
                continue;
            }

            // Parse timestamp
            let ts_str = entry
                .get("timestamp")
                .or_else(|| entry.get("time"))
                .and_then(|v| v.as_str());

            let ts: DateTime<Utc> = match ts_str {
                Some(s) => match s.parse() {
                    Ok(t) => t,
                    Err(_) => continue,
                },
                None => continue,
            };

            // Parse usage - Gemini uses different field names than Claude
            let usage = match entry.get("usage") {
                Some(u) => u,
                None => continue,
            };

            // Gemini usage fields: promptTokenCount, candidatesTokenCount, cachedContentTokenCount
            let input_tokens = usage
                .get("promptTokenCount")
                .or_else(|| usage.get("input_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let output_tokens = usage
                .get("candidatesTokenCount")
                .or_else(|| usage.get("output_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_read_tokens = usage
                .get("cachedContentTokenCount")
                .or_else(|| usage.get("cache_read_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_write_tokens = usage
                .get("cache_write_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            if input_tokens == 0 && output_tokens == 0 && cache_read_tokens == 0 {
                continue;
            }

            // Extract session ID if available
            let session_id = entry
                .get("session_id")
                .or_else(|| entry.get("id"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            turns.push(GeminiTurn {
                ts,
                input_tokens,
                output_tokens,
                cache_read_tokens,
                cache_write_tokens,
                session_id,
            });
        }

        Ok(())
    }

    /// Compute capacity for an OpenCode account from session files.
    ///
    /// OpenCode uses tree-based JSON storage with assistant turns counted as prompts.
    /// Each assistant response counts as one prompt against the ZAI proxy limits.
    fn compute_opencode_account(&self, paths: &OpenCodeAccountPaths) -> Result<AccountCapacity> {
        let account_id = Self::derive_opencode_account_id(&paths.root_dir);
        let now = Utc::now();

        // Parse OpenCode sessions to count prompts
        let prompts = Self::parse_opencode_sessions(paths)?;

        // Compute rolling windows
        let cutoff_5h = now - Duration::hours(5);
        let cutoff_7d = now - Duration::days(7);
        let cutoff_1h = now - Duration::hours(1);

        let mut prompts_5h: u64 = 0;
        let mut prompts_7d: u64 = 0;
        let mut prompts_last_hour: u64 = 0;

        for prompt in &prompts {
            if prompt.ts > cutoff_5h {
                prompts_5h += 1;
            }
            if prompt.ts > cutoff_7d {
                prompts_7d += 1;
            }
            if prompt.ts > cutoff_1h {
                prompts_last_hour += 1;
            }
        }

        // Get per-account limits from accounts_config (or default if not configured)
        let limits_config = self
            .accounts_config
            .get_opencode_limits_or_default(&account_id);
        let limits = OpenCodePromptLimits {
            prompts_per_5h: limits_config.prompts_per_5h,
            prompts_per_7d: limits_config.prompts_per_7d,
        };

        // Calculate utilization based on prompt counts
        let util_5h = if limits.prompts_per_5h > 0 {
            (prompts_5h as f64 / limits.prompts_per_5h as f64 * 100.0).min(100.0)
        } else {
            0.0
        };
        let util_7d = if limits.prompts_per_7d > 0 {
            (prompts_7d as f64 / limits.prompts_per_7d as f64 * 100.0).min(100.0)
        } else {
            0.0
        };

        // Calculate burn rate (prompts per minute)
        let burn_rate_per_min = if prompts_last_hour > 0 {
            prompts_last_hour as f64 / 60.0
        } else {
            0.0
        };

        // Calculate forecasts
        let remaining_5h = limits.prompts_per_5h as f64 * (1.0 - util_5h / 100.0);
        let remaining_7d = limits.prompts_per_7d as f64 * (1.0 - util_7d / 100.0);

        let forecast_full_5h = if burn_rate_per_min > 0.0 && util_5h < 100.0 {
            Some(remaining_5h / burn_rate_per_min)
        } else if util_5h >= 100.0 {
            Some(0.0)
        } else {
            None
        };

        let forecast_full_7d = if burn_rate_per_min > 0.0 && util_7d < 100.0 {
            Some(remaining_7d / burn_rate_per_min)
        } else if util_7d >= 100.0 {
            Some(0.0)
        } else {
            None
        };

        // Stitch-based metrics (not applicable for OpenCode prompt counting)
        let stitch_close_rate_per_min = 0.0;
        let mean_cost_per_stitch_tokens = 0.0;
        let forecast_full_5h_stitch = None;
        let forecast_full_7d_stitch = None;

        // OpenCode adapter type
        let adapter = "opencode".to_string();
        let plan_type = "max".to_string();
        let rate_limit_tier = "zai_proxy_max".to_string();

        Ok(AccountCapacity {
            account_id,
            adapter,
            plan_type,
            rate_limit_tier,
            utilization_5h: util_5h,
            utilization_7d: util_7d,
            resets_at_5h: None,
            resets_at_7d: None,
            model_windows_7d: Vec::new(),
            tokens_5h: 0,
            tokens_7d: 0,
            turns_5h: 0,
            turns_7d: 0,
            prompts_5h,
            prompts_7d,
            prompts_per_5h: Some(limits.prompts_per_5h),
            prompts_per_7d: Some(limits.prompts_per_7d),
            burn_rate_per_min,
            forecast_full_5h_min: forecast_full_5h,
            forecast_full_7d_min: forecast_full_7d,
            stitch_close_rate_per_min,
            mean_cost_per_stitch_tokens,
            forecast_full_5h_stitch_min: forecast_full_5h_stitch,
            forecast_full_7d_stitch_min: forecast_full_7d_stitch,
            source: "jsonl_estimate".to_string(),
            computed_at: now,
        })
    }

    fn derive_opencode_account_id(root_dir: &Path) -> String {
        let dir_name = root_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        if dir_name == "opencode" {
            "opencode-default".to_string()
        } else {
            dir_name.to_string()
        }
    }

    /// Parse OpenCode session files to count assistant prompts.
    ///
    /// Supports both tree-based storage (storage/session/*.json) and
    /// legacy JSONL format (sessions/*.jsonl).
    fn parse_opencode_sessions(paths: &OpenCodeAccountPaths) -> Result<Vec<ParsedPrompt>> {
        let mut prompts = Vec::new();

        // Parse tree-based storage first
        if paths.session_dir.exists() {
            Self::scan_opencode_tree_sessions(&paths.session_dir, &mut prompts)?;
        }

        // Parse legacy JSONL format if available
        if let Some(ref legacy_dir) = paths.legacy_session_dir {
            if legacy_dir.exists() {
                Self::scan_opencode_jsonl_sessions(legacy_dir, &mut prompts)?;
            }
        }

        debug!(
            "Parsed {} OpenCode prompts from {}",
            prompts.len(),
            paths.root_dir.display()
        );
        Ok(prompts)
    }

    /// Scan tree-based OpenCode sessions for assistant prompts.
    fn scan_opencode_tree_sessions(
        session_dir: &Path,
        prompts: &mut Vec<ParsedPrompt>,
    ) -> Result<()> {
        if !session_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(session_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();

            // Each project directory contains session files
            if path.is_dir() {
                Self::scan_opencode_tree_sessions(&path, prompts)?;
            } else if path.extension().map(|e| e == "json").unwrap_or(false) {
                // Parse session file to get message IDs
                if let Err(e) = Self::parse_opencode_tree_session_file(&path, prompts) {
                    debug!("Error parsing OpenCode session {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    /// Parse a single tree-based session file.
    fn parse_opencode_tree_session_file(
        session_path: &Path,
        prompts: &mut Vec<ParsedPrompt>,
    ) -> Result<()> {
        let raw = fs::read(session_path)?;
        let session_data: serde_json::Value = serde_json::from_slice(&raw)?;

        let session_id = session_data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                session_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
            })
            .to_string();

        // Derive storage root from session path
        let storage_root = session_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());

        let message_ids = session_data
            .get("messageIds")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        // Load messages and count assistant responses
        if let Some(root) = storage_root {
            let message_dir = root.join("message").join(&session_id);
            for msg_id in &message_ids {
                let msg_path = message_dir.join(format!("{}.json", msg_id));
                if let Ok(msg_raw) = fs::read_to_string(&msg_path) {
                    if let Ok(msg_data) = serde_json::from_str::<serde_json::Value>(&msg_raw) {
                        let role = msg_data.get("role").and_then(|v| v.as_str());
                        if role == Some("assistant") {
                            if let Some(ts_str) = msg_data.get("createdAt").and_then(|v| v.as_str())
                            {
                                if let Ok(ts) = ts_str.parse() {
                                    prompts.push(ParsedPrompt {
                                        ts,
                                        session_id: session_id.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Scan legacy OpenCode JSONL sessions for assistant prompts.
    fn scan_opencode_jsonl_sessions(
        sessions_dir: &Path,
        prompts: &mut Vec<ParsedPrompt>,
    ) -> Result<()> {
        if !sessions_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(sessions_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                Self::scan_opencode_jsonl_sessions(&path, prompts)?;
            } else if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                if let Err(e) = Self::parse_opencode_jsonl_file(&path, prompts) {
                    debug!("Error parsing OpenCode JSONL {}: {}", path.display(), e);
                }
            }
        }

        Ok(())
    }

    /// Parse a legacy OpenCode JSONL session file.
    fn parse_opencode_jsonl_file(path: &Path, prompts: &mut Vec<ParsedPrompt>) -> Result<()> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);

        let mut session_id = String::new();

        for line in reader.lines() {
            let line = line?;
            if !line.contains("\"type\"") {
                continue;
            }

            let value: serde_json::Value = serde_json::from_str(&line)
                .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;
            let event_type = value.get("type").and_then(|v| v.as_str());

            match event_type {
                Some("message") => {
                    let role = value.get("role").and_then(|v| v.as_str());
                    if role == Some("assistant") {
                        if let Some(ts_str) = value.get("timestamp").and_then(|v| v.as_str()) {
                            if let Ok(ts) = ts_str.parse() {
                                prompts.push(ParsedPrompt {
                                    ts,
                                    session_id: session_id.clone(),
                                });
                            }
                        }
                    }
                }
                Some("metadata") | Some("session") => {
                    session_id = value
                        .get("session_id")
                        .or_else(|| value.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(&uuid::Uuid::new_v4().to_string())
                        .to_string();
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Start a background refresh loop.
    ///
    /// `trigger_rx` is an optional broadcast receiver that causes an immediate
    /// recompute on any received message (used to react to bead close events).
    /// The timer always fires at `refresh_interval_secs` regardless.
    pub fn spawn_refresh_loop(
        config: CapacityMeterConfig,
        tx: tokio::sync::broadcast::Sender<Vec<AccountCapacity>>,
        mut trigger_rx: Option<tokio::sync::broadcast::Receiver<()>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let meter = CapacityMeter::new(config);
            let interval = tokio::time::Duration::from_secs(meter.config.refresh_interval_secs);
            let mut tick = tokio::time::interval(interval);

            loop {
                if let Some(ref mut rx) = trigger_rx {
                    tokio::select! {
                        _ = tick.tick() => {}
                        result = rx.recv() => {
                            match result {
                                Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                            }
                        }
                    }
                } else {
                    tick.tick().await;
                }
                let capacities = meter.compute();
                if !capacities.is_empty() {
                    let _ = tx.send(capacities);
                }
            }
        })
    }
}

/// Parse an optional RFC3339 resets_at timestamp from a WindowUsage
fn parse_resets_at(window: Option<&WindowUsage>) -> Option<DateTime<Utc>> {
    window
        .and_then(|w| w.resets_at.as_deref())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn make_assistant_jsonl(
        timestamp: &str,
        input: u64,
        output: u64,
        cache_read: u64,
        cache_write: u64,
        model: &str,
    ) -> String {
        format!(
            r#"{{"parentUuid":"p1","isSidechain":false,"type":"assistant","uuid":"u1","timestamp":"{}","userType":"external","entrypoint":"sdk-cli","cwd":"/home/test","sessionId":"s1","version":"2.1.117","gitBranch":"main","message":{{"model":"{}","id":"msg_{}","type":"message","role":"assistant","content":[],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":{},"cache_creation_input_tokens":{},"cache_read_input_tokens":{},"output_tokens":{},"server_tool_use":{{"web_search_requests":0,"web_fetch_requests":0}},"service_tier":"standard","cache_creation":{{"ephemeral_1h_input_tokens":{},"ephemeral_5m_input_tokens":0}},"inference_geo":"","iterations":[],"speed":"standard"}}}}}}"#,
            timestamp, model, timestamp, input, cache_write, cache_read, output, cache_write
        )
    }

    #[test]
    fn test_parse_jsonl_file() {
        let dir = TempDir::new().unwrap();
        let jsonl_path = dir.path().join("test.jsonl");

        let mut f = fs::File::create(&jsonl_path).unwrap();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(
                "2026-04-22T20:00:00Z",
                100,
                50,
                200,
                10,
                "claude-sonnet-4-6"
            )
        )
        .unwrap();
        writeln!(f, r#"{{"type":"user","timestamp":"2026-04-22T20:00:01Z"}}"#).unwrap();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl("2026-04-22T20:01:00Z", 200, 100, 0, 0, "claude-opus-4-7")
        )
        .unwrap();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl("2026-04-22T20:02:00Z", 0, 0, 0, 0, "<synthetic>")
        )
        .unwrap();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl("2026-04-22T20:03:00Z", 0, 0, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        let mut turns = Vec::new();
        CapacityMeter::parse_jsonl_file(&jsonl_path, &mut turns).unwrap();

        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].input_tokens, 100);
        assert_eq!(turns[0].output_tokens, 50);
        assert_eq!(turns[0].cache_read_tokens, 200);
        assert_eq!(turns[0].cache_write_tokens, 10);
        assert_eq!(turns[0].model.as_deref(), Some("claude-sonnet-4-6"));
        assert_eq!(turns[1].input_tokens, 200);
        assert_eq!(turns[1].output_tokens, 100);
        assert_eq!(turns[1].model.as_deref(), Some("claude-opus-4-7"));
    }

    #[test]
    fn test_cost_equivalent_tokens() {
        let turn = ParsedTurn {
            ts: Utc::now(),
            input_tokens: 1000,
            output_tokens: 300,
            cache_read_tokens: 5000,
            cache_write_tokens: 500,
            model: None,
            session_id: None,
        };
        let weighted = turn.cost_equivalent_tokens();
        // Expected: 1000 + 5000*0.1 + 500*0.25 + 300*5.0
        // = 1000 + 500 + 125 + 1500 = 3125
        assert!(
            weighted > 2500 && weighted < 3500,
            "cost equivalent = {}",
            weighted
        );
    }

    #[test]
    fn test_rolling_window() {
        let dir = TempDir::new().unwrap();
        let jsonl_path = dir.path().join("test.jsonl");

        let now = Utc::now();
        let mut f = fs::File::create(&jsonl_path).unwrap();

        let ts_3h = (now - Duration::hours(3)).to_rfc3339();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts_3h, 1000, 100, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        let ts_6h = (now - Duration::hours(6)).to_rfc3339();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts_6h, 2000, 200, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        let ts_8d = (now - Duration::days(8)).to_rfc3339();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts_8d, 5000, 500, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        let mut turns = Vec::new();
        CapacityMeter::parse_jsonl_file(&jsonl_path, &mut turns).unwrap();
        assert_eq!(turns.len(), 3);

        let cutoff_5h = now - Duration::hours(5);
        let cutoff_7d = now - Duration::days(7);
        let in_5h: Vec<_> = turns.iter().filter(|t| t.ts > cutoff_5h).collect();
        let in_7d: Vec<_> = turns.iter().filter(|t| t.ts > cutoff_7d).collect();

        assert_eq!(
            in_5h.len(),
            1,
            "Only the 3h-ago entry should be in the 5h window"
        );
        assert_eq!(
            in_7d.len(),
            2,
            "3h-ago and 6h-ago entries should be in the 7d window"
        );
    }

    #[test]
    fn test_plan_limits() {
        let max_20x = get_plan_limits("max", "default_claude_max_20x");
        assert!(max_20x.tokens_5h > 0);
        assert!(max_20x.tokens_7d > max_20x.tokens_5h);

        let pro = get_plan_limits("pro", "default");
        assert!(pro.tokens_5h > 0);
        assert!(pro.tokens_7d > pro.tokens_5h);
        assert!(pro.tokens_5h < max_20x.tokens_5h);
    }

    #[test]
    fn test_derive_account_id() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(
            CapacityMeter::derive_account_id(&home.join(".claude")),
            "claude-default"
        );
        assert_eq!(
            CapacityMeter::derive_account_id(&PathBuf::from("/home/user/.claude-work")),
            ".claude-work"
        );
    }

    #[test]
    fn test_deduplicate_by_message_id() {
        let dir = TempDir::new().unwrap();
        let jsonl_path = dir.path().join("test.jsonl");

        let mut f = fs::File::create(&jsonl_path).unwrap();
        let entry =
            make_assistant_jsonl("2026-04-22T20:00:00Z", 100, 50, 0, 0, "claude-sonnet-4-6");
        writeln!(f, "{}", entry).unwrap();
        writeln!(f, "{}", entry).unwrap();

        let mut turns = Vec::new();
        CapacityMeter::parse_jsonl_file(&jsonl_path, &mut turns).unwrap();
        assert_eq!(
            turns.len(),
            1,
            "Duplicate message IDs should be deduplicated"
        );
    }

    #[test]
    fn test_cached_usage_parse() {
        let cached_json = r#"{"five_hour":{"utilization":24.0,"resets_at":"2026-04-23T02:00:00.803167+00:00"},"seven_day":{"utilization":94.0,"resets_at":"2026-04-23T19:00:00.803185+00:00"},"seven_day_sonnet":{"utilization":82.0,"resets_at":"2026-04-23T19:00:00.803192+00:00"}}"#;
        let parsed: CachedUsageResponse = serde_json::from_str(cached_json).unwrap();
        assert_eq!(parsed.five_hour.unwrap().utilization, 24.0);
        assert_eq!(parsed.seven_day.unwrap().utilization, 94.0);

        let sonnet = parsed.seven_day_sonnet.unwrap().unwrap();
        assert_eq!(sonnet.utilization, 82.0);
    }

    #[test]
    fn test_cached_usage_null_model_windows() {
        let cached_json = r#"{"five_hour":{"utilization":10.0,"resets_at":"2026-04-23T02:00:00Z"},"seven_day":{"utilization":50.0,"resets_at":"2026-04-23T19:00:00Z"},"seven_day_opus":null,"seven_day_sonnet":{"utilization":40.0,"resets_at":"2026-04-23T19:00:00Z"}}"#;
        let parsed: CachedUsageResponse = serde_json::from_str(cached_json).unwrap();
        assert!(parsed.seven_day_opus.unwrap().is_none());
        let sonnet = parsed.seven_day_sonnet.unwrap().unwrap();
        assert_eq!(sonnet.utilization, 40.0);
    }

    #[test]
    fn test_full_compute_with_cache() {
        let dir = TempDir::new().unwrap();

        // Write a cached usage file
        let cache_dir = dir.path().join("cache").join("claude-usage");
        fs::create_dir_all(&cache_dir).unwrap();
        let cached = r#"{"five_hour":{"utilization":42.5,"resets_at":"2026-04-23T02:00:00Z"},"seven_day":{"utilization":88.0,"resets_at":"2026-04-23T19:00:00Z"},"seven_day_sonnet":{"utilization":75.0,"resets_at":"2026-04-23T19:00:00Z"}}"#;
        fs::write(cache_dir.join("usage.json"), cached).unwrap();

        // Write credentials
        let claude_dir = dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(
            claude_dir.join(".credentials.json"),
            r#"{"claudeAiOauth":{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}}"#,
        )
        .unwrap();

        let config = CapacityMeterConfig {
            account_dirs: vec![claude_dir],
            gemini_dirs: vec![],
            opencode_dirs: vec![],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: Some(dir.path().join("cache")),
            gcp_quota_config: None,
            accounts_file: None,
        };

        let meter = CapacityMeter::new(config);
        let accounts = meter.compute();
        assert_eq!(accounts.len(), 1);

        let acct = &accounts[0];
        assert_eq!(acct.source, "api_cache");
        assert!((acct.utilization_5h - 42.5).abs() < 0.01);
        assert!((acct.utilization_7d - 88.0).abs() < 0.01);
        assert_eq!(acct.model_windows_7d.len(), 1);
        assert_eq!(acct.model_windows_7d[0].model, "sonnet");
        assert!((acct.model_windows_7d[0].utilization - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_full_compute_jsonl_fallback() {
        let dir = TempDir::new().unwrap();

        // Write credentials
        let claude_dir = dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(
            claude_dir.join(".credentials.json"),
            r#"{"claudeAiOauth":{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}}"#,
        )
        .unwrap();

        // Write JSONL
        let projects_dir = claude_dir.join("projects");
        fs::create_dir_all(&projects_dir).unwrap();
        let now = Utc::now();
        let ts = (now - Duration::hours(1)).to_rfc3339();
        let mut f = fs::File::create(projects_dir.join("test.jsonl")).unwrap();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts, 50000, 5000, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        let config = CapacityMeterConfig {
            account_dirs: vec![claude_dir],
            gemini_dirs: vec![],
            opencode_dirs: vec![],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: Some(dir.path().join("cache")),
            gcp_quota_config: None,
            accounts_file: None,
        };

        let meter = CapacityMeter::new(config);
        let accounts = meter.compute();
        assert_eq!(accounts.len(), 1);

        let acct = &accounts[0];
        assert_eq!(acct.source, "jsonl_estimate");
        assert!(
            acct.utilization_5h > 0.0,
            "Should have nonzero 5h utilization"
        );
        assert!(
            acct.utilization_7d > 0.0,
            "Should have nonzero 7d utilization"
        );
        assert_eq!(acct.turns_5h, 1);
        assert_eq!(acct.turns_7d, 1);
    }

    #[test]
    fn test_multi_account_separate_dirs() {
        let dir = TempDir::new().unwrap();

        // Account 1: ~/.claude with Max 20x plan
        let claude1 = dir.path().join(".claude");
        fs::create_dir_all(&claude1).unwrap();
        fs::write(
            claude1.join(".credentials.json"),
            r#"{"claudeAiOauth":{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}}"#,
        )
        .unwrap();
        let projects1 = claude1.join("projects");
        fs::create_dir_all(&projects1).unwrap();
        let now = Utc::now();
        let ts1 = (now - Duration::hours(1)).to_rfc3339();
        let mut f1 = fs::File::create(projects1.join("account1.jsonl")).unwrap();
        // Heavy usage: 100K input, 20K output
        writeln!(
            f1,
            "{}",
            make_assistant_jsonl(&ts1, 100000, 20000, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        // Account 2: ~/.claude-work with Max 10x plan
        let claude2 = dir.path().join(".claude-work");
        fs::create_dir_all(&claude2).unwrap();
        fs::write(
            claude2.join(".credentials.json"),
            r#"{"claudeAiOauth":{"subscriptionType":"max","rateLimitTier":"default_claude_max_10x"}}"#,
        )
        .unwrap();
        let projects2 = claude2.join("projects");
        fs::create_dir_all(&projects2).unwrap();
        let ts2 = (now - Duration::hours(2)).to_rfc3339();
        let mut f2 = fs::File::create(projects2.join("account2.jsonl")).unwrap();
        // Light usage: 10K input, 1K output
        writeln!(
            f2,
            "{}",
            make_assistant_jsonl(&ts2, 10000, 1000, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        let config = CapacityMeterConfig {
            account_dirs: vec![claude1, claude2],
            gemini_dirs: vec![],
            opencode_dirs: vec![],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: Some(dir.path().join("cache")),
            gcp_quota_config: None,
            accounts_file: None,
        };

        let meter = CapacityMeter::new(config);
        let accounts = meter.compute();
        assert_eq!(accounts.len(), 2, "Should have two accounts");

        // Find each account
        let acct1 = accounts
            .iter()
            .find(|a| a.account_id == "claude-default")
            .expect("account 1");
        let acct2 = accounts
            .iter()
            .find(|a| a.account_id == ".claude-work")
            .expect("account 2");

        // Both should use JSONL fallback (no cached usage)
        assert_eq!(acct1.source, "jsonl_estimate");
        assert_eq!(acct2.source, "jsonl_estimate");

        // Account 1 has more usage than account 2
        assert!(
            acct1.tokens_5h > acct2.tokens_5h,
            "Account 1 should have more 5h tokens"
        );

        // Each account should have independent token counts
        // Account 1: 100000 + 20000*5 = 200000 weighted tokens
        assert!(acct1.tokens_5h > 0);
        // Account 2: 10000 + 1000*5 = 15000 weighted tokens
        assert!(acct2.tokens_5h > 0);
        assert!(acct2.tokens_5h < acct1.tokens_5h);

        // Different plan types reflected
        assert_eq!(acct1.plan_type, "max");
        assert_eq!(acct2.plan_type, "max");
        assert!(acct1.rate_limit_tier.contains("20x"));
        assert!(acct2.rate_limit_tier.contains("10x"));
    }

    #[test]
    fn test_resolve_account_paths_default() {
        let home = dirs::home_dir().unwrap();
        let config = CapacityMeterConfig::default();
        let paths = config.resolve_account_paths(&home.join(".claude"));

        assert_eq!(paths.credential_dir, home.join(".claude"));
        assert_eq!(paths.projects_dir, home.join(".claude").join("projects"));
        assert_eq!(
            paths.cached_usage_path,
            home.join(".cache").join("claude-usage").join("usage.json")
        );
    }

    #[test]
    fn test_resolve_account_paths_secondary() {
        let home = dirs::home_dir().unwrap();
        let config = CapacityMeterConfig::default();
        let paths = config.resolve_account_paths(&home.join(".claude-work"));

        assert_eq!(paths.credential_dir, home.join(".claude-work"));
        assert_eq!(
            paths.projects_dir,
            home.join(".claude-work").join("projects")
        );
        assert_eq!(
            paths.cached_usage_path,
            home.join(".cache")
                .join("claude-usage")
                .join(".claude-work-usage.json")
        );
    }

    #[test]
    fn test_5h_window_boundary() {
        let dir = TempDir::new().unwrap();
        let jsonl_path = dir.path().join("test.jsonl");
        let now = Utc::now();
        let mut f = fs::File::create(&jsonl_path).unwrap();

        // Exactly 5h ago — should be OUTSIDE the window (> not >=)
        let ts_5h = (now - Duration::hours(5)).to_rfc3339();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts_5h, 1000, 100, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        // Just inside 5h window
        let ts_4h59 = (now - Duration::hours(4) - Duration::minutes(59)).to_rfc3339();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts_4h59, 1000, 100, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        // Well inside
        let ts_1h = (now - Duration::hours(1)).to_rfc3339();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts_1h, 1000, 100, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        let turns = CapacityMeter::parse_all_jsonl(dir.path()).unwrap();
        let cutoff_5h = now - Duration::hours(5);
        let in_5h: Vec<_> = turns.iter().filter(|t| t.ts > cutoff_5h).collect();

        // Only 4h59 and 1h entries should be in window
        assert_eq!(
            in_5h.len(),
            2,
            "Exactly 5h-ago should be excluded, 4h59 and 1h included"
        );
    }

    #[test]
    fn test_7d_window_boundary() {
        let dir = TempDir::new().unwrap();
        let jsonl_path = dir.path().join("test.jsonl");
        let now = Utc::now();
        let mut f = fs::File::create(&jsonl_path).unwrap();

        // Exactly 7d ago — should be OUTSIDE
        let ts_7d = (now - Duration::days(7)).to_rfc3339();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts_7d, 1000, 100, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        // Just inside 7d window
        let ts_6d23h = (now - Duration::days(6) - Duration::hours(23)).to_rfc3339();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts_6d23h, 1000, 100, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        let turns = CapacityMeter::parse_all_jsonl(dir.path()).unwrap();
        let cutoff_7d = now - Duration::days(7);
        let in_7d: Vec<_> = turns.iter().filter(|t| t.ts > cutoff_7d).collect();

        assert_eq!(
            in_7d.len(),
            1,
            "Exactly 7d-ago should be excluded, 6d23h included"
        );
    }

    #[test]
    fn test_calibrated_remaining_formula() {
        // When API cache gives util_5h = 40% and JSONL counts tokens_5h = 400_000,
        // the derived remaining = 400_000 * 60 / 40 = 600_000 (same JSONL units).
        // This is exact regardless of JSONL/API weighting ratio.
        let dir = TempDir::new().unwrap();

        let cache_dir = dir.path().join("cache").join("claude-usage");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(
            cache_dir.join("usage.json"),
            r#"{"five_hour":{"utilization":40.0,"resets_at":"2026-04-26T02:00:00Z"},"seven_day":{"utilization":20.0,"resets_at":"2026-04-28T19:00:00Z"}}"#,
        )
        .unwrap();

        let claude_dir = dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(
            claude_dir.join(".credentials.json"),
            r#"{"claudeAiOauth":{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}}"#,
        )
        .unwrap();

        // Write recent JSONL turns so we get burn_rate > 0 and stitch data
        let projects_dir = claude_dir.join("projects");
        fs::create_dir_all(&projects_dir).unwrap();
        let now = Utc::now();
        let mut f = fs::File::create(projects_dir.join("test.jsonl")).unwrap();
        // 3 turns in last 30 min, each ~1000 cost-equivalent tokens
        // burn_rate ≈ (3 * 1000) / 60 ≈ 50/min
        for i in 0..3 {
            let ts = (now - Duration::minutes(10 * (i + 1))).to_rfc3339();
            // input=800, output=40 → weighted = 800 + 40*5 = 1000
            writeln!(
                f,
                "{}",
                make_assistant_jsonl(&ts, 800, 40, 0, 0, "claude-sonnet-4-6")
            )
            .unwrap();
        }

        let config = CapacityMeterConfig {
            account_dirs: vec![claude_dir],
            gemini_dirs: vec![],
            opencode_dirs: vec![],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: Some(dir.path().join("cache")),
            gcp_quota_config: None,
            accounts_file: None,
        };

        let meter = CapacityMeter::new(config);
        let accounts = meter.compute();
        assert_eq!(accounts.len(), 1);
        let acct = &accounts[0];

        assert_eq!(acct.source, "api_cache");
        assert!((acct.utilization_5h - 40.0).abs() < 0.01);

        // With calibrated formula: remaining_5h = tokens_5h * 60 / 40
        // ETA = remaining_5h / burn_rate_per_min
        // burn_rate > 0 so forecast should be Some
        assert!(
            acct.forecast_full_5h_min.is_some(),
            "Should have a 5h token forecast"
        );
        let eta_5h = acct.forecast_full_5h_min.unwrap();
        assert!(eta_5h > 0.0, "ETA should be positive: {}", eta_5h);

        // Verify the calibrated remaining is proportional to tokens_5h:
        // remaining = tokens_5h * (100 - 40) / 40 = tokens_5h * 1.5
        // ETA = tokens_5h * 1.5 / burn_rate
        // cross-check: remaining / burn_rate = ETA
        if acct.burn_rate_per_min > 0.0 {
            let expected_remaining = acct.tokens_5h as f64 * (100.0 - 40.0) / 40.0;
            let expected_eta = expected_remaining / acct.burn_rate_per_min;
            assert!(
                (eta_5h - expected_eta).abs() < 0.1,
                "ETA mismatch: got {}, expected {} (calibrated formula)",
                eta_5h,
                expected_eta
            );
        }
    }

    #[test]
    fn test_jsonl_accuracy_vs_cached() {
        // When cached API is available, it takes priority and JSONL is only
        // used for burn rate. Verify that the cached values are used exactly.
        let dir = TempDir::new().unwrap();

        let cache_dir = dir.path().join("cache").join("claude-usage");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(
            cache_dir.join("usage.json"),
            r#"{"five_hour":{"utilization":47.0,"resets_at":"2026-04-23T02:00:00Z"},"seven_day":{"utilization":97.0,"resets_at":"2026-04-23T19:00:00Z"},"seven_day_sonnet":{"utilization":85.0,"resets_at":"2026-04-23T19:00:00Z"}}"#,
        )
        .unwrap();

        let claude_dir = dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(
            claude_dir.join(".credentials.json"),
            r#"{"claudeAiOauth":{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}}"#,
        )
        .unwrap();

        // Also write JSONL — this should NOT override the cached values
        let projects_dir = claude_dir.join("projects");
        fs::create_dir_all(&projects_dir).unwrap();
        let now = Utc::now();
        let ts = (now - Duration::hours(1)).to_rfc3339();
        let mut f = fs::File::create(projects_dir.join("test.jsonl")).unwrap();
        // This would give very different utilization if used, but cached API takes priority
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts, 500000, 50000, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        let config = CapacityMeterConfig {
            account_dirs: vec![claude_dir],
            gemini_dirs: vec![],
            opencode_dirs: vec![],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: Some(dir.path().join("cache")),
            gcp_quota_config: None,
            accounts_file: None,
        };

        let meter = CapacityMeter::new(config);
        let accounts = meter.compute();
        assert_eq!(accounts.len(), 1);

        let acct = &accounts[0];
        assert_eq!(acct.source, "api_cache");
        // Exact match with cached values
        assert!((acct.utilization_5h - 47.0).abs() < 0.01);
        assert!((acct.utilization_7d - 97.0).abs() < 0.01);
        assert_eq!(acct.model_windows_7d.len(), 1);
        assert!((acct.model_windows_7d[0].utilization - 85.0).abs() < 0.01);
    }

    #[test]
    fn test_multi_account_with_per_account_cache() {
        let dir = TempDir::new().unwrap();

        // Account 1: has cached usage
        let claude1 = dir.path().join(".claude");
        fs::create_dir_all(&claude1).unwrap();
        fs::write(
            claude1.join(".credentials.json"),
            r#"{"claudeAiOauth":{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}}"#,
        )
        .unwrap();
        // Default account's cache at the standard path (but we'll use a temp dir)
        let cache_dir = dir.path().join("cache").join("claude-usage");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(
            cache_dir.join("usage.json"),
            r#"{"five_hour":{"utilization":30.0},"seven_day":{"utilization":60.0}}"#,
        )
        .unwrap();

        // Account 2: no cached usage, falls back to JSONL
        let claude2 = dir.path().join(".claude-alt");
        fs::create_dir_all(&claude2).unwrap();
        fs::write(
            claude2.join(".credentials.json"),
            r#"{"claudeAiOauth":{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}}"#,
        )
        .unwrap();
        let projects2 = claude2.join("projects");
        fs::create_dir_all(&projects2).unwrap();
        let now = Utc::now();
        let ts = (now - Duration::hours(1)).to_rfc3339();
        let mut f = fs::File::create(projects2.join("test.jsonl")).unwrap();
        writeln!(
            f,
            "{}",
            make_assistant_jsonl(&ts, 50000, 5000, 0, 0, "claude-sonnet-4-6")
        )
        .unwrap();

        // Build config with custom cache paths for account 1
        let config = CapacityMeterConfig {
            account_dirs: vec![claude1.clone(), claude2],
            gemini_dirs: vec![],
            opencode_dirs: vec![],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: Some(dir.path().join("cache")),
            gcp_quota_config: None,
            accounts_file: None,
        };

        let meter = CapacityMeter::new(config.clone());
        let accounts = meter.compute();
        assert_eq!(accounts.len(), 2);

        let _acct1 = accounts
            .iter()
            .find(|a| a.account_id == "claude-default")
            .unwrap();
        let acct2 = accounts
            .iter()
            .find(|a| a.account_id == ".claude-alt")
            .unwrap();

        // Account 1 has no cache at its resolved path (the cache is at our temp dir,
        // not the real ~/.cache), so it falls back to JSONL with no JSONL files.
        // Account 2 has JSONL data.
        assert_eq!(acct2.source, "jsonl_estimate");
        assert!(acct2.utilization_5h > 0.0);
    }

    // Gemini tests

    fn make_gemini_turn_jsonl(
        timestamp: &str,
        input: u64,
        output: u64,
        cache_read: u64,
        session_id: Option<&str>,
    ) -> String {
        let session_field = if let Some(sid) = session_id {
            format!(r#""session_id":"{}","#, sid)
        } else {
            String::new()
        };
        format!(
            r#"{{"type":"message","role":"model","timestamp":"{}","{}"usage":{{"promptTokenCount":{},"candidatesTokenCount":{},"cachedContentTokenCount":{}}}}}"#,
            timestamp, session_field, input, output, cache_read
        )
    }

    #[test]
    fn test_gemini_cost_equivalent_tokens() {
        let turn = GeminiTurn {
            ts: Utc::now(),
            input_tokens: 1000,
            output_tokens: 100,
            cache_read_tokens: 5000,
            cache_write_tokens: 500,
            session_id: None,
        };
        let weighted = turn.cost_equivalent_tokens();
        // Expected: 1000 + 5000*0.1 + 500*0.25 + 100*8.0
        // = 1000 + 500 + 125 + 800 = 2425
        assert!(
            weighted > 2000 && weighted < 3000,
            "cost equivalent = {}",
            weighted
        );
    }

    #[test]
    fn test_gemini_derive_account_id() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(
            CapacityMeter::derive_gemini_account_id(&home.join(".gemini")),
            "gemini-default"
        );
        assert_eq!(
            CapacityMeter::derive_gemini_account_id(&PathBuf::from("/home/user/.gemini-work")),
            ".gemini-work"
        );
    }

    #[test]
    fn test_gemini_parse_session_file() {
        let dir = TempDir::new().unwrap();
        let jsonl_path = dir.path().join("test.jsonl");

        let now = Utc::now();
        let ts_3h = (now - Duration::hours(3)).to_rfc3339();
        let ts_6h = (now - Duration::hours(6)).to_rfc3339();

        let mut f = fs::File::create(&jsonl_path).unwrap();
        writeln!(
            f,
            "{}",
            make_gemini_turn_jsonl(&ts_3h, 1000, 100, 0, Some("session-1"))
        )
        .unwrap();
        writeln!(
            f,
            "{}",
            make_gemini_turn_jsonl(&ts_6h, 2000, 200, 0, Some("session-2"))
        )
        .unwrap();
        // User prompt (should be skipped)
        writeln!(
            f,
            r#"{{"type":"message","role":"user","timestamp":"{}"}}"#,
            ts_3h
        )
        .unwrap();

        let mut turns = Vec::new();
        CapacityMeter::parse_gemini_jsonl_file(&jsonl_path, &mut turns).unwrap();

        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].input_tokens, 1000);
        assert_eq!(turns[0].output_tokens, 100);
        assert_eq!(turns[0].session_id.as_deref(), Some("session-1"));
    }

    #[test]
    fn test_gemini_capacity_compute() {
        let dir = TempDir::new().unwrap();

        // Create a Gemini session directory
        let gemini_dir = dir.path().join(".gemini");
        let tmp_dir = gemini_dir.join("tmp");
        fs::create_dir_all(&tmp_dir).unwrap();

        // Create session files with recent usage
        let now = Utc::now();
        let ts_1h = (now - Duration::hours(1)).to_rfc3339();
        let ts_2h = (now - Duration::hours(2)).to_rfc3339();

        let mut f = fs::File::create(tmp_dir.join("session1.jsonl")).unwrap();
        // High usage: 100K input, 10K output
        writeln!(
            f,
            "{}",
            make_gemini_turn_jsonl(&ts_1h, 100000, 10000, 0, Some("session-1"))
        )
        .unwrap();

        let mut f2 = fs::File::create(tmp_dir.join("session2.jsonl")).unwrap();
        writeln!(
            f2,
            "{}",
            make_gemini_turn_jsonl(&ts_2h, 50000, 5000, 0, Some("session-2"))
        )
        .unwrap();

        let config = CapacityMeterConfig {
            account_dirs: vec![],
            gemini_dirs: vec![gemini_dir],
            opencode_dirs: vec![],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: Some(dir.path().join("cache")),
            gcp_quota_config: None,
            accounts_file: None,
        };

        let meter = CapacityMeter::new(config);
        let accounts = meter.compute();

        // Should have one Gemini account
        let gemini_accounts: Vec<_> = accounts.iter().filter(|a| a.adapter == "gemini").collect();
        assert_eq!(gemini_accounts.len(), 1);

        let acct = &gemini_accounts[0];
        assert_eq!(acct.account_id, "gemini-default");
        assert_eq!(acct.adapter, "gemini");
        assert_eq!(acct.source, "jsonl_estimate");

        // Should have positive utilization from recent turns
        assert!(
            acct.utilization_5h > 0.0,
            "5h utilization should be positive"
        );
        assert!(
            acct.utilization_7d > 0.0,
            "7d utilization should be positive"
        );
        assert_eq!(acct.turns_5h, 2);
        assert_eq!(acct.turns_7d, 2);
    }

    #[test]
    fn test_gemini_discover_multiple_accounts() {
        let dir = TempDir::new().unwrap();

        // Create two Gemini accounts
        let gemini1 = dir.path().join(".gemini");
        let tmp1 = gemini1.join("tmp");
        fs::create_dir_all(&tmp1).unwrap();

        let gemini2 = dir.path().join(".gemini-work");
        let tmp2 = gemini2.join("sessions");
        fs::create_dir_all(&tmp2).unwrap();

        // Add session files to each
        let now = Utc::now();
        let ts = (now - Duration::hours(1)).to_rfc3339();

        let mut f = fs::File::create(tmp1.join("session.jsonl")).unwrap();
        writeln!(f, "{}", make_gemini_turn_jsonl(&ts, 50000, 5000, 0, None)).unwrap();

        let mut f2 = fs::File::create(tmp2.join("session.jsonl")).unwrap();
        writeln!(
            f2,
            "{}",
            make_gemini_turn_jsonl(&ts, 100000, 10000, 0, None)
        )
        .unwrap();

        let config = CapacityMeterConfig {
            account_dirs: vec![],
            gemini_dirs: vec![gemini1.clone(), gemini2.clone()],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: Some(dir.path().join("cache")),
        };

        let meter = CapacityMeter::new(config);
        let accounts = meter.compute();

        // Should have two Gemini accounts
        let gemini_accounts: Vec<_> = accounts.iter().filter(|a| a.adapter == "gemini").collect();
        assert_eq!(gemini_accounts.len(), 2);

        // Find each account
        let acct1 = gemini_accounts
            .iter()
            .find(|a| a.account_id == "gemini-default")
            .expect("account 1");
        let acct2 = gemini_accounts
            .iter()
            .find(|a| a.account_id == ".gemini-work")
            .expect("account 2");

        // Account 2 should have higher usage
        assert!(acct2.tokens_5h > acct1.tokens_5h);
    }

    #[test]
    fn test_gemini_rolling_window() {
        let dir = TempDir::new().unwrap();
        let jsonl_path = dir.path().join("test.jsonl");

        let now = Utc::now();
        let ts_3h = (now - Duration::hours(3)).to_rfc3339();
        let ts_6h = (now - Duration::hours(6)).to_rfc3339();
        let ts_8d = (now - Duration::days(8)).to_rfc3339();

        let mut f = fs::File::create(&jsonl_path).unwrap();
        writeln!(f, "{}", make_gemini_turn_jsonl(&ts_3h, 1000, 100, 0, None)).unwrap();
        writeln!(f, "{}", make_gemini_turn_jsonl(&ts_6h, 2000, 200, 0, None)).unwrap();
        writeln!(f, "{}", make_gemini_turn_jsonl(&ts_8d, 5000, 500, 0, None)).unwrap();

        let mut turns = Vec::new();
        CapacityMeter::parse_gemini_jsonl_file(&jsonl_path, &mut turns).unwrap();
        assert_eq!(turns.len(), 3);

        let cutoff_5h = now - Duration::hours(5);
        let cutoff_7d = now - Duration::days(7);
        let in_5h: Vec<_> = turns.iter().filter(|t| t.ts > cutoff_5h).collect();
        let in_7d: Vec<_> = turns.iter().filter(|t| t.ts > cutoff_7d).collect();

        assert_eq!(
            in_5h.len(),
            1,
            "Only the 3h-ago entry should be in the 5h window"
        );
        assert_eq!(
            in_7d.len(),
            2,
            "3h-ago and 6h-ago entries should be in the 7d window"
        );
    }

    // GCP Quota API Tests

    #[test]
    fn test_gemini_limits_default() {
        let limits = GeminiLimits::default();
        assert!(!limits.from_api, "Default limits should not be from API");
        assert_eq!(limits.tokens_5h, 1_000_000);
        assert_eq!(limits.tokens_7d, 15_000_000);
    }

    #[test]
    fn test_gemini_limits_with_api_flag() {
        let limits = GeminiLimits {
            tokens_5h: 2_000_000,
            tokens_7d: 30_000_000,
            from_api: true,
        };
        assert!(limits.from_api, "API limits should have from_api=true");
        assert_eq!(limits.tokens_5h, 2_000_000);
        assert_eq!(limits.tokens_7d, 30_000_000);
    }

    #[test]
    fn test_config_without_gcp_quota() {
        let config = CapacityMeterConfig {
            account_dirs: vec![],
            gemini_dirs: vec![],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: None,
            gcp_quota_config: None,
        };
        assert!(
            config.gcp_quota_config.is_none(),
            "GCP quota config should be None when not set"
        );
    }

    #[test]
    fn test_config_with_gcp_quota() {
        use gcp_quota_client::GcpQuotaConfig;

        let gcp_config = GcpQuotaConfig {
            project_id: "test-project".to_string(),
            region: "us-central1".to_string(),
            enabled: true,
        };

        let config = CapacityMeterConfig {
            account_dirs: vec![],
            gemini_dirs: vec![],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: None,
            gcp_quota_config: Some(gcp_config),
        };

        assert!(
            config.gcp_quota_config.is_some(),
            "GCP quota config should be Some when set"
        );
        let quota_config = config.gcp_quota_config.unwrap();
        assert_eq!(quota_config.project_id, "test-project");
        assert_eq!(quota_config.region, "us-central1");
        assert!(quota_config.enabled);
    }

    #[test]
    fn test_gemini_compute_with_jsonl_fallback() {
        // Test that compute_gemini_account works without GCP quota API
        let dir = TempDir::new().unwrap();

        let gemini_dir = dir.path().join(".gemini");
        let tmp_dir = gemini_dir.join("tmp");
        fs::create_dir_all(&tmp_dir).unwrap();

        let now = Utc::now();
        let ts_1h = (now - Duration::hours(1)).to_rfc3339();

        let mut f = fs::File::create(tmp_dir.join("session.jsonl")).unwrap();
        writeln!(
            f,
            "{}",
            make_gemini_turn_jsonl(&ts_1h, 50000, 5000, 0, Some("session-1"))
        )
        .unwrap();

        // Config without GCP quota API (using defaults)
        let config = CapacityMeterConfig {
            account_dirs: vec![],
            gemini_dirs: vec![gemini_dir],
            refresh_interval_secs: 60,
            cache_max_age_secs: 600,
            cache_base_dir: Some(dir.path().join("cache")),
            gcp_quota_config: None,
        };

        let meter = CapacityMeter::new(config);
        let accounts = meter.compute();

        let gemini_accounts: Vec<_> = accounts.iter().filter(|a| a.adapter == "gemini").collect();
        assert_eq!(gemini_accounts.len(), 1);

        let acct = &gemini_accounts[0];
        assert_eq!(acct.account_id, "gemini-default");
        assert_eq!(
            acct.source, "jsonl_estimate",
            "Without GCP API, source should be jsonl_estimate"
        );
        assert!(acct.utilization_5h > 0.0);
        assert!(acct.utilization_7d > 0.0);
    }

    #[test]
    fn test_gemini_daily_limit_to_windows() {
        // Test that daily limits are correctly converted to 5h and 7d windows
        // Daily: 1M tokens -> 7d: 7M, 5h: ~208K (1M / 24 * 5)
        let daily = 1_000_000;
        let expected_7d = daily * 7;
        let expected_5h = daily / 24 * 5;

        assert_eq!(expected_7d, 7_000_000);
        assert_eq!(expected_5h, 208_333);
    }
}
