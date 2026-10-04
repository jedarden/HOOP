//! Shared daemon address resolution for CLI commands.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// The daemon bind address used when no config file or explicit address exists.
pub const DEFAULT_DAEMON_BIND_ADDR: &str = "127.0.0.1:3000";

#[derive(Debug, Deserialize)]
struct CliConfig {
    server: Option<ServerConfig>,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    bind_addr: Option<String>,
}

/// Resolve the daemon's HTTP URL, honoring an explicit command-line address.
pub fn resolve_daemon_url(explicit_addr: Option<&str>) -> String {
    resolve_daemon_url_from_config_path(&config_path(), explicit_addr)
}

/// Resolve a daemon URL using a specific config path.
///
/// This keeps the filesystem lookup injectable for tests while preserving the
/// normal `~/.hoop/config.yml` behavior used by the CLI.
fn resolve_daemon_url_from_config_path(config_path: &Path, explicit_addr: Option<&str>) -> String {
    let configured_addr = explicit_addr
        .map(str::trim)
        .filter(|addr| !addr.is_empty())
        .map(str::to_owned)
        .or_else(|| read_bind_addr(config_path));

    configured_addr
        .as_deref()
        .map(http_url)
        .unwrap_or_else(|| http_url(DEFAULT_DAEMON_BIND_ADDR))
}

fn read_bind_addr(config_path: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(config_path).ok()?;
    let config: CliConfig = serde_yaml::from_str(&contents).ok()?;
    config
        .server
        .and_then(|server| server.bind_addr)
        .map(|addr| addr.trim().to_owned())
        .filter(|addr| !addr.is_empty())
}

fn http_url(addr: &str) -> String {
    if addr.starts_with("http://") || addr.starts_with("https://") {
        addr.trim_end_matches('/').to_owned()
    } else {
        format!("http://{}", addr.trim_end_matches('/'))
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".hoop")
        .join("config.yml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn resolves_bind_addr_from_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yml");
        std::fs::write(&config_path, "server:\n  bind_addr: 127.0.0.1:39999\n").unwrap();

        assert_eq!(
            resolve_daemon_url_from_config_path(&config_path, None),
            "http://127.0.0.1:39999"
        );
    }

    #[test]
    fn falls_back_to_default_without_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("missing-config.yml");

        assert_eq!(
            resolve_daemon_url_from_config_path(&config_path, None),
            "http://127.0.0.1:3000"
        );
    }

    #[test]
    fn explicit_address_overrides_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yml");
        std::fs::write(&config_path, "server:\n  bind_addr: 127.0.0.1:39999\n").unwrap();

        assert_eq!(
            resolve_daemon_url_from_config_path(&config_path, Some("127.0.0.1:41234")),
            "http://127.0.0.1:41234"
        );
    }
}
