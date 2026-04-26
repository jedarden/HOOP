//! build command - CLI subcommand for TestRepo

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Debug, Parser)]
pub struct BuildOpts {
    /// Target identifier
    #[arg(short, long)]
    target: Option<String>,
}

pub fn handle(opts: BuildOpts) -> Result<()> {
    println!("Executing build");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command() {
        assert!(true);
    }
}
