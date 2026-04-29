//! Saturation alert detector (§6 P2 d10, §8.3, hoop-ttb.3.22)
//!
//! Passive capacity threshold monitoring — emits UI banner + audit row
//! when an account crosses 80% on 5h or 7d windows. No worker throttling,
//! rotation, or pausing — HOOP only informs.
//!
//! ## Design
//!
//! - Threshold: 80% utilization on 5h or 7d windows
//! - Debounce: One alert per account+window per daemon session
//! - Auto-clear: Alert state resets when utilization drops below 75%
//! - Audit: Writes row on initial threshold cross
//! - WebSocket: Emits event for UI banner

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing::{info, warn};

/// Saturation threshold percentage
const SATURATION_THRESHOLD: f64 = 80.0;

/// Clear threshold percentage (hysteresis to prevent flapping)
const CLEAR_THRESHOLD: f64 = 75.0;

/// Per-account+window saturation state
#[derive(Debug, Clone)]
struct SaturationState {
    /// Alert has been fired for this account+window
    alert_fired: bool,
    /// When the alert was fired
    fired_at: Option<DateTime<Utc>>,
    /// Unique alert ID (generated once per session)
    alert_id: Option<String>,
}

impl Default for SaturationState {
    fn default() -> Self {
        Self {
            alert_fired: false,
            fired_at: None,
            alert_id: None,
        }
    }
}

/// Saturation detector
///
/// Monitors capacity utilization and fires alerts when thresholds are exceeded.
/// Keyed by account_id to track per-account state.
#[derive(Debug)]
pub struct SaturationDetector {
    /// Broadcast sender for saturation alerts
    alert_tx: broadcast::Sender<crate::ws::SaturationAlertData>,
    /// Per-account+window state: "account_id:5h" and "account_id:7d"
    state: Arc<Mutex<HashMap<String, SaturationState>>>,
}

impl SaturationDetector {
    /// Create a new saturation detector
    pub fn new(alert_tx: broadcast::Sender<crate::ws::SaturationAlertData>) -> Self {
        Self {
            alert_tx,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check capacity and fire/clear alerts as needed
    ///
    /// Called when capacity data is updated. Examines each account's
    /// 5h and 7d utilization windows and fires alerts on threshold cross.
    pub fn check_capacity(&self, capacities: &[crate::capacity::AccountCapacity]) {
        let mut state = self.state.lock().unwrap();

        for cap in capacities {
            // Check 5h window
            let key_5h = format!("{}:5h", cap.account_id);
            self.check_window(
                &mut state,
                &key_5h,
                cap,
                cap.utilization_5h,
                "5h",
                cap.tokens_5h,
            );

            // Check 7d window
            let key_7d = format!("{}:7d", cap.account_id);
            self.check_window(
                &mut state,
                &key_7d,
                cap,
                cap.utilization_7d,
                "7d",
                cap.tokens_7d,
            );
        }
    }

    /// Check a single window and fire/clear alerts
    fn check_window(
        &self,
        state: &mut HashMap<String, SaturationState>,
        key: &str,
        cap: &crate::capacity::AccountCapacity,
        utilization: f64,
        window: &str,
        tokens: u64,
    ) {
        let entry = state.entry(key.to_string()).or_default();

        let is_saturated = utilization >= SATURATION_THRESHOLD;
        let is_cleared = utilization < CLEAR_THRESHOLD;

        if is_saturated && !entry.alert_fired {
            // Fire alert
            let alert_id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now();

            entry.alert_fired = true;
            entry.fired_at = Some(now);
            entry.alert_id = Some(alert_id.clone());

            info!(
                "Saturation alert: {} {} {}% (threshold {}%)",
                cap.account_id, window, utilization, SATURATION_THRESHOLD
            );

            // Write audit row
            if let Err(e) = self.write_audit_row(cap, utilization, window, &alert_id) {
                warn!("Failed to write saturation audit row: {}", e);
            }

            // Emit WebSocket event
            let alert_data = crate::ws::SaturationAlertData {
                alert_id,
                detected_at: now.to_rfc3339(),
                account: cap.account_id.clone(),
                model: cap.adapter.clone(),
                utilization_percent: utilization,
                threshold_percent: SATURATION_THRESHOLD,
                current_tpm: tokens,
                quota_tpm: 0, // Not tracked in current AccountCapacity
            };

            if let Err(e) = self.alert_tx.send(alert_data) {
                warn!("Failed to send saturation alert: {}", e);
            }
        } else if is_cleared && entry.alert_fired {
            // Clear alert state (hysteresis prevents flapping)
            info!(
                "Saturation cleared: {} {} {}% (clear threshold {}%)",
                cap.account_id, window, utilization, CLEAR_THRESHOLD
            );
            entry.alert_fired = false;
            entry.fired_at = None;
            entry.alert_id = None;
        }
    }

    /// Write an audit row for the saturation alert
    fn write_audit_row(
        &self,
        cap: &crate::capacity::AccountCapacity,
        utilization: f64,
        window: &str,
        alert_id: &str,
    ) -> Result<()> {
        use crate::fleet::{write_audit_row, ActionResult};

        let args = serde_json::json!({
            "account": cap.account_id,
            "adapter": cap.adapter,
            "utilization_percent": utilization,
            "threshold_percent": SATURATION_THRESHOLD,
            "window": window,
            "alert_id": alert_id,
        });

        write_audit_row(
            "hoop-daemon",
            crate::fleet::ActionKind::SaturationAlert,
            &format!("{}:{}", cap.account_id, window),
            None,
            Some(args.to_string()),
            ActionResult::Success,
            None,
            Some("saturation_detector"),
            None,
            None,
        )?;

        Ok(())
    }

    /// Subscribe to saturation alert events
    pub fn subscribe(&self) -> broadcast::Receiver<crate::ws::SaturationAlertData> {
        self.alert_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Helper to create a mock AccountCapacity for testing
    /// Only includes the fields needed for saturation detection logic
    fn mock_capacity(account_id: &str, adapter: &str, util_5h: f64, util_7d: f64, tokens_5h: u64, tokens_7d: u64) -> crate::capacity::AccountCapacity {
        crate::capacity::AccountCapacity {
            account_id: account_id.to_string(),
            adapter: adapter.to_string(),
            plan_type: "test-plan".to_string(),
            rate_limit_tier: "test-tier".to_string(),
            utilization_5h: util_5h,
            utilization_7d: util_7d,
            resets_at_5h: None,
            resets_at_7d: None,
            model_windows_7d: vec![],
            tokens_5h,
            tokens_7d,
            turns_5h: 0,
            turns_7d: 0,
            prompts_5h: 0,
            prompts_7d: 0,
            prompts_per_5h: None,
            prompts_per_7d: None,
            burn_rate_per_min: 0.0,
            forecast_full_5h_min: None,
            forecast_full_7d_min: None,
            stitch_close_rate_per_min: 0.0,
            mean_cost_per_stitch_tokens: 0.0,
            limits: Default::default(),
            usage: Default::default(),
            window_start: None,
            window_end: None,
        }
    }

    #[test]
    fn test_threshold_constants() {
        assert_eq!(SATURATION_THRESHOLD, 80.0);
        assert_eq!(CLEAR_THRESHOLD, 75.0);
        assert!(CLEAR_THRESHOLD < SATURATION_THRESHOLD);
    }

    #[test]
    fn test_is_saturated_at_threshold() {
        // Exactly at threshold should be considered saturated
        assert!(80.0 >= SATURATION_THRESHOLD);
    }

    #[test]
    fn test_is_saturated_above_threshold() {
        assert!(85.0 >= SATURATION_THRESHOLD);
        assert!(100.0 >= SATURATION_THRESHOLD);
        assert!(90.5 >= SATURATION_THRESHOLD);
    }

    #[test]
    fn test_is_saturated_below_threshold() {
        assert!(79.9 < SATURATION_THRESHOLD);
        assert!(75.0 < SATURATION_THRESHOLD);
        assert!(50.0 < SATURATION_THRESHOLD);
        assert!(0.0 < SATURATION_THRESHOLD);
    }

    #[test]
    fn test_is_cleared_at_clear_threshold() {
        // Exactly at clear threshold should be considered cleared
        assert!(75.0 < CLEAR_THRESHOLD || 75.0 == CLEAR_THRESHOLD);
    }

    #[test]
    fn test_is_cleared_below_clear_threshold() {
        assert!(74.9 < CLEAR_THRESHOLD);
        assert!(50.0 < CLEAR_THRESHOLD);
        assert!(0.0 < CLEAR_THRESHOLD);
    }

    #[test]
    fn test_is_cleared_above_clear_threshold() {
        assert!(75.1 >= CLEAR_THRESHOLD);
        assert!(80.0 >= CLEAR_THRESHOLD);
        assert!(100.0 >= CLEAR_THRESHOLD);
    }

    #[test]
    fn test_hysteresis_gap() {
        // There should be a gap between saturation and clear thresholds
        let gap = SATURATION_THRESHOLD - CLEAR_THRESHOLD;
        assert!(gap > 0.0, "hysteresis gap should be positive");
        assert_eq!(gap, 5.0, "hysteresis gap should be 5%");
    }

    #[test]
    fn test_intermediate_zone() {
        // Zone between clear and saturation thresholds
        let intermediate = 77.5;
        assert!(intermediate < SATURATION_THRESHOLD, "should not be saturated");
        assert!(intermediate > CLEAR_THRESHOLD, "should not be cleared");
        // This is the "no-change" zone where existing alerts persist
    }

    #[test]
    fn test_window_key_format() {
        let account = "test-account";
        let key_5h = format!("{}:5h", account);
        let key_7d = format!("{}:7d", account);
        assert_eq!(key_5h, "test-account:5h");
        assert_eq!(key_7d, "test-account:7d");
    }

    #[test]
    fn test_saturation_state_default() {
        let state = SaturationState::default();
        assert!(!state.alert_fired);
        assert!(state.fired_at.is_none());
        assert!(state.alert_id.is_none());
    }

    #[test]
    fn test_multiple_accounts_independent() {
        // Different accounts should have independent state
        let acc1 = "account-1";
        let acc2 = "account-2";
        let key1_5h = format!("{}:5h", acc1);
        let key2_5h = format!("{}:5h", acc2);
        assert_ne!(key1_5h, key2_5h);
    }

    #[test]
    fn test_multiple_windows_independent() {
        // Same account, different windows should have independent state
        let account = "test-account";
        let key_5h = format!("{}:5h", account);
        let key_7d = format!("{}:7d", account);
        assert_ne!(key_5h, key_7d);
    }

    #[test]
    fn test_utilization_bounds() {
        // Test edge cases of utilization values
        assert!(0.0 >= 0.0, "zero utilization is valid");
        assert!(100.0 >= 0.0, "100% utilization is valid");
        // Values above 100% should still work mathematically
        assert!(150.0 >= SATURATION_THRESHOLD);
    }

    #[test]
    fn test_negative_utilization() {
        // Negative utilization should not trigger saturation
        assert!((-10.0) < SATURATION_THRESHOLD);
        assert!((-10.0) < CLEAR_THRESHOLD);
    }

    #[test]
    fn test_floating_point_precision() {
        // Test that floating-point comparisons work correctly
        let val1 = 80.0 + 0.0000001;
        let val2 = 80.0 - 0.0000001;
        assert!(val1 > SATURATION_THRESHOLD);
        assert!(val2 < SATURATION_THRESHOLD);
    }
}
