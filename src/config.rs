use serde::Deserialize;
use std::path::Path;
use std::fs;
use anyhow::Result;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(rename = "ignored-attributes")]
    pub ignored_attributes: Option<Vec<String>>,
    #[serde(rename = "check-private")]
    pub check_private: Option<bool>,
    #[serde(rename = "src-dir")]
    pub src_dir: Option<String>,
    #[serde(rename = "spec-dir")]
    pub spec_dir: Option<String>,
    #[serde(rename = "log-file")]
    pub log_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    #[serde(rename = "spec-check")]
    spec_check: Option<Config>,
}

impl Config {
    pub fn load_from_cargo_toml() -> Result<Self> {
        let cargo_toml_path = Path::new("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(cargo_toml_path)?;
        let cargo: CargoToml = toml::from_str(&content)?;

        Ok(cargo.package
            .and_then(|p| p.metadata)
            .and_then(|m| m.spec_check)
            .unwrap_or_default())
    }

    pub fn get_ignored_attributes(&self) -> Vec<String> {
        self.ignored_attributes.clone().unwrap_or_else(|| vec!["doc".to_string()])
    }
}
