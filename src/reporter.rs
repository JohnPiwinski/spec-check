use crate::comparator::ComparisonResult;
use crate::rust_parser::{RustItem, ItemKind};
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub struct Reporter {
    log_file: std::fs::File,
}

impl Reporter {
    pub fn new(log_path: &Path) -> Result<Self> {
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)?;
        
        Ok(Self { log_file })
    }

    pub fn report_missing_spec(&mut self, file: &Path) -> Result<()> {
        writeln!(self.log_file, "WARNING: No spec file found for {}", file.display())?;
        Ok(())
    }

    pub fn report_results(&mut self, file: &Path, result: &ComparisonResult) -> Result<()> {
        if !result.has_errors() {
            writeln!(self.log_file, "OK: {}", file.display())?;
            return Ok(());
        }

        writeln!(self.log_file, "\nERROR: {}", file.display())?;

        // Report items in code but not in spec
        if !result.missing_in_spec.is_empty() {
            writeln!(self.log_file, "  Items in code but not in spec:")?;
            for item in &result.missing_in_spec {
                writeln!(self.log_file, "    - {} (line {})", format_item(item), item.line_number)?;
            }
        }

        // Report items in spec but not in code
        if !result.missing_in_code.is_empty() {
            writeln!(self.log_file, "  Items in spec but not in code:")?;
            for item in &result.missing_in_code {
                writeln!(self.log_file, "    - {} (line {})", format_item(item), item.line_number)?;
            }
        }

        // Report signature mismatches
        if !result.signature_mismatches.is_empty() {
            writeln!(self.log_file, "  Signature mismatches:")?;
            for mismatch in &result.signature_mismatches {
                writeln!(self.log_file, "    - {}", format_item(&mismatch.code_item))?;
                writeln!(self.log_file, "      Code (line {}): {}", mismatch.code_item.line_number, &mismatch.code_item.signature)?;
                writeln!(self.log_file, "      Spec (line {}): {}", mismatch.spec_item.line_number, &mismatch.spec_item.signature)?;
                if let Some(pos) = mismatch.first_diff_pos {
                    writeln!(self.log_file, "      First difference at character {}", pos)?;
                }
            }
        }

        // Report attribute mismatches
        if !result.attribute_mismatches.is_empty() {
            writeln!(self.log_file, "  Attribute mismatches:")?;
            for mismatch in &result.attribute_mismatches {
                writeln!(self.log_file, "    - {} (code line {}, spec line {})", 
                    format_item(&mismatch.code_item),
                    mismatch.code_item.line_number,
                    mismatch.spec_item.line_number)?;
                writeln!(self.log_file, "      Code attributes: {}", format_attributes(&mismatch.code_item.attributes))?;
                writeln!(self.log_file, "      Spec attributes: {}", format_attributes(&mismatch.spec_item.attributes))?;
            }
        }

        Ok(())
    }

    pub fn write_summary(&mut self, total_files: usize, files_with_errors: usize) -> Result<()> {
        writeln!(self.log_file, "\n{}", "=".repeat(80))?;
        writeln!(self.log_file, "SUMMARY")?;
        writeln!(self.log_file, "Total files checked: {}", total_files)?;
        writeln!(self.log_file, "Files with errors: {}", files_with_errors)?;
        writeln!(self.log_file, "Files passing: {}", total_files - files_with_errors)?;
        Ok(())
    }
}

fn format_item(item: &RustItem) -> String {
    match &item.kind {
        ItemKind::Struct => format!("struct {}", item.name),
        ItemKind::Enum => format!("enum {}", item.name),
        ItemKind::Trait => format!("trait {}", item.name),
        ItemKind::TraitMethod { trait_name } => format!("{}::{}", trait_name, item.name),
        ItemKind::Function => format!("fn {}", item.name),
    }
}

fn format_attributes(attrs: &[String]) -> String {
    if attrs.is_empty() {
        "none".to_string()
    } else {
        attrs.join(", ")
    }
}

