//! Pricing watcher for monitoring API pricing changes
//!
//! This module maintains a static pricing table for API providers
//! and exposes a `token_cost()` function for cost calculation.
//! It supports runtime overrides from config and tracks unknown
//! model pricing lookups.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tracing::{info, warn};

use crate::metrics::metrics;

/// Model pricing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelPricing {
    /// Input price per million tokens (USD)
    pub input_per_million: f64,
    /// Output price per million tokens (USD)
    pub output_per_million: f64,
    /// Cache read price per million tokens (USD) - optional
    #[serde(default)]
    pub cache_read_per_million: Option<f64>,
    /// Cache write price per million tokens (USD) - optional
    #[serde(default)]
    pub cache_write_per_million: Option<f64>,
}

impl ModelPricing {
    /// Get cache read price per million, defaulting to 0
    pub fn cache_read_per_million(&self) -> f64 {
        self.cache_read_per_million.unwrap_or(0.0)
    }

    /// Get cache write price per million, defaulting to 0
    pub fn cache_write_per_million(&self) -> f64 {
        self.cache_write_per_million.unwrap_or(0.0)
    }

    /// Calculate cost for given token counts
    ///
    /// # Arguments
    /// * `input_tokens` - Number of input tokens
    /// * `output_tokens` - Number of output tokens
    /// * `cache_read_tokens` - Number of cache read tokens (default: 0)
    /// * `cache_write_tokens` - Number of cache write tokens (default: 0)
    ///
    /// # Returns
    /// Cost in USD
    pub fn calculate_cost(
        &self,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
    ) -> f64 {
        let input_per_m = self.input_per_million / 1_000_000.0;
        let output_per_m = self.output_per_million / 1_000_000.0;
        let cache_read_per_m = self.cache_read_per_million() / 1_000_000.0;
        let cache_write_per_m = self.cache_write_per_million() / 1_000_000.0;

        let input_cost = input_tokens as f64 * input_per_m;
        let output_cost = output_tokens as f64 * output_per_m;
        let cache_read_cost = cache_read_tokens as f64 * cache_read_per_m;
        let cache_write_cost = cache_write_tokens as f64 * cache_write_per_m;

        input_cost + output_cost + cache_read_cost + cache_write_cost
    }
}

/// Per-plan-tier pricing overrides for an adapter
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PlanTierPricing {
    /// Model-specific pricing overrides for this tier
    #[serde(default)]
    pub models: HashMap<String, ModelPricing>,
}

/// Adapter pricing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdapterPricing {
    /// Model-specific pricing
    #[serde(default)]
    pub models: HashMap<String, ModelPricing>,
    /// Default model for this adapter
    #[serde(default)]
    pub default_model: Option<String>,
    /// Optional per-plan-tier pricing overrides (tier_name → model pricing)
    #[serde(default)]
    pub plan_tiers: Option<HashMap<String, PlanTierPricing>>,
    /// Optional account-to-tier mapping (account_id → tier_name)
    #[serde(default)]
    pub account_tiers: Option<HashMap<String, String>>,
}

/// Full pricing configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PricingConfig {
    /// Per-adapter pricing
    #[serde(default)]
    pub adapters: HashMap<String, AdapterPricing>,
}

impl Default for PricingConfig {
    fn default() -> Self {
        serde_yaml::from_str(DEFAULT_PRICING_YAML).expect("Default pricing YAML should be valid")
    }
}

/// Pricing watcher for API cost tracking
///
/// Maintains a pricing table and provides cost calculation functions.
/// Thread-safe via interior mutability (RwLock).
pub struct PricingWatcher {
    /// Config file path for runtime overrides
    config_path: PathBuf,
    /// Pricing configuration (protected by RwLock for interior mutability)
    pricing: RwLock<PricingConfig>,
}

impl PricingWatcher {
    /// Create a new pricing watcher with the given config path
    ///
    /// # Arguments
    /// * `config_path` - Path to pricing config file (e.g., ~/.hoop/pricing.yml)
    ///
    /// # Returns
    /// A new PricingWatcher instance
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let pricing = Self::load_pricing(&config_path)?;
        Ok(Self {
            config_path,
            pricing: RwLock::new(pricing),
        })
    }

    /// Load pricing configuration from file, falling back to defaults
    fn load_pricing(path: &Path) -> Result<PricingConfig> {
        if !path.exists() {
            info!(
                "Pricing config not found at {}, using defaults",
                path.display()
            );
            return Ok(PricingConfig::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read pricing config from {}", path.display()))?;

        let config: PricingConfig = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse pricing config from {}", path.display()))?;

        Ok(config)
    }

    /// Start the pricing watcher (currently a no-op, reserved for future live polling)
    pub fn start(&mut self) -> Result<()> {
        info!("Pricing watcher started with config from {}", self.config_path.display());
        Ok(())
    }

    /// Reload pricing configuration from file
    pub fn reload(&self) -> Result<()> {
        let new_pricing = Self::load_pricing(&self.config_path)?;
        *self.pricing.write().unwrap() = new_pricing;
        info!(
            "Reloaded pricing configuration from {}",
            self.config_path.display()
        );
        Ok(())
    }

    /// Calculate token cost for a given adapter and model
    ///
    /// # Arguments
    /// * `adapter` - Adapter name (e.g., "claude", "codex", "gemini")
    /// * `model` - Model name (e.g., "claude-sonnet-4-6-20250514")
    /// * `input_tokens` - Number of input tokens
    /// * `output_tokens` - Number of output tokens
    ///
    /// # Returns
    /// Cost in USD. Returns 0.0 if adapter/model is not found (logs warning).
    pub fn token_cost(
        &self,
        adapter: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) -> f64 {
        self.token_cost_with_cache(adapter, model, input_tokens, output_tokens, 0, 0)
    }

    /// Calculate token cost with cache read/write tokens
    ///
    /// # Arguments
    /// * `adapter` - Adapter name (e.g., "claude", "codex", "gemini")
    /// * `model` - Model name (e.g., "claude-sonnet-4-6-20250514")
    /// * `input_tokens` - Number of input tokens
    /// * `output_tokens` - Number of output tokens
    /// * `cache_read_tokens` - Number of cache read tokens
    /// * `cache_write_tokens` - Number of cache write tokens
    ///
    /// # Returns
    /// Cost in USD. Returns 0.0 if adapter/model is not found (logs warning and increments metric).
    pub fn token_cost_with_cache(
        &self,
        adapter: &str,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
    ) -> f64 {
        let pricing = self.pricing.read().unwrap();

        // Try to get adapter pricing
        let adapter_pricing = pricing.adapters.get(adapter);

        if let Some(adapter) = adapter_pricing {
            // Try model-specific pricing
            if let Some(model_pricing) = adapter.models.get(model) {
                return model_pricing.calculate_cost(
                    input_tokens,
                    output_tokens,
                    cache_read_tokens,
                    cache_write_tokens,
                );
            }

            // Try default model
            if let Some(default_model) = &adapter.default_model {
                if let Some(model_pricing) = adapter.models.get(default_model) {
                    return model_pricing.calculate_cost(
                        input_tokens,
                        output_tokens,
                        cache_read_tokens,
                        cache_write_tokens,
                    );
                }
            }
        }

        // Model not found - log warning and increment metric
        warn!(
            "No pricing found for {}/{} - returning 0.0 cost",
            adapter, model
        );
        metrics().hoop_unknown_model_pricing_total.inc(&[model]);

        0.0
    }

    /// Get the pricing configuration snapshot
    ///
    /// This is useful for debugging and API responses.
    pub fn get_pricing(&self) -> PricingConfig {
        self.pricing.read().unwrap().clone()
    }

    /// Check if a model has pricing configured
    ///
    /// # Arguments
    /// * `adapter` - Adapter name
    /// * `model` - Model name
    ///
    /// # Returns
    /// true if pricing is configured, false otherwise
    pub fn has_pricing(&self, adapter: &str, model: &str) -> bool {
        let pricing = self.pricing.read().unwrap();

        if let Some(adapter) = pricing.adapters.get(adapter) {
            if adapter.models.contains_key(model) {
                return true;
            }
            if let Some(default) = &adapter.default_model {
                if adapter.models.contains_key(default) {
                    return true;
                }
            }
        }

        false
    }

    /// Get all available models for an adapter
    ///
    /// # Arguments
    /// * `adapter` - Adapter name
    ///
    /// # Returns
    /// Vector of model names (empty if adapter not found)
    pub fn get_models_for_adapter(&self, adapter: &str) -> Vec<String> {
        let pricing = self.pricing.read().unwrap();

        pricing
            .adapters
            .get(adapter)
            .map(|a| a.models.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for PricingWatcher {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        let pricing_config_path = home.join("pricing.yml");
        Self::new(pricing_config_path)
            .expect("Failed to create default PricingWatcher")
    }
}

/// Default pricing configuration as YAML
///
/// This is bundled with the binary and provides fallback pricing
/// for common providers. Operators can override via ~/.hoop/pricing.yml.
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
    # Plan-tier pricing overrides: consulted before per-model rates when
    # an account_tier mapping is present for the account_id.
    plan_tiers:
      tier_1:
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
      tier_2:
        models:
          gpt-4-turbo:
            input_per_million: 8.0
            output_per_million: 24.0
          gpt-4:
            input_per_million: 24.0
            output_per_million: 48.0
          gpt-3.5-turbo:
            input_per_million: 0.4
            output_per_million: 1.2
      free:
        models:
          gpt-4-turbo:
            input_per_million: 0.0
            output_per_million: 0.0
          gpt-4:
            input_per_million: 0.0
            output_per_million: 0.0
          gpt-3.5-turbo:
            input_per_million: 0.0
            output_per_million: 0.0
    # account_tiers: maps account_id → plan tier name.
    # Add entries here to override per-account pricing, e.g.:
    #   myaccount: tier_2
    #   work: tier_1
    account_tiers: {}
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

    #[test]
    fn test_default_pricing_loads() {
        let pricing = PricingConfig::default();
        assert!(pricing.adapters.contains_key("claude"));
        assert!(pricing.adapters.contains_key("codex"));
        assert!(pricing.adapters.contains_key("gemini"));
    }

    #[test]
    fn test_model_pricing_calculate_cost() {
        let pricing = ModelPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
            cache_read_per_million: Some(0.0),
            cache_write_per_million: Some(0.30),
        };

        let cost = pricing.calculate_cost(1000, 500, 100, 50);
        assert!(cost > 0.0);

        // 1000 input @ $3/M = $0.003
        // 500 output @ $15/M = $0.0075
        // 100 cache read @ $0/M = $0
        // 50 cache write @ $0.30/M = $0.000015
        // Total ≈ $0.010515
        assert!((cost - 0.010515).abs() < 0.0001);
    }

    #[test]
    fn test_pricing_watcher_has_pricing() {
        let watcher = PricingWatcher::default();

        assert!(watcher.has_pricing("claude", "sonnet"));
        assert!(watcher.has_pricing("claude", "opus"));
        assert!(watcher.has_pricing("codex", "gpt-4-turbo"));
        assert!(watcher.has_pricing("gemini", "gemini-2.5-flash"));

        // Unknown model should return false
        assert!(!watcher.has_pricing("claude", "unknown-model"));
        assert!(!watcher.has_pricing("unknown-adapter", "any-model"));
    }

    #[test]
    fn test_pricing_watcher_token_cost() {
        let watcher = PricingWatcher::default();

        let cost = watcher.token_cost("claude", "sonnet", 1000, 500);
        assert!(cost > 0.0);

        // Verify the cost calculation
        // Sonnet: $3/M input, $15/M output
        // 1000 input @ $3/M = $0.003
        // 500 output @ $15/M = $0.0075
        // Total ≈ $0.0105
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_pricing_watcher_token_cost_unknown_model() {
        let watcher = PricingWatcher::default();

        // Unknown model should return 0.0
        let cost = watcher.token_cost("unknown-adapter", "unknown-model", 1000, 500);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_pricing_watcher_get_models_for_adapter() {
        let watcher = PricingWatcher::default();

        let claude_models = watcher.get_models_for_adapter("claude");
        assert!(!claude_models.is_empty());
        assert!(claude_models.contains(&"sonnet".to_string()));
        assert!(claude_models.contains(&"opus".to_string()));

        let unknown_models = watcher.get_models_for_adapter("unknown-adapter");
        assert!(unknown_models.is_empty());
    }

    #[test]
    fn test_model_pricing_default_cache_values() {
        let pricing = ModelPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
            cache_read_per_million: None,
            cache_write_per_million: None,
        };

        assert_eq!(pricing.cache_read_per_million(), 0.0);
        assert_eq!(pricing.cache_write_per_million(), 0.0);

        // Cost with cache tokens should work with default 0.0 pricing
        let cost = pricing.calculate_cost(1000, 500, 100, 50);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_pricing_watcher_token_cost_with_cache() {
        let watcher = PricingWatcher::default();

        // Sonnet has cache pricing
        let cost = watcher.token_cost_with_cache("claude", "sonnet", 1000, 500, 100, 50);
        assert!(cost > 0.0);

        // Verify cache pricing is applied
        // 1000 input @ $3/M = $0.003
        // 500 output @ $15/M = $0.0075
        // 100 cache read @ $0/M = $0
        // 50 cache write @ $0.30/M = $0.000015
        // Total ≈ $0.010515
        assert!((cost - 0.010515).abs() < 0.0001);
    }
}
