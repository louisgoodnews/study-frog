//! Integration tests for the GGUF model catalog.

use studyfrog_agent::model_catalog::{
    GgufModelEntry, default_model_catalog, filter_by_family, find_model_by_id,
    recommended_default_model, validate_file_size,
};

#[test]
#[allow(clippy::cognitive_complexity)]
fn default_catalog_contains_expected_models() {
    let catalog = default_model_catalog();
    assert!(catalog.len() >= 2);

    let qwen = find_model_by_id(&catalog, "qwen2.5-1.5b-instruct-q4_k_m");
    assert!(qwen.is_some());
    let qwen = qwen.unwrap();
    assert_eq!(qwen.family, "qwen2.5");
    assert_eq!(qwen.parameter_size, "1.5B");
    assert_eq!(qwen.quantization, "Q4_K_M");
    assert!(qwen.file_size_bytes > 0);
    assert_eq!(qwen.sha256.len(), 64);
    assert!(qwen.download_url.starts_with("https://"));
    assert_eq!(qwen.license, "Apache-2.0");

    let llama = find_model_by_id(&catalog, "llama-3.2-1b-instruct-q4_k_m");
    assert!(llama.is_some());
    let llama = llama.unwrap();
    assert_eq!(llama.family, "llama-3.2");
    assert_eq!(llama.parameter_size, "1B");
    assert_eq!(llama.quantization, "Q4_K_M");
    assert!(llama.file_size_bytes > 0);
    assert_eq!(llama.sha256.len(), 64);
    assert!(llama.download_url.starts_with("https://"));
}

#[test]
fn find_model_by_id_returns_none_for_missing() {
    let catalog = default_model_catalog();
    assert!(find_model_by_id(&catalog, "non-existent-model").is_none());
}

#[test]
fn validate_file_size_logic() {
    assert!(validate_file_size(1024, 1024));
    assert!(!validate_file_size(1024, 1023));
    assert!(!validate_file_size(1024, 1025));
    assert!(validate_file_size(0, 0));
}

#[test]
fn filter_models_by_family() {
    let catalog = default_model_catalog();
    let qwen_models = filter_by_family(&catalog, "qwen2.5");
    assert_eq!(qwen_models.len(), 1);
    assert_eq!(qwen_models[0].id, "qwen2.5-1.5b-instruct-q4_k_m");

    let empty = filter_by_family(&catalog, "gemma");
    assert!(empty.is_empty());
}

#[test]
fn recommended_default_model_returns_qwen() {
    let catalog = default_model_catalog();
    let rec = recommended_default_model(&catalog);
    assert!(rec.is_some());
    assert_eq!(rec.unwrap().id, "qwen2.5-1.5b-instruct-q4_k_m");
}

#[test]
fn serde_roundtrip_gguf_model_entry() {
    let entry = GgufModelEntry {
        id: "test-model-q4_k_m".to_string(),
        name: "Test Model".to_string(),
        family: "test".to_string(),
        parameter_size: "7B".to_string(),
        quantization: "Q4_K_M".to_string(),
        file_size_bytes: 4_500_000_000,
        sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        download_url: "https://example.com/test.gguf".to_string(),
        license: "MIT".to_string(),
    };

    let json = serde_json::to_string(&entry).expect("serialize");
    let deserialized: GgufModelEntry = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(entry, deserialized);
}
