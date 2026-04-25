//! Tailscale identity resolution with per-connection caching
//!
//! HTTP/WebSocket connections from Tailscale clients have their peer identity
//! resolved via `tailscale whois --json`. The result is cached per IP address with a
//! TTL to avoid repeated subprocess calls.
//!
//! Falls back to OS user when Tailscale identity is unavailable.
//!
//! Identity format:
//! - `tailscale:user@example.com` - User identity (from UserProfile.LoginName)
//! - `tailscale:machine-name` - Machine identity (from Node.ComputedName)
//! - `os:username` - OS user fallback

use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Cache TTL for whois results (5 minutes)
const WHOIS_CACHE_TTL: Duration = Duration::from_secs(300);

/// Cached identity entry with timestamp
#[derive(Debug)]
struct CacheEntry {
    identity: String,
    cached_at: Instant,
}

impl CacheEntry {
    fn new(identity: String) -> Self {
        Self {
            identity,
            cached_at: Instant::now(),
        }
    }

    fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < WHOIS_CACHE_TTL
    }
}

/// Identity cache for Tailscale whois lookups
///
/// Maps IP addresses to resolved identities with automatic expiration.
#[derive(Debug, Clone, Default)]
pub struct IdentityCache {
    inner: Arc<RwLock<HashMap<IpAddr, CacheEntry>>>,
}

use std::sync::Arc;

impl IdentityCache {
    /// Create a new identity cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve identity for a remote IP address
    ///
    /// - Returns cached identity if available and not expired
    /// - Runs `tailscale whois` for uncached or expired entries
    /// - Falls back to OS user when Tailscale identity is unavailable
    pub fn resolve(&self, remote_addr: Option<std::net::SocketAddr>) -> String {
        if let Some(addr) = remote_addr {
            let ip = addr.ip();

            // Check cache first (read lock)
            {
                let cache = self.inner.read().unwrap();
                if let Some(entry) = cache.get(&ip) {
                    if entry.is_valid() {
                        return entry.identity.clone();
                    }
                }
            }

            // Cache miss or expired - run whois (write lock)
            let identity = self.whois_lookup(ip);
            let mut cache = self.inner.write().unwrap();
            cache.insert(ip, CacheEntry::new(identity.clone()));
            identity
        } else {
            // No remote address - use OS user fallback
            os_user_fallback()
        }
    }

    /// Run `tailscale whois --json` for an IP address
    ///
    /// Parses the JSON output to extract:
    /// 1. UserProfile.LoginName (e.g., "user@example.com") for user connections
    /// 2. Node.ComputedName (e.g., "pixel-6") for machine/tagged connections
    /// 3. Falls back to OS user if whois fails
    fn whois_lookup(&self, ip: IpAddr) -> String {
        let output = std::process::Command::new("tailscale")
            .arg("whois")
            .arg("--json")
            .arg(ip.to_string())
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                if let Ok(json) = String::from_utf8(out.stdout) {
                    if let Ok(whois) = parse_whois_json(&json) {
                        return whois;
                    }
                }
            }
        }

        // Whois failed - fall back to OS user
        os_user_fallback()
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        let mut cache = self.inner.write().unwrap();
        cache.clear();
    }

    /// Return the number of cached entries
    pub fn len(&self) -> usize {
        let cache = self.inner.read().unwrap();
        cache.len()
    }

    /// Remove expired entries from the cache
    pub fn purge_expired(&self) {
        let mut cache = self.inner.write().unwrap();
        cache.retain(|_, entry| entry.is_valid());
    }
}

/// Parse `tailscale whois --json` output and extract a meaningful identity
///
/// JSON structure:
/// ```json
/// {
///   "Node": {
///     "ComputedName": "pixel-6",
///     "Name": "pixel-6.tail1b1987.ts.net."
///   },
///   "UserProfile": {
///     "LoginName": "user@example.com",
///     "DisplayName": "User Name"
///   }
/// }
/// ```
///
/// Returns identity string in format:
/// - `tailscale:user@example.com` - User identity (preferred)
/// - `tailscale:machine-name` - Machine identity (fallback)
fn parse_whois_json(json: &str) -> Result<String, Box<dyn std::error::Error>> {
    let v: JsonValue = serde_json::from_str(json)?;

    // Prefer UserProfile.LoginName (user's email/login)
    if let Some(profile) = v.get("UserProfile") {
        if let Some(login_name) = profile.get("LoginName").and_then(|v| v.as_str()) {
            // Filter out non-user identities like "tagged-devices"
            if !login_name.contains("tagged-devices") && !login_name.contains("Tagged Devices") {
                return Ok(format!("tailscale:{}", login_name));
            }
        }
    }

    // Fall back to Node.ComputedName (machine name)
    if let Some(node) = v.get("Node") {
        if let Some(computed_name) = node.get("ComputedName").and_then(|v| v.as_str()) {
            if !computed_name.is_empty() {
                return Ok(format!("tailscale:{}", computed_name));
            }
        }
    }

    // Final fallback to Node.Name
    if let Some(node) = v.get("Node") {
        if let Some(name) = node.get("Name").and_then(|v| v.as_str()) {
            if !name.is_empty() {
                // Strip trailing dot if present
                let name = name.trim_end_matches('.');
                return Ok(format!("tailscale:{}", name));
            }
        }
    }

    Err("Could not extract identity from whois JSON".into())
}

/// OS username fallback when Tailscale whois is unavailable
fn os_user_fallback() -> String {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    format!("os:{}", user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_user_fallback() {
        let user = os_user_fallback();
        assert!(user.starts_with("os:"));
        let name = user.strip_prefix("os:").unwrap();
        assert!(!name.is_empty());
        assert_ne!(name, "unknown"); // Should have a username in tests
    }

    #[test]
    fn test_cache_entry_validity() {
        let entry = CacheEntry::new("test_identity".to_string());
        assert!(entry.is_valid());
    }

    #[test]
    fn test_identity_cache_new() {
        let cache = IdentityCache::new();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_identity_cache_resolve_no_addr() {
        let cache = IdentityCache::new();
        let identity = cache.resolve(None);
        // Should return OS user fallback
        assert!(identity.starts_with("os:"));
    }

    #[test]
    fn test_identity_cache_clear() {
        let cache = IdentityCache::new();
        // This will cache the OS user fallback for localhost
        let _addr = "127.0.0.1:8080".parse::<std::net::SocketAddr>().ok();
        let _identity = cache.resolve(_addr);
        assert!(cache.len() > 0);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_parse_whois_json_user_email() {
        let json = r#"{
            "Node": {
                "ComputedName": "pixel-6",
                "Name": "pixel-6.tail1b1987.ts.net."
            },
            "UserProfile": {
                "LoginName": "user@example.com",
                "DisplayName": "User Name"
            }
        }"#;
        let result = parse_whois_json(json).unwrap();
        assert_eq!(result, "tailscale:user@example.com");
    }

    #[test]
    fn test_parse_whois_json_tagged_device() {
        let json = r#"{
            "Node": {
                "ComputedName": "pixel-6",
                "Name": "pixel-6.tail1b1987.ts.net."
            },
            "UserProfile": {
                "LoginName": "tagged-devices",
                "DisplayName": "Tagged Devices"
            }
        }"#;
        let result = parse_whois_json(json).unwrap();
        // Should fall back to computed name for tagged devices
        assert_eq!(result, "tailscale:pixel-6");
    }

    #[test]
    fn test_parse_whois_json_no_profile() {
        let json = r#"{
            "Node": {
                "ComputedName": "my-server",
                "Name": "my-server.tail1b1987.ts.net."
            }
        }"#;
        let result = parse_whois_json(json).unwrap();
        assert_eq!(result, "tailscale:my-server");
    }

    #[test]
    fn test_parse_whois_json_invalid_json() {
        let json = "not valid json";
        assert!(parse_whois_json(json).is_err());
    }

    #[test]
    fn test_parse_whois_json_empty() {
        let json = "{}";
        assert!(parse_whois_json(json).is_err());
    }
}
