//! Golden transcripts regression test for agent adapters.
//!
//! Validates acceptance criteria from hoop-ttb.11.8:
//! - Golden-transcripts corpus exists per adapter (Claude, Codex, OpenCode, Gemini, Aider)
//! - Each adapter has ≥ 3 scenarios: simple turn, tool-heavy turn, failure turn
//! - Parser regressions produce visible diffs in CI
//! - Corpus is bounded size (<10MB)
//!
//! The fixture files at testrepo/golden-transcripts/<adapter>/<version>/*.jsonl
//! serve as the canonical reference for what each adapter's parser must handle.
//!
//! Plan reference: §14.3 golden transcripts

use hoop_daemon::agent_adapter::AdapterKind;
use std::fs;
use std::path::PathBuf;

// ── Fixture paths ────────────────────────────────────────────────────────────

fn testrepo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root is parent of hoop-daemon/")
        .join("testrepo")
}

fn golden_transcripts_root() -> PathBuf {
    testrepo_root().join("golden-transcripts")
}

/// All adapters that must have golden transcripts.
const ADAPTERS: &[&str] = &["claude", "codex", "opencode", "gemini", "aider"];

/// All scenarios that must be present for each adapter.
const SCENARIOS: &[&str] = &["simple", "tool_heavy", "failure"];

/// Expected version directory name.
const VERSION: &str = "v1.0";

// ── Fixture sanity checks ─────────────────────────────────────────────────────

#[test]
fn golden_transcripts_directory_exists() {
    let root = golden_transcripts_root();
    assert!(
        root.exists(),
        "testrepo/golden-transcripts/ must exist — create it with adapter subdirectories"
    );
    assert!(
        root.is_dir(),
        "testrepo/golden-transcripts/ must be a directory"
    );
}

#[test]
fn all_adapters_have_golden_transcripts() {
    let root = golden_transcripts_root();

    for &adapter in ADAPTERS {
        let adapter_path = root.join(adapter).join(VERSION);
        assert!(
            adapter_path.exists(),
            "testrepo/golden-transcripts/{adapter}/{VERSION}/ must exist — create adapter directory"
        );
        assert!(
            adapter_path.is_dir(),
            "testrepo/golden-transcripts/{adapter}/{VERSION}/ must be a directory"
        );
    }
}

#[test]
fn all_scenarios_exist_for_each_adapter() {
    let root = golden_transcripts_root();

    for &adapter in ADAPTERS {
        for &scenario in SCENARIOS {
            let scenario_path = root.join(adapter).join(VERSION).join(scenario);
            assert!(
                scenario_path.exists(),
                "testrepo/golden-transcripts/{adapter}/{VERSION}/{scenario}/ must exist"
            );
            assert!(
                scenario_path.is_dir(),
                "testrepo/golden-transcripts/{adapter}/{VERSION}/{scenario}/ must be a directory"
            );

            // Each scenario should have at least one .jsonl file
            let jsonl_files: Vec<_> = fs::read_dir(&scenario_path)
                .unwrap_or_else(|e| {
                    panic!("Failed to read scenario directory {scenario_path:?}: {e}")
                })
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry.path().extension().map(|ext| ext == "jsonl").unwrap_or(false)
                })
                .collect();

            assert!(
                !jsonl_files.is_empty(),
                "testrepo/golden-transcripts/{adapter}/{VERSION}/{scenario}/ must contain at least one .jsonl file"
            );
        }
    }
}

#[test]
fn corpus_size_is_bounded() {
    let root = golden_transcripts_root();
    let mut total_size = 0u64;

    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().map(|ext| ext == "jsonl").unwrap_or(false) {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }

    const MAX_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10MB
    assert!(
        total_size < MAX_SIZE_BYTES,
        "Golden transcripts corpus must be < 10MB, currently {} bytes",
        total_size
    );
}

// ── Content validation ────────────────────────────────────────────────────────

/// All JSONL files must contain valid JSON (one object per line).
#[test]
fn all_jsonl_files_contain_valid_json() {
    let root = golden_transcripts_root();

    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map(|ext| ext == "jsonl").unwrap_or(false) {
            let content = fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

            for (i, line) in content.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                let _: serde_json::Value = serde_json::from_str(line)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Invalid JSON on line {} of {:?}: {}\n  Line: {}",
                            i + 1,
                            path,
                            e,
                            line
                        )
                    });
            }
        }
    }
}

/// Each scenario file should contain non-empty content.
#[test]
fn all_scenario_files_have_content() {
    let root = golden_transcripts_root();

    for &adapter in ADAPTERS {
        for &scenario in SCENARIOS {
            let scenario_dir = root.join(adapter).join(VERSION).join(scenario);
            let entries: Vec<_> = fs::read_dir(&scenario_dir)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", scenario_dir, e))
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
                .collect();

            for entry in entries {
                let path = entry.path();
                let content = fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

                // Count non-empty, non-whitespace lines
                let non_empty_lines = content
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .count();

                assert!(
                    non_empty_lines > 0,
                    "Golden transcript file {:?} must contain at least one non-empty JSON line",
                    path
                );
            }
        }
    }
}

/// Simple turn scenarios should contain text/content events.
#[test]
fn simple_turn_scenarios_contain_text_events() {
    let root = golden_transcripts_root();

    for &adapter in ADAPTERS {
        let simple_dir = root.join(adapter).join(VERSION).join("simple");
        let entries: Vec<_> = fs::read_dir(&simple_dir)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", simple_dir, e))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
            .collect();

        for entry in entries {
            let path = entry.path();
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

            // At least one line should contain a text-related field
            let has_text = content.lines().any(|line| {
                if line.trim().is_empty() {
                    return false;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                    // Check for common text fields across adapter formats
                    val.get("text").is_some()
                        || val.get("content").is_some()
                        || val.get("delta").and_then(|d| d.get("text")).is_some()
                        || val.get("candidates").and_then(|c| c.as_array()).and_then(|arr| {
                            arr.first().and_then(|cand| {
                                cand.get("content").and_then(|content| {
                                    content.get("parts").and_then(|parts| {
                                        parts.as_array().and_then(|parts_arr| {
                                            parts_arr.first().and_then(|part| part.get("text"))
                                        })
                                    })
                                })
                            })
                        }).is_some()
                } else {
                    false
                }
            });

            assert!(
                has_text,
                "Simple turn scenario {:?} for adapter '{}' must contain at least one text event",
                path,
                adapter
            );
        }
    }
}

/// Tool-heavy scenarios should contain tool use/call events.
#[test]
fn tool_heavy_scenarios_contain_tool_events() {
    let root = golden_transcripts_root();

    for &adapter in ADAPTERS {
        let tool_dir = root.join(adapter).join(VERSION).join("tool_heavy");
        let entries: Vec<_> = fs::read_dir(&tool_dir)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", tool_dir, e))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
            .collect();

        for entry in entries {
            let path = entry.path();
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

            // At least one line should contain a tool-related field
            let has_tool = content.lines().any(|line| {
                if line.trim().is_empty() {
                    return false;
                }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                    // Check for common tool fields across adapter formats
                    val.get("tool_use").is_some()
                        || val.get("tool_call").is_some()
                        || val.get("functionCall").is_some()
                        || val.get("tool").is_some()
                        || val.get("name").and_then(|n| n.as_str()).map(|s| {
                            // Check if this looks like a tool name (common tool names)
                            matches!(
                                s,
                                "read_file" | "write_file" | "list_files" | "browse"
                                    | "search_files" | "run_command"
                            )
                        }).unwrap_or(false)
                } else {
                    false
                }
            });

            assert!(
                has_tool,
                "Tool-heavy scenario {:?} for adapter '{}' must contain at least one tool event",
                path,
                adapter
            );
        }
    }
}

/// Failure scenarios should contain error events or error-related fields.
#[test]
fn failure_scenarios_contain_error_events() {
    let root = golden_transcripts_root();

    for &adapter in ADAPTERS {
        let failure_dir = root.join(adapter).join(VERSION).join("failure");
        let entries: Vec<_> = fs::read_dir(&failure_dir)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", failure_dir, e))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
            .collect();

        for entry in entries {
            let path = entry.path();
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

            // At least one line should contain an error-related field
            let has_error = content.lines().any(|line| {
                if line.trim().is_empty() {
                    return false;
                }
                let line_lower = line.to_lowercase();
                line_lower.contains("\"error\"")
                    || line_lower.contains("\"type\":\"error\"")
                    || line_lower.contains("\"error\":")
                    || line_lower.contains("rate limit")
                    || line_lower.contains("authentication")
                    || line_lower.contains("quota")
                    || line_lower.contains("failed")
            });

            assert!(
                has_error,
                "Failure scenario {:?} for adapter '{}' must contain at least one error indication",
                path,
                adapter
            );
        }
    }
}

/// AdapterKind enum covers all adapters in the golden transcripts.
#[test]
fn adapter_kind_enum_covers_all_adapters() {
    for &adapter in ADAPTERS {
        let kind = AdapterKind::from_config(adapter);
        assert!(
            kind.is_some(),
            "AdapterKind::from_config must recognize adapter '{}': add it to the enum",
            adapter
        );
    }
}

/// Verify the file structure is exactly as expected.
#[test]
fn golden_transcripts_structure_matches_spec() {
    let root = golden_transcripts_root();

    // Check that adapters are directories
    for &adapter in ADAPTERS {
        let adapter_path = root.join(adapter);
        assert!(
            adapter_path.is_dir(),
            "Adapter path {:?} must be a directory",
            adapter_path
        );
    }

    // Check that each adapter has the version directory
    for &adapter in ADAPTERS {
        let version_path = root.join(adapter).join(VERSION);
        assert!(
            version_path.is_dir(),
            "Version path {:?} must be a directory",
            version_path
        );
    }

    // Check that each version has all scenarios
    for &adapter in ADAPTERS {
        for &scenario in SCENARIOS {
            let scenario_path = root.join(adapter).join(VERSION).join(scenario);
            assert!(
                scenario_path.is_dir(),
                "Scenario path {:?} must be a directory",
                scenario_path
            );
        }
    }
}

/// Verify no unexpected files in the corpus (only .jsonl files expected, plus README.md).
#[test]
fn corpus_contains_only_jsonl_files() {
    let root = golden_transcripts_root();

    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            // Allow README.md as documentation
            if path.file_name() == Some(std::ffi::OsStr::new("README.md")) {
                continue;
            }
            let ext = path.extension();
            assert!(
                ext.map(|e| e == "jsonl").unwrap_or(false),
                "Corpus should only contain .jsonl files (plus README.md), found {:?}",
                path
            );
        }
    }
}

/// README should exist in the golden-transcripts directory explaining the format.
#[test]
fn golden_transcripts_has_readme() {
    let readme_path = golden_transcripts_root().join("README.md");
    assert!(
        readme_path.exists(),
        "testrepo/golden-transcripts/README.md must exist — document the fixture format"
    );
}

// ── Parser regression tests ─────────────────────────────────────────────────────

use hoop_daemon::agent_adapter::parse_claude_stream_line;

/// All golden transcript lines should parse successfully to AgentEvent.
#[test]
fn all_golden_transcripts_parse_successfully() {
    let root = golden_transcripts_root();

    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map(|ext| ext == "jsonl").unwrap_or(false) {
            let content = fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

            for (i, line) in content.lines().enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                let result = parse_claude_stream_line(line);
                assert!(
                    result.is_ok(),
                    "Failed to parse line {} of {:?}:\n  Line: {}\n  Error: {:?}",
                    i + 1,
                    path,
                    line,
                    result.err()
                );
            }
        }
    }
}

/// Simple turn scenarios should parse to TextDelta events.
#[test]
fn simple_turn_scenarios_parse_to_text_delta() {
    let root = golden_transcripts_root();

    for &adapter in ADAPTERS {
        let simple_dir = root.join(adapter).join(VERSION).join("simple");
        let entries: Vec<_> = fs::read_dir(&simple_dir)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", simple_dir, e))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
            .collect();

        for entry in entries {
            let path = entry.path();
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

            let mut has_text_delta = false;
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(event) = parse_claude_stream_line(line) {
                    if matches!(event, hoop_daemon::agent_adapter::AgentEvent::TextDelta { .. }) {
                        has_text_delta = true;
                        break;
                    }
                }
            }

            assert!(
                has_text_delta,
                "Simple turn scenario {:?} for adapter '{}' must parse to at least one TextDelta event",
                path,
                adapter
            );
        }
    }
}

/// Tool-heavy scenarios should parse to ToolUse and ToolResult events.
#[test]
fn tool_heavy_scenarios_parse_to_tool_events() {
    let root = golden_transcripts_root();

    for &adapter in ADAPTERS {
        let tool_dir = root.join(adapter).join(VERSION).join("tool_heavy");
        let entries: Vec<_> = fs::read_dir(&tool_dir)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", tool_dir, e))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
            .collect();

        for entry in entries {
            let path = entry.path();
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

            let mut has_tool_use = false;
            let mut has_tool_result = false;
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(event) = parse_claude_stream_line(line) {
                    match event {
                        hoop_daemon::agent_adapter::AgentEvent::ToolUse { .. } => has_tool_use = true,
                        hoop_daemon::agent_adapter::AgentEvent::ToolResult { .. } => has_tool_result = true,
                        _ => {}
                    }
                }
            }

            assert!(
                has_tool_use,
                "Tool-heavy scenario {:?} for adapter '{}' must parse to at least one ToolUse event",
                path,
                adapter
            );
            assert!(
                has_tool_result,
                "Tool-heavy scenario {:?} for adapter '{}' must parse to at least one ToolResult event",
                path,
                adapter
            );
        }
    }
}

/// Failure scenarios should parse to Error events.
#[test]
fn failure_scenarios_parse_to_error_events() {
    let root = golden_transcripts_root();

    for &adapter in ADAPTERS {
        let failure_dir = root.join(adapter).join(VERSION).join("failure");
        let entries: Vec<_> = fs::read_dir(&failure_dir)
            .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", failure_dir, e))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "jsonl").unwrap_or(false))
            .collect();

        for entry in entries {
            let path = entry.path();
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));

            let mut has_error = false;
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(event) = parse_claude_stream_line(line) {
                    if matches!(event, hoop_daemon::agent_adapter::AgentEvent::Error { .. }) {
                        has_error = true;
                        break;
                    }
                }
            }

            assert!(
                has_error,
                "Failure scenario {:?} for adapter '{}' must parse to at least one Error event",
                path,
                adapter
            );
        }
    }
}
