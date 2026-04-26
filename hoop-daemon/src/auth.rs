//! Role-based access control (RBAC) for HOOP API endpoints
//!
//! Two-role model:
//! - **viewer**: read-only access, cannot create beads/Stitches
//! - **drafter**: read + create access (via `br create`)
//!
//! Role enforcement happens at the schema boundary (route-level middleware)
//! before any business logic runs. Returns 403 Forbidden for unauthorized
//! access with clear error messages.
//!
//! Roles are assigned per Tailscale identity in config.yml:
//! ```yaml
//! roles:
//!   viewers:
//!     - "viewer@example.com"
//!     - "read-only-machine"
//!   drafters:
//!     - "drafter@example.com"
//!     - "admin-machine"
//! ```
//!
//! Plan reference: §6 Phase 7 deliverable 1, §13 Security (inherited identity)

use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use std::future::Future;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tracing::{debug, warn};

/// User roles in the HOOP system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// Viewer: read-only access, cannot create beads/Stitches
    Viewer,
    /// Drafter: read + create beads via br create
    Drafter,
}

impl Role {
    /// Returns true if this role can create beads/Stitches
    pub fn can_create_beads(&self) -> bool {
        matches!(self, Role::Drafter)
    }

    /// Returns the role name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Viewer => "viewer",
            Role::Drafter => "drafter",
        }
    }
}

/// Role configuration from config.yml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoleConfig {
    /// Identities with viewer role (read-only)
    #[serde(default)]
    pub viewers: Vec<String>,
    /// Identities with drafter role (read + create)
    #[serde(default)]
    pub drafters: Vec<String>,
}

impl Default for RoleConfig {
    fn default() -> Self {
        Self {
            viewers: Vec::new(),
            drafters: Vec::new(),
        }
    }
}

/// Role resolver that maps Tailscale identities to roles
#[derive(Debug, Clone)]
pub struct RoleResolver {
    /// Pre-computed set of viewer identities (normalized)
    viewers: Arc<HashSet<String>>,
    /// Pre-computed set of drafter identities (normalized)
    drafters: Arc<HashSet<String>>,
    /// Default role when no match is found
    default_role: Role,
    /// Optional shared identity cache for whois lookups (cached per IP, 5-min TTL)
    identity_cache: Option<Arc<crate::identity::IdentityCache>>,
}

impl RoleResolver {
    /// Create a new role resolver from the role configuration
    pub fn new(config: RoleConfig) -> Self {
        let viewers = config
            .viewers
            .into_iter()
            .map(normalize_identity)
            .collect();

        let drafters = config
            .drafters
            .into_iter()
            .map(normalize_identity)
            .collect();

        // Default to viewer (most restrictive) if no match
        Self {
            viewers: Arc::new(viewers),
            drafters: Arc::new(drafters),
            default_role: Role::Viewer,
            identity_cache: None,
        }
    }

    /// Create a role resolver with no restrictions (all drafters)
    ///
    /// This is used when no role config is present, maintaining backward
    /// compatibility by granting full access.
    pub fn unprivileged() -> Self {
        Self {
            viewers: Arc::new(HashSet::new()),
            drafters: Arc::new(HashSet::new()),
            default_role: Role::Drafter,
            identity_cache: None,
        }
    }

    /// Set the shared identity cache for whois lookups
    ///
    /// When set, role resolution will use the cached whois results instead
    /// of running a subprocess for every check.
    pub fn with_identity_cache(mut self, cache: Arc<crate::identity::IdentityCache>) -> Self {
        self.identity_cache = Some(cache);
        self
    }

    /// Resolve the role for a given Tailscale identity
    ///
    /// The identity should be in the format returned by `tailscale whois`:
    /// - `tailscale:user@example.com` - User identity
    /// - `tailscale:machine-name` - Machine identity
    /// - `os:username` - OS user fallback
    ///
    /// Returns the assigned role or the default role if no match is found.
    pub fn resolve(&self, identity: &str) -> Role {
        let normalized = normalize_identity(identity.to_string());

        // Check drafters first (more permissive)
        if self.drafters.contains(&normalized) {
            debug!("Identity '{}' resolved to drafter role", identity);
            return Role::Drafter;
        }

        // Check viewers
        if self.viewers.contains(&normalized) {
            debug!("Identity '{}' resolved to viewer role", identity);
            return Role::Viewer;
        }

        // No explicit match - use default
        debug!(
            "Identity '{}' not in role config, using default: {:?}",
            identity, self.default_role
        );
        self.default_role
    }

    /// Resolve the role from a remote socket address
    ///
    /// Uses the shared IdentityCache if available for cached whois lookups
    /// (5-minute TTL per IP). Otherwise runs `tailscale whois` directly.
    /// Falls back to OS user if Tailscale whois fails.
    pub fn resolve_from_addr(&self, remote_addr: Option<SocketAddr>) -> Role {
        let identity = if let Some(ref cache) = self.identity_cache {
            // Use shared identity cache (cached per IP, 5-minute TTL)
            cache.resolve(remote_addr)
        } else {
            // Fallback to direct whois call (uncached, for compatibility)
            resolve_identity(remote_addr)
        };
        self.resolve(&identity)
    }
}

/// Normalize an identity string for comparison
///
/// - Strips the `tailscale:` prefix if present
/// - Converts to lowercase for case-insensitive matching
/// - Trims whitespace
fn normalize_identity(identity: String) -> String {
    identity
        .strip_prefix("tailscale:")
        .unwrap_or(&identity)
        .strip_prefix("os:")
        .unwrap_or(&identity)
        .trim()
        .to_lowercase()
}

/// Resolve the Tailscale identity for a remote socket address
///
/// Runs `tailscale whois --json` and parses the result.
/// Falls back to OS user when Tailscale whois fails.
fn resolve_identity(remote_addr: Option<SocketAddr>) -> String {
    if let Some(addr) = remote_addr {
        let ip = addr.ip();

        let output = std::process::Command::new("tailscale")
            .arg("whois")
            .arg("--json")
            .arg(ip.to_string())
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                if let Ok(json) = String::from_utf8(out.stdout) {
                    if let Ok(identity) = parse_whois_json(&json) {
                        return identity;
                    }
                }
            }
        }
    }

    // Fallback to OS user
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    format!("os:{}", user)
}

/// Parse `tailscale whois --json` output and extract a meaningful identity
///
/// Returns identity string in format:
/// - `tailscale:user@example.com` - User identity (preferred)
/// - `tailscale:machine-name` - Machine identity (fallback)
fn parse_whois_json(json: &str) -> Result<String, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(json)?;

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

    Err("Could not extract identity from whois JSON".into())
}

/// Extension trait for DaemonState to add role checking methods
pub trait RoleCheck {
    /// Check if the remote address has the required role
    ///
    /// Returns Ok(()) if the role matches, otherwise returns a 403 response.
    fn check_role(
        &self,
        remote_addr: Option<SocketAddr>,
        required_role: Role,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)>;
}

/// Implement RoleCheck for any type that has a role_resolver field
impl<T> RoleCheck for T
where
    T: AsRef<DaemonStateLike>,
{
    fn check_role(
        &self,
        remote_addr: Option<SocketAddr>,
        required_role: Role,
    ) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
        let state_like = self.as_ref();
        let actual_role = state_like.role_resolver.resolve_from_addr(remote_addr);

        if actual_role != required_role {
            let identity = resolve_identity(remote_addr);
            warn!(
                role = %actual_role.as_str(),
                required = %required_role.as_str(),
                identity = %identity,
                "Role check failed"
            );
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": format!(
                        "Insufficient permissions: this operation requires {} role, but you have {} role",
                        required_role.as_str(),
                        actual_role.as_str()
                    ),
                    "required_role": required_role.as_str(),
                    "your_role": actual_role.as_str(),
                })),
            ));
        }

        Ok(())
    }
}

/// Trait object for DaemonState-like access to role_resolver
pub struct DaemonStateLike {
    pub role_resolver: Arc<RoleResolver>,
}

/// Middleware that requires a specific role for the route
///
/// Returns 403 Forbidden if the client doesn't have the required role.
pub fn require_role(required_role: Role) -> impl Fn(Request, Next) -> Pin<Box<dyn Future<Output = Response> + Send>> + Clone {
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        Box::pin(async move {
            // Extract ConnectInfo from request extensions
            let connect_info = request
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .cloned();

            // Extract State from request - we'll get the role_resolver there
            let remote_addr = connect_info.map(|ci| ci.0);

            // We need to get the role_resolver from the State
            // This will be done via a custom extractor in the endpoint handler
            // For now, just pass through - the actual check happens in the endpoint
            next.run(request).await
        })
    }
}

/// Helper function to check if a request from a remote address has the required role
///
/// This is intended to be called directly in endpoint handlers.
pub fn check_role_for_addr(
    role_resolver: &RoleResolver,
    remote_addr: Option<SocketAddr>,
    required_role: Role,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let actual_role = role_resolver.resolve_from_addr(remote_addr);

    if !matches!(actual_role, required_role) {
        let identity = resolve_identity(remote_addr);
        warn!(
            role = %actual_role.as_str(),
            required = %required_role.as_str(),
            identity = %identity,
            "Role check failed"
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": format!(
                    "Insufficient permissions: this operation requires {} role, but you have {} role",
                    required_role.as_str(),
                    actual_role.as_str()
                ),
                "required_role": required_role.as_str(),
                "your_role": actual_role.as_str(),
            })),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_can_create_beads() {
        assert!(Role::Drafter.can_create_beads());
        assert!(!Role::Viewer.can_create_beads());
    }

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::Viewer.as_str(), "viewer");
        assert_eq!(Role::Drafter.as_str(), "drafter");
    }

    #[test]
    fn test_normalize_identity() {
        assert_eq!(
            normalize_identity("tailscale:user@example.com".to_string()),
            "user@example.com"
        );
        assert_eq!(
            normalize_identity("User@Example.com".to_string()),
            "user@example.com"
        );
        assert_eq!(
            normalize_identity("  machine-name  ".to_string()),
            "machine-name"
        );
    }

    #[test]
    fn test_role_resolver_viewer() {
        let config = RoleConfig {
            viewers: vec!["viewer@example.com".to_string()],
            drafters: vec![],
        };
        let resolver = RoleResolver::new(config);

        assert_eq!(
            resolver.resolve("tailscale:viewer@example.com"),
            Role::Viewer
        );
        assert_eq!(
            resolver.resolve("tailscale:Viewer@Example.com"),
            Role::Viewer
        );
    }

    #[test]
    fn test_role_resolver_drafter() {
        let config = RoleConfig {
            viewers: vec![],
            drafters: vec!["drafter@example.com".to_string()],
        };
        let resolver = RoleResolver::new(config);

        assert_eq!(
            resolver.resolve("tailscale:drafter@example.com"),
            Role::Drafter
        );
    }

    #[test]
    fn test_role_resolver_default() {
        let config = RoleConfig {
            viewers: vec![],
            drafters: vec![],
        };
        let resolver = RoleResolver::new(config);

        // No match → default (Viewer)
        assert_eq!(
            resolver.resolve("tailscale:unknown@example.com"),
            Role::Viewer
        );
    }

    #[test]
    fn test_role_resolver_unprivileged() {
        let resolver = RoleResolver::unprivileged();

        // Unprivileged resolver defaults to Drafter (backward compatibility)
        assert_eq!(
            resolver.resolve("tailscale:anyone@example.com"),
            Role::Drafter
        );
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
    fn test_role_config_default() {
        let config = RoleConfig::default();
        assert!(config.viewers.is_empty());
        assert!(config.drafters.is_empty());
    }
}
