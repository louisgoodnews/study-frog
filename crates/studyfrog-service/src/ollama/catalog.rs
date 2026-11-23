//! The curated model catalog (`data/ollama_catalog.toml`): the models the assistant may suggest,
//! with the sizes the fit rule needs. A missing size stays `None`.

use serde::Deserialize;

/// The catalog file, compiled in.
pub const CATALOG_TOML: &str = include_str!("../../data/ollama_catalog.toml");

/// One model the assistant may suggest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogModel {
    /// The tag to pull.
    pub name: String,
    /// The parameter count as people write it (`1.5B`).
    pub parameters: String,
    /// The parameter count in millions, for ranking.
    pub parameters_millions: u32,
    /// The quantization.
    pub quantization: String,
    /// Download size in bytes; `None` when unknown.
    pub download_bytes: Option<u64>,
    /// Memory needed to run, in bytes; `None` when unknown.
    pub memory_bytes: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCatalog {
    #[serde(default)]
    model: Vec<RawModel>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawModel {
    name: String,
    parameters: String,
    parameters_millions: u32,
    quantization: String,
    download_bytes: Option<u64>,
    memory_bytes: Option<u64>,
}

/// Parse a catalog file. An entry with an empty name is an error, so a typo cannot pass silently.
pub fn parse_catalog(text: &str) -> Result<Vec<CatalogModel>, String> {
    let raw: RawCatalog = toml::from_str(text).map_err(|e| e.message().to_owned())?;
    let mut out = Vec::with_capacity(raw.model.len());
    for m in raw.model {
        if m.name.trim().is_empty() {
            return Err("a catalog entry has no name".to_owned());
        }
        out.push(CatalogModel {
            name: m.name,
            parameters: m.parameters,
            parameters_millions: m.parameters_millions,
            quantization: m.quantization,
            download_bytes: m.download_bytes,
            memory_bytes: m.memory_bytes,
        });
    }
    Ok(out)
}

/// The catalog compiled into the application.
pub fn builtin_catalog() -> Result<Vec<CatalogModel>, String> {
    parse_catalog(CATALOG_TOML)
}
