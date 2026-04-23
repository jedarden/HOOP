//! Stitch decomposition service: intent → bead graph
//!
//! Translates a Stitch draft into a bead graph based on data-driven rules
//! loaded from config.yml. Rules are configurable per Stitch kind.
//!
//! Examples:
//! - "investigation" Stitch → [task bead, review bead depending on it]
//! - "fix" Stitch with acceptance criteria → [tests bead, fix bead depending on tests, review bead depending on fix]
//!
//! The operator can override the bead graph at preview time before submission.

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A single bead in a decomposed graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphBead {
    /// Unique key within this graph (used for dependency references, not the br bead ID)
    pub key: String,
    /// Title for this bead
    pub title: String,
    /// Issue type (task, fix, review, etc.)
    pub issue_type: String,
    /// Keys of beads this one depends on
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Optional body template (may reference Stitch fields via {{title}}, {{description}})
    #[serde(default)]
    pub body_template: Option<String>,
    /// Priority override (None = inherit from Stitch)
    #[serde(default)]
    pub priority: Option<i64>,
    /// Additional labels
    #[serde(default)]
    pub labels: Vec<String>,
}

/// A complete bead graph produced by decomposition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BeadGraph {
    /// The rule name that produced this graph
    pub rule_name: String,
    /// Ordered list of beads (dependency order)
    pub beads: Vec<GraphBead>,
}

/// Stitch intent for decomposition input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StitchIntent {
    /// Kind of Stitch: investigation, fix, feature, etc.
    pub kind: String,
    /// Stitch title
    pub title: String,
    /// Stitch description / body
    pub description: Option<String>,
    /// Whether the Stitch has acceptance criteria (affects fix decomposition)
    pub has_acceptance_criteria: bool,
    /// Stitch's workspace scope
    pub project: String,
    /// Priority from the Stitch
    pub priority: Option<i64>,
    /// Labels from the Stitch
    pub labels: Vec<String>,
}

/// Override applied by the operator at preview time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphOverride {
    /// Beads to add (not in the original graph)
    #[serde(default)]
    pub add: Vec<GraphBead>,
    /// Bead keys to remove
    #[serde(default)]
    pub remove: Vec<String>,
    /// Beads to replace (matched by key)
    #[serde(default)]
    pub replace: Vec<GraphBead>,
    /// Override the rule name (for audit)
    #[serde(default)]
    pub override_reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Config-driven rules
// ---------------------------------------------------------------------------

/// A decomposition rule definition (loaded from config.yml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposeRule {
    /// Human-readable rule name
    pub name: String,
    /// Which Stitch kinds this rule matches
    pub match_kinds: Vec<String>,
    /// Whether this rule only applies when has_acceptance_criteria is true
    #[serde(default)]
    pub requires_acceptance_criteria: Option<bool>,
    /// Ordered bead templates
    pub beads: Vec<GraphBeadTemplate>,
}

/// Template for a bead within a decomposition rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphBeadTemplate {
    /// Key within the graph
    pub key: String,
    /// Issue type
    pub issue_type: String,
    /// Title template. Supports {{title}} and {{kind}} placeholders.
    pub title_template: String,
    /// Keys of beads this one depends on
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Body template (supports {{title}}, {{description}}, {{kind}})
    #[serde(default)]
    pub body_template: Option<String>,
    /// Priority override
    #[serde(default)]
    pub priority: Option<i64>,
    /// Extra labels
    #[serde(default)]
    pub labels: Vec<String>,
}

/// Full decomposition config section (embedded in config.yml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposeConfig {
    /// List of decomposition rules
    pub rules: Vec<DecomposeRule>,
}

impl Default for DecomposeConfig {
    fn default() -> Self {
        Self {
            rules: default_rules(),
        }
    }
}

/// Built-in default rules used when no config is provided
fn default_rules() -> Vec<DecomposeRule> {
    vec![
        DecomposeRule {
            name: "investigation".into(),
            match_kinds: vec!["investigation".into()],
            requires_acceptance_criteria: None,
            beads: vec![
                GraphBeadTemplate {
                    key: "task".into(),
                    issue_type: "task".into(),
                    title_template: "{{title}}".into(),
                    depends_on: vec![],
                    body_template: Some("Investigate: {{description}}".into()),
                    priority: None,
                    labels: vec![],
                },
                GraphBeadTemplate {
                    key: "review".into(),
                    issue_type: "review".into(),
                    title_template: "Review: {{title}}".into(),
                    depends_on: vec!["task".into()],
                    body_template: Some("Review findings from investigation: {{title}}".into()),
                    priority: None,
                    labels: vec![],
                },
            ],
        },
        DecomposeRule {
            name: "fix-with-criteria".into(),
            match_kinds: vec!["fix".into()],
            requires_acceptance_criteria: Some(true),
            beads: vec![
                GraphBeadTemplate {
                    key: "tests".into(),
                    issue_type: "task".into(),
                    title_template: "Tests: {{title}}".into(),
                    depends_on: vec![],
                    body_template: Some(
                        "Write tests covering acceptance criteria for: {{description}}".into(),
                    ),
                    priority: None,
                    labels: vec!["tests".into()],
                },
                GraphBeadTemplate {
                    key: "fix".into(),
                    issue_type: "fix".into(),
                    title_template: "{{title}}".into(),
                    depends_on: vec!["tests".into()],
                    body_template: Some("Fix per acceptance criteria: {{description}}".into()),
                    priority: None,
                    labels: vec![],
                },
                GraphBeadTemplate {
                    key: "review".into(),
                    issue_type: "review".into(),
                    title_template: "Review: {{title}}".into(),
                    depends_on: vec!["fix".into()],
                    body_template: Some("Review fix: {{title}}".into()),
                    priority: None,
                    labels: vec![],
                },
            ],
        },
        DecomposeRule {
            name: "fix-simple".into(),
            match_kinds: vec!["fix".into()],
            requires_acceptance_criteria: Some(false),
            beads: vec![
                GraphBeadTemplate {
                    key: "fix".into(),
                    issue_type: "fix".into(),
                    title_template: "{{title}}".into(),
                    depends_on: vec![],
                    body_template: Some("Fix: {{description}}".into()),
                    priority: None,
                    labels: vec![],
                },
                GraphBeadTemplate {
                    key: "review".into(),
                    issue_type: "review".into(),
                    title_template: "Review: {{title}}".into(),
                    depends_on: vec!["fix".into()],
                    body_template: Some("Review fix: {{title}}".into()),
                    priority: None,
                    labels: vec![],
                },
            ],
        },
        DecomposeRule {
            name: "feature".into(),
            match_kinds: vec!["feature".into()],
            requires_acceptance_criteria: None,
            beads: vec![
                GraphBeadTemplate {
                    key: "task".into(),
                    issue_type: "task".into(),
                    title_template: "{{title}}".into(),
                    depends_on: vec![],
                    body_template: Some("Implement: {{description}}".into()),
                    priority: None,
                    labels: vec![],
                },
                GraphBeadTemplate {
                    key: "review".into(),
                    issue_type: "review".into(),
                    title_template: "Review: {{title}}".into(),
                    depends_on: vec!["task".into()],
                    body_template: Some("Review implementation: {{title}}".into()),
                    priority: None,
                    labels: vec![],
                },
            ],
        },
    ]
}

// ---------------------------------------------------------------------------
// Decomposition engine (pure function)
// ---------------------------------------------------------------------------

/// Resolve a template string by substituting placeholders
fn resolve_template(template: &str, intent: &StitchIntent) -> String {
    let desc = intent.description.as_deref().unwrap_or("");
    template
        .replace("{{title}}", &intent.title)
        .replace("{{description}}", desc)
        .replace("{{kind}}", &intent.kind)
}

/// Find the matching rule for a Stitch intent
fn find_rule<'a>(rules: &'a [DecomposeRule], intent: &StitchIntent) -> Option<&'a DecomposeRule> {
    // Prefer more specific rules first (those with requires_acceptance_criteria = Some(true))
    rules.iter().find(|r| {
        r.match_kinds.contains(&intent.kind)
            && r.requires_acceptance_criteria == Some(true)
            && intent.has_acceptance_criteria
    }).or_else(|| {
        // Then rules without criteria requirement or with criteria=false
        rules.iter().find(|r| {
            r.match_kinds.contains(&intent.kind)
                && r.requires_acceptance_criteria != Some(true)
        })
    })
}

/// Decompose a Stitch intent into a bead graph.
///
/// This is a pure function: same inputs always produce the same output.
/// No side effects, no I/O.
pub fn decompose(rules: &[DecomposeRule], intent: &StitchIntent) -> Option<BeadGraph> {
    let rule = find_rule(rules, intent)?;

    let beads: Vec<GraphBead> = rule
        .beads
        .iter()
        .map(|tmpl| {
            let mut labels = tmpl.labels.clone();
            // Inherit Stitch labels
            for lbl in &intent.labels {
                if !labels.contains(lbl) {
                    labels.push(lbl.clone());
                }
            }

            GraphBead {
                key: tmpl.key.clone(),
                title: resolve_template(&tmpl.title_template, intent),
                issue_type: tmpl.issue_type.clone(),
                depends_on: tmpl.depends_on.clone(),
                body_template: tmpl.body_template.as_ref().map(|t| resolve_template(t, intent)),
                priority: tmpl.priority.or(intent.priority),
                labels,
            }
        })
        .collect();

    Some(BeadGraph {
        rule_name: rule.name.clone(),
        beads,
    })
}

/// Apply operator overrides to a bead graph.
///
/// Returns the modified graph. Override additions are appended,
/// removals delete matching keys, replacements update in-place.
pub fn apply_override(graph: &BeadGraph, override_: &GraphOverride) -> BeadGraph {
    let mut beads = graph.beads.clone();

    // Remove beads by key
    let remove_set: std::collections::HashSet<_> = override_.remove.iter().collect();
    beads.retain(|b| !remove_set.contains(&b.key));

    // Replace beads by key
    for replacement in &override_.replace {
        if let Some(existing) = beads.iter_mut().find(|b| b.key == replacement.key) {
            *existing = replacement.clone();
        }
    }

    // Add new beads
    beads.extend(override_.add.clone());

    BeadGraph {
        rule_name: if override_.override_reason.is_some() {
            format!("{} (overridden: {})", graph.rule_name, override_.override_reason.as_deref().unwrap_or("operator edit"))
        } else {
            format!("{} (overridden)", graph.rule_name)
        },
        beads,
    }
}

/// Load decomposition config from a YAML string.
/// Returns default rules if the string is empty.
pub fn load_config(yaml: &str) -> DecomposeConfig {
    if yaml.trim().is_empty() {
        return DecomposeConfig::default();
    }

    match serde_yaml::from_str::<DecomposeConfig>(yaml) {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to parse decomposition config, using defaults: {}", e);
            DecomposeConfig::default()
        }
    }
}

/// Load decomposition config from config.yml's `stitch_decompose` section.
/// Reads from `~/.hoop/config.yml` if it exists.
pub fn load_config_from_file() -> DecomposeConfig {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let config_path = home.join(".hoop").join("config.yml");

    if !config_path.exists() {
        return DecomposeConfig::default();
    }

    match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            // Extract just the stitch_decompose section if present
            match serde_yaml::from_str::<serde_yaml::Value>(&content) {
                Ok(val) => {
                    if let Some(section) = val.get("stitch_decompose") {
                        match serde_yaml::from_value::<DecomposeConfig>(section.clone()) {
                            Ok(config) => {
                                info!("Loaded {} decomposition rules from config.yml", config.rules.len());
                                config
                            }
                            Err(e) => {
                                warn!("Invalid stitch_decompose config: {}, using defaults", e);
                                DecomposeConfig::default()
                            }
                        }
                    } else {
                        DecomposeConfig::default()
                    }
                }
                Err(e) => {
                    warn!("Failed to parse config.yml: {}, using defaults", e);
                    DecomposeConfig::default()
                }
            }
        }
        Err(e) => {
            warn!("Failed to read config.yml: {}, using defaults", e);
            DecomposeConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent(kind: &str, title: &str) -> StitchIntent {
        StitchIntent {
            kind: kind.to_string(),
            title: title.to_string(),
            description: None,
            has_acceptance_criteria: false,
            project: "test-project".to_string(),
            priority: Some(2),
            labels: vec![],
        }
    }

    fn intent_with_criteria(kind: &str, title: &str) -> StitchIntent {
        StitchIntent {
            kind: kind.to_string(),
            title: title.to_string(),
            description: Some("Detailed description".to_string()),
            has_acceptance_criteria: true,
            project: "test-project".to_string(),
            priority: Some(3),
            labels: vec!["urgent".to_string()],
        }
    }

    // ---- Rule matching ----

    #[test]
    fn test_investigation_produces_task_and_review() {
        let rules = default_rules();
        let i = intent("investigation", "Why is auth flaky?");
        let graph = decompose(&rules, &i).unwrap();

        assert_eq!(graph.rule_name, "investigation");
        assert_eq!(graph.beads.len(), 2);
        assert_eq!(graph.beads[0].key, "task");
        assert_eq!(graph.beads[0].issue_type, "task");
        assert_eq!(graph.beads[0].title, "Why is auth flaky?");
        assert!(graph.beads[0].depends_on.is_empty());

        assert_eq!(graph.beads[1].key, "review");
        assert_eq!(graph.beads[1].issue_type, "review");
        assert_eq!(graph.beads[1].depends_on, vec!["task"]);
    }

    #[test]
    fn test_fix_with_criteria_produces_three_beads() {
        let rules = default_rules();
        let i = intent_with_criteria("fix", "Fix auth race condition");
        let graph = decompose(&rules, &i).unwrap();

        assert_eq!(graph.rule_name, "fix-with-criteria");
        assert_eq!(graph.beads.len(), 3);

        // tests bead (no deps)
        assert_eq!(graph.beads[0].key, "tests");
        assert_eq!(graph.beads[0].issue_type, "task");
        assert!(graph.beads[0].depends_on.is_empty());
        assert!(graph.beads[0].labels.contains(&"tests".to_string()));

        // fix bead (depends on tests)
        assert_eq!(graph.beads[1].key, "fix");
        assert_eq!(graph.beads[1].issue_type, "fix");
        assert_eq!(graph.beads[1].depends_on, vec!["tests"]);

        // review bead (depends on fix)
        assert_eq!(graph.beads[2].key, "review");
        assert_eq!(graph.beads[2].issue_type, "review");
        assert_eq!(graph.beads[2].depends_on, vec!["fix"]);
    }

    #[test]
    fn test_fix_without_criteria_produces_two_beads() {
        let rules = default_rules();
        let i = intent("fix", "Fix typo in README");
        let graph = decompose(&rules, &i).unwrap();

        assert_eq!(graph.rule_name, "fix-simple");
        assert_eq!(graph.beads.len(), 2);
        assert_eq!(graph.beads[0].key, "fix");
        assert_eq!(graph.beads[1].key, "review");
        assert_eq!(graph.beads[1].depends_on, vec!["fix"]);
    }

    #[test]
    fn test_feature_produces_task_and_review() {
        let rules = default_rules();
        let i = intent("feature", "Add dark mode");
        let graph = decompose(&rules, &i).unwrap();

        assert_eq!(graph.rule_name, "feature");
        assert_eq!(graph.beads.len(), 2);
        assert_eq!(graph.beads[0].key, "task");
        assert_eq!(graph.beads[1].key, "review");
    }

    #[test]
    fn test_unknown_kind_returns_none() {
        let rules = default_rules();
        let i = intent("unknown-kind", "Mystery");
        assert!(decompose(&rules, &i).is_none());
    }

    // ---- Template resolution ----

    #[test]
    fn test_template_title_resolution() {
        let rules = default_rules();
        let mut i = intent("investigation", "Why is auth flaky?");
        i.description = Some("Auth fails intermittently on Tuesdays.".to_string());
        let graph = decompose(&rules, &i).unwrap();

        assert_eq!(graph.beads[0].title, "Why is auth flaky?");
        assert_eq!(
            graph.beads[0].body_template.as_deref(),
            Some("Investigate: Auth fails intermittently on Tuesdays.")
        );
    }

    // ---- Label inheritance ----

    #[test]
    fn test_labels_inherited_from_stitch() {
        let rules = default_rules();
        let mut i = intent("investigation", "Check logs");
        i.labels = vec!["infra".to_string(), "high-priority".to_string()];
        let graph = decompose(&rules, &i).unwrap();

        for bead in &graph.beads {
            assert!(bead.labels.contains(&"infra".to_string()));
            assert!(bead.labels.contains(&"high-priority".to_string()));
        }
    }

    #[test]
    fn test_rule_labels_merged_with_stitch_labels() {
        let rules = default_rules();
        let mut i = intent_with_criteria("fix", "Fix auth");
        let graph = decompose(&rules, &i).unwrap();

        // tests bead has rule label "tests" + stitch label "urgent"
        let tests_bead = &graph.beads[0];
        assert!(tests_bead.labels.contains(&"tests".to_string()));
        assert!(tests_bead.labels.contains(&"urgent".to_string()));
    }

    // ---- Priority inheritance ----

    #[test]
    fn test_priority_inherited_when_template_has_none() {
        let rules = default_rules();
        let i = intent("investigation", "Something");
        let graph = decompose(&rules, &i).unwrap();

        // Template has no priority, so should inherit from intent (2)
        assert_eq!(graph.beads[0].priority, Some(2));
    }

    // ---- Overrides ----

    #[test]
    fn test_override_remove_bead() {
        let rules = default_rules();
        let i = intent("investigation", "Check");
        let graph = decompose(&rules, &i).unwrap();
        assert_eq!(graph.beads.len(), 2);

        let over = GraphOverride {
            add: vec![],
            remove: vec!["review".to_string()],
            replace: vec![],
            override_reason: Some("no review needed".to_string()),
        };
        let modified = apply_override(&graph, &over);
        assert_eq!(modified.beads.len(), 1);
        assert_eq!(modified.beads[0].key, "task");
        assert!(modified.rule_name.contains("overridden"));
    }

    #[test]
    fn test_override_add_bead() {
        let rules = default_rules();
        let i = intent("fix", "Fix typo");
        let graph = decompose(&rules, &i).unwrap();
        assert_eq!(graph.beads.len(), 2);

        let over = GraphOverride {
            add: vec![GraphBead {
                key: "docs".into(),
                title: "Update docs".into(),
                issue_type: "task".into(),
                depends_on: vec!["fix".into()],
                body_template: None,
                priority: None,
                labels: vec![],
            }],
            remove: vec![],
            replace: vec![],
            override_reason: None,
        };
        let modified = apply_override(&graph, &over);
        assert_eq!(modified.beads.len(), 3);
        assert_eq!(modified.beads[2].key, "docs");
    }

    #[test]
    fn test_override_replace_bead() {
        let rules = default_rules();
        let i = intent("investigation", "Look into it");
        let graph = decompose(&rules, &i).unwrap();

        let over = GraphOverride {
            add: vec![],
            remove: vec![],
            replace: vec![GraphBead {
                key: "review".into(),
                title: "Deep review with QA".into(),
                issue_type: "review".into(),
                depends_on: vec!["task".into()],
                body_template: Some("Full QA review".into()),
                priority: Some(5),
                labels: vec!["qa".into()],
            }],
            override_reason: Some("need thorough review".to_string()),
        };
        let modified = apply_override(&graph, &over);

        let review = modified.beads.iter().find(|b| b.key == "review").unwrap();
        assert_eq!(review.title, "Deep review with QA");
        assert_eq!(review.priority, Some(5));
        assert!(review.labels.contains(&"qa".to_string()));
    }

    // ---- Config loading ----

    #[test]
    fn test_load_config_empty_uses_defaults() {
        let config = load_config("");
        assert!(!config.rules.is_empty());
    }

    #[test]
    fn test_load_config_custom_rules() {
        let yaml = r#"
rules:
  - name: custom-rule
    match_kinds:
      - custom
    beads:
      - key: work
        issue_type: task
        title_template: "Do: {{title}}"
        depends_on: []
"#;
        let config = load_config(yaml);
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].name, "custom-rule");
    }

    #[test]
    fn test_load_config_invalid_yaml_uses_defaults() {
        let config = load_config("not: valid: yaml: [[[");
        assert!(!config.rules.is_empty());
    }

    // ---- Purity ----

    #[test]
    fn test_purity_same_inputs_same_output() {
        let rules = default_rules();
        let i = intent("investigation", "Test purity");

        let g1 = decompose(&rules, &i);
        let g2 = decompose(&rules, &i);
        let g3 = decompose(&rules, &i);

        assert_eq!(g1, g2);
        assert_eq!(g2, g3);
    }
}
