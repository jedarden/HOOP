//! Risk pattern management CLI
//!
//! `hoop risk-patterns` subcommands for managing risk patterns

use anyhow::Result;
use clap::Subcommand;
use hoop_daemon::risk_patterns::{default_risk_patterns, FixLineageLibrary, RiskPattern};
use std::fs;
use std::path::PathBuf;

/// Risk pattern management commands
#[derive(Subcommand, Debug)]
pub enum RiskPatternsCommands {
    /// Add a new risk pattern
    Add {
        /// Pattern ID (unique identifier)
        #[arg(long)]
        id: String,
        /// Pattern name
        #[arg(long)]
        name: String,
        /// Pattern description
        #[arg(long)]
        description: String,
        /// Comma-separated keywords that trigger this pattern
        #[arg(long)]
        keywords: String,
        /// Comma-separated label keywords that increase confidence
        #[arg(long)]
        label_keywords: String,
        /// Recommended fix approach
        #[arg(long)]
        fix_recommendation: String,
        /// Severity level (low, medium, high, critical)
        #[arg(long)]
        severity: String,
        /// Pattern category (performance, correctness, security, integration, code_quality, infrastructure)
        #[arg(long)]
        category: String,
    },
    /// List all risk patterns
    List {
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Seed initial risk patterns (first-run setup)
    Seed {
        /// Force re-seed even if patterns exist
        #[arg(long)]
        force: bool,
    },
}

/// Handle risk-patterns subcommands
pub async fn handle_risk_patterns(cmd: RiskPatternsCommands) -> Result<()> {
    match cmd {
        RiskPatternsCommands::Add {
            id,
            name,
            description,
            keywords,
            label_keywords,
            fix_recommendation,
            severity,
            category,
        } => {
            add_pattern(PatternCreateInfo {
                id: &id,
                name: &name,
                description: &description,
                keywords: &keywords,
                label_keywords: &label_keywords,
                fix_recommendation: &fix_recommendation,
                severity: &severity,
                category: &category,
            })?;
        }
        RiskPatternsCommands::List { json } => {
            list_patterns(json)?;
        }
        RiskPatternsCommands::Seed { force } => {
            seed_patterns(force)?;
        }
    }
    Ok(())
}

/// Get the path to the risk patterns file
fn risk_patterns_path() -> Result<PathBuf> {
    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.push(".hoop");
    home.push("risk_patterns.json");
    Ok(home)
}

/// Pattern creation parameters
struct PatternCreateInfo<'a> {
    id: &'a str,
    name: &'a str,
    description: &'a str,
    keywords: &'a str,
    label_keywords: &'a str,
    fix_recommendation: &'a str,
    severity: &'a str,
    category: &'a str,
}

/// Add a new risk pattern
fn add_pattern(info: PatternCreateInfo<'_>) -> Result<()> {
    let path = risk_patterns_path()?;

    // Load existing patterns or create empty library
    let mut library = if path.exists() {
        FixLineageLibrary::load_from_file(&path)?
    } else {
        FixLineageLibrary::new()
    };

    // Check if pattern ID already exists
    if library.patterns().iter().any(|p| p.id == info.id) {
        eprintln!("Pattern with id '{}' already exists", info.id);
        std::process::exit(1);
    }

    // Parse severity
    let severity_val = match info.severity.to_lowercase().as_str() {
        "low" => hoop_daemon::risk_patterns::RiskSeverity::Low,
        "medium" => hoop_daemon::risk_patterns::RiskSeverity::Medium,
        "high" => hoop_daemon::risk_patterns::RiskSeverity::High,
        "critical" => hoop_daemon::risk_patterns::RiskSeverity::Critical,
        _ => {
            eprintln!("Invalid severity '{}'. Must be: low, medium, high, critical", info.severity);
            std::process::exit(1);
        }
    };

    // Parse category
    let category_val = match info.category.to_lowercase().as_str() {
        "performance" => hoop_daemon::risk_patterns::RiskCategory::Performance,
        "correctness" => hoop_daemon::risk_patterns::RiskCategory::Correctness,
        "security" => hoop_daemon::risk_patterns::RiskCategory::Security,
        "integration" => hoop_daemon::risk_patterns::RiskCategory::Integration,
        "code_quality" => hoop_daemon::risk_patterns::RiskCategory::CodeQuality,
        "infrastructure" => hoop_daemon::risk_patterns::RiskCategory::Infrastructure,
        _ => {
            eprintln!(
                "Invalid category '{}'. Must be: performance, correctness, security, integration, code_quality, infrastructure",
                info.category
            );
            std::process::exit(1);
        }
    };

    // Parse keywords
    let keywords_vec: Vec<String> = info.keywords
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if keywords_vec.is_empty() {
        eprintln!("At least one keyword is required");
        std::process::exit(1);
    }

    // Parse label keywords
    let label_keywords_vec: Vec<String> = info.label_keywords
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Create the pattern
    let pattern = RiskPattern {
        id: info.id.to_string(),
        name: info.name.to_string(),
        description: info.description.to_string(),
        keywords: keywords_vec,
        label_keywords: label_keywords_vec,
        fix_recommendation: info.fix_recommendation.to_string(),
        severity: severity_val,
        category: category_val,
    };

    // Add to library
    library.add_pattern(pattern);

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Save to file
    let patterns_json = serde_json::to_string_pretty(library.patterns())?;
    fs::write(&path, patterns_json)?;

    println!("Added pattern '{}': {}", info.id, info.name);
    println!("Pattern saved to: {}", path.display());

    Ok(())
}

/// List all risk patterns
fn list_patterns(json: bool) -> Result<()> {
    let path = risk_patterns_path()?;

    let library = if path.exists() {
        FixLineageLibrary::load_from_file(&path)?
    } else {
        // Use default patterns if no custom file exists
        FixLineageLibrary::from_patterns(default_risk_patterns())
    };

    let patterns = library.patterns();

    if json {
        println!("{}", serde_json::to_string_pretty(patterns)?);
    } else {
        if patterns.is_empty() {
            println!("No risk patterns configured");
            println!("\nSeed default patterns with:");
            println!("  hoop risk-patterns seed");
        } else {
            println!("Risk Patterns ({}):", patterns.len());
            for pattern in patterns {
                let severity_str = format!("{:?}", pattern.severity).to_lowercase();
                let category_str = format!("{:?}", pattern.category).to_lowercase();
                println!("  {} ({}/{})", pattern.id, severity_str, category_str);
                println!("    Name: {}", pattern.name);
                println!("    Description: {}", pattern.description);
                println!("    Keywords: {}", pattern.keywords.join(", "));
                if !pattern.label_keywords.is_empty() {
                    println!("    Label Keywords: {}", pattern.label_keywords.join(", "));
                }
                println!("    Fix: {}", pattern.fix_recommendation);
                println!();
            }
        }
    }

    Ok(())
}

/// Seed initial risk patterns
fn seed_patterns(force: bool) -> Result<()> {
    let path = risk_patterns_path()?;

    // Check if patterns already exist
    if path.exists() && !force {
        eprintln!("Risk patterns file already exists: {}", path.display());
        eprintln!("  Use --force to overwrite with default patterns");
        std::process::exit(1);
    }

    // Get default patterns
    let default_patterns = default_risk_patterns();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Save to file
    let patterns_json = serde_json::to_string_pretty(&default_patterns)?;
    fs::write(&path, patterns_json)?;

    println!("Seeded {} default risk patterns", default_patterns.len());
    println!("Patterns saved to: {}", path.display());
    println!("\nPattern IDs:");
    for pattern in &default_patterns {
        println!("  - {}", pattern.id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_keywords() {
        let keywords = "codegen,generate,implement";
        let parsed: Vec<String> = keywords
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        assert_eq!(parsed, vec!["codegen", "generate", "implement"]);
    }

    #[test]
    fn test_parse_keywords_with_spaces() {
        let keywords = "codegen, generate, implement";
        let parsed: Vec<String> = keywords
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        assert_eq!(parsed, vec!["codegen", "generate", "implement"]);
    }

    #[test]
    fn test_parse_empty_keywords() {
        let keywords = "";
        let parsed: Vec<String> = keywords
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        assert!(parsed.is_empty());
    }
}
