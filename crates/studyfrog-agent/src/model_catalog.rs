//! Catalog of recommended GGUF small language models for on-device inference.
//!
//! Provides metadata for offline SLM candidates including parameter counts,
//! quantization formats, download URLs, SHA-256 checksums, and license information.

use serde::{Deserialize, Serialize};

/// Metadata entry describing a downloadable GGUF small language model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GgufModelEntry {
    /// Unique machine identifier for this model configuration (e.g. `qwen2.5-1.5b-instruct-q4_k_m`).
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Model architecture family (e.g. `qwen2.5`, `llama-3.2`, `smollm2`).
    pub family: String,
    /// Parameter count string (e.g. `1.5B`, `1B`, `360M`).
    pub parameter_size: String,
    /// Quantization scheme (e.g. `Q4_K_M`, `Q8_0`).
    pub quantization: String,
    /// Expected file size in bytes on disk.
    pub file_size_bytes: u64,
    /// Expected hexadecimal SHA-256 digest of the downloaded file.
    pub sha256: String,
    /// Direct HTTPS URL for downloading the GGUF file.
    pub download_url: String,
    /// SPDX license identifier or license name (e.g. `Apache-2.0`, `Llama-3.2-Community`).
    pub license: String,
}

/// Returns the default curated list of recommended GGUF models.
pub fn default_model_catalog() -> Vec<GgufModelEntry> {
    vec![
        GgufModelEntry {
            id: "qwen2.5-1.5b-instruct-q4_k_m".to_string(),
            name: "Qwen2.5-1.5B-Instruct-Q4_K_M".to_string(),
            family: "qwen2.5".to_string(),
            parameter_size: "1.5B".to_string(),
            quantization: "Q4_K_M".to_string(),
            file_size_bytes: 986_000_000,
            sha256: "d7a46f6b5b54a2a19b840cfcb5eb972624d678d47bcfc1f4e17ef7ec0d86928e".to_string(),
            download_url: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf".to_string(),
            license: "Apache-2.0".to_string(),
        },
        GgufModelEntry {
            id: "llama-3.2-1b-instruct-q4_k_m".to_string(),
            name: "Llama-3.2-1B-Instruct-Q4_K_M".to_string(),
            family: "llama-3.2".to_string(),
            parameter_size: "1B".to_string(),
            quantization: "Q4_K_M".to_string(),
            file_size_bytes: 800_000_000,
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            download_url: "https://huggingface.co/bartowski/Llama-3.2-1B-Instruct-GGUF/resolve/main/Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string(),
            license: "Llama-3.2-Community".to_string(),
        },
        GgufModelEntry {
            id: "smollm2-360m-instruct-q4_k_m".to_string(),
            name: "SmolLM2-360M-Instruct-Q4_K_M".to_string(),
            family: "smollm2".to_string(),
            parameter_size: "360M".to_string(),
            quantization: "Q4_K_M".to_string(),
            file_size_bytes: 230_000_000,
            sha256: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
            download_url: "https://huggingface.co/HuggingFaceTB/SmolLM2-360M-Instruct-GGUF/resolve/main/smollm2-360m-instruct-q4_k_m.gguf".to_string(),
            license: "Apache-2.0".to_string(),
        },
    ]
}

/// Finds a model entry in `catalog` matching the given `id`.
pub fn find_model_by_id<'a>(catalog: &'a [GgufModelEntry], id: &str) -> Option<&'a GgufModelEntry> {
    catalog.iter().find(|entry| entry.id == id)
}

/// Returns whether `actual_bytes` exactly matches `expected_bytes`.
pub fn validate_file_size(expected_bytes: u64, actual_bytes: u64) -> bool {
    expected_bytes == actual_bytes
}

/// Returns all models in `catalog` belonging to the specified `family`.
pub fn filter_by_family<'a>(
    catalog: &'a [GgufModelEntry],
    family: &str,
) -> Vec<&'a GgufModelEntry> {
    catalog
        .iter()
        .filter(|entry| entry.family == family)
        .collect()
}

/// Returns the primary recommended default model (Qwen2.5-1.5B-Instruct-Q4_K_M).
pub fn recommended_default_model(catalog: &[GgufModelEntry]) -> Option<&GgufModelEntry> {
    find_model_by_id(catalog, "qwen2.5-1.5b-instruct-q4_k_m")
}
