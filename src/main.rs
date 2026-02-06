mod rust_parser;
mod markdown_parser;
mod comparator;
mod file_walker;
mod reporter;
mod config;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::fs;

#[derive(Parser)]
#[command(name = "spec-check")]
#[command(about = "Validate Rust code against specification markdown files", long_about = None)]
struct Args {
    /// Source directory
    #[arg(short, long)]
    src: Option<PathBuf>,
    
    /// Spec directory
    #[arg(short = 'p', long)]
    spec: Option<PathBuf>,

    /// Check private items in addition to public items
    #[arg(long)]
    check_private: Option<bool>,

    /// Output log file
    #[arg(short, long)]
    log: Option<PathBuf>,

    /// Attributes to ignore (can be specified multiple times)
    #[arg(short = 'i', long)]
    ignore_attr: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = config::Config::load_from_cargo_toml().unwrap_or_default();

    // Determine final values (CLI overrides Cargo.toml metadata)
    let src = args.src
        .or_else(|| config.src_dir.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("src"));
    
    let spec = args.spec
        .or_else(|| config.spec_dir.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("spec"));
    
    let log = args.log
        .or_else(|| config.log_file.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("spec-check.log"));
    
    let check_private = args.check_private
        .or(config.check_private)
        .unwrap_or(false);

    let mut ignored_attributes = config.get_ignored_attributes();
    ignored_attributes.extend(args.ignore_attr);

    // Validate directories exist
    if !src.exists() {
        anyhow::bail!("Source directory does not exist: {}", src.display());
    }
    if !spec.exists() {
        anyhow::bail!("Spec directory does not exist: {}", spec.display());
    }

    // Initialize reporter
    let mut reporter = reporter::Reporter::new(&log)
        .context("Failed to create log file")?;

    // Find all file mappings
    let mappings = file_walker::find_file_mappings(&src, &spec)
        .context("Failed to find file mappings")?;

    let mut files_with_errors = 0;
    let total_files = mappings.len();

    // Process each file
    for mapping in &mappings {
        // Parse Rust file
        let rust_content = fs::read_to_string(&mapping.rust_file)
            .with_context(|| format!("Failed to read {}", mapping.rust_file.display()))?;
        
        let code_items = rust_parser::parse_rust_file(&rust_content, check_private)
            .with_context(|| format!("Failed to parse {}", mapping.rust_file.display()))?;

        // Check if spec file exists
        let Some(spec_file) = &mapping.spec_file else {
            reporter.report_missing_spec(&mapping.rust_file)?;
            files_with_errors += 1;
            continue;
        };

        // Parse spec file
        let spec_content = fs::read_to_string(spec_file)
            .with_context(|| format!("Failed to read {}", spec_file.display()))?;
        
        let rust_blocks = markdown_parser::extract_rust_blocks(&spec_content)
            .with_context(|| format!("Failed to parse markdown {}", spec_file.display()))?;

        // Parse all Rust blocks from spec
        let mut spec_items = Vec::new();
        for block in rust_blocks {
            if let Ok(items) = rust_parser::parse_rust_file(&block, check_private) {
                spec_items.extend(items);
            }
        }

        // Compare items
        let result = comparator::compare_items(code_items, spec_items, &ignored_attributes);
        
        if result.has_errors() {
            files_with_errors += 1;
        }

        reporter.report_results(&mapping.rust_file, &result)?;
    }

    // Write summary
    reporter.write_summary(total_files, files_with_errors)?;

    // Exit with error code if there were any errors
    if files_with_errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}
