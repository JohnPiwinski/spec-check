use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct FileMapping {
    pub rust_file: PathBuf,
    pub spec_file: Option<PathBuf>,
}

pub fn find_file_mappings(src_dir: &Path, spec_dir: &Path) -> Result<Vec<FileMapping>> {
    let mut mappings = Vec::new();

    for entry in WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        let rust_file = entry.path().to_path_buf();
        
        // Calculate relative path from src_dir
        let relative_path = rust_file.strip_prefix(src_dir)?;
        
        // Convert .rs to .md and prepend spec_dir
        let spec_path = spec_dir.join(relative_path).with_extension("md");
        
        let spec_file = if spec_path.exists() {
            Some(spec_path)
        } else {
            None
        };

        mappings.push(FileMapping { rust_file, spec_file });
    }

    Ok(mappings)
}
