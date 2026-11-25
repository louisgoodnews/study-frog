//! What a model download would fetch: the files, their sources, sizes and pinned SHA-256 digests (TASK-0298).
//! Pure data and functions; nothing here touches the network or the disk.

use studyfrog_agent::model_catalog::GgufModelEntry;

/// One file of a download, with its source and the digest it must have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanFile {
    /// File name in the target folder.
    pub name: String,
    /// Direct HTTPS URL.
    pub url: String,
    /// Expected size in bytes.
    pub expected_bytes: u64,
    /// Expected lowercase hexadecimal SHA-256 digest.
    pub sha256: String,
}

/// A download shown to the user before it starts: what it is, where it comes from, and its files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Human-readable title.
    pub title: String,
    /// Host the files are fetched from (for example `huggingface.co`).
    pub source_host: String,
    /// The files, in download order.
    pub files: Vec<PlanFile>,
}

/// Sum of the expected sizes of every file in `plan`.
pub fn total_bytes(plan: &Plan) -> u64 {
    plan.files.iter().map(|f| f.expected_bytes).sum()
}

/// The host of an `https://` URL, or `None` when the URL has no host.
pub fn host_of(url: &str) -> Option<String> {
    let rest = url.strip_prefix("https://")?;
    let host = rest.split('/').next()?;
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

const MINILM_BASE: &str =
    "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main";

/// The MiniLM embedding model: the three files the loader in `studyfrog-candle` reads. Digests and sizes are the
/// values of the files fetched on 2026-11-25 (the weights match the digest Hugging Face publishes for the LFS object).
pub fn minilm_plan() -> Plan {
    Plan {
        title: "MiniLM embedding model (all-MiniLM-L6-v2)".to_string(),
        source_host: "huggingface.co".to_string(),
        files: vec![
            PlanFile {
                name: "config.json".to_string(),
                url: format!("{MINILM_BASE}/config.json"),
                expected_bytes: 612,
                sha256: "953f9c0d463486b10a6871cc2fd59f223b2c70184f49815e7efbcab5d8908b41"
                    .to_string(),
            },
            PlanFile {
                name: "tokenizer.json".to_string(),
                url: format!("{MINILM_BASE}/tokenizer.json"),
                expected_bytes: 466_247,
                sha256: "be50c3628f2bf5bb5e3a7f17b1f74611b2561a3a27eeab05e5aa30f411572037"
                    .to_string(),
            },
            PlanFile {
                name: "model.safetensors".to_string(),
                url: format!("{MINILM_BASE}/model.safetensors"),
                expected_bytes: 90_868_376,
                sha256: "53aa51172d142c89d9012cce15ae4d6cc0ca6895895114379cacb4fab128d9db"
                    .to_string(),
            },
        ],
    }
}

/// A catalog GGUF model as a plan with one file, named after the entry's id.
pub fn gguf_plan(entry: &GgufModelEntry) -> Plan {
    let source_host = host_of(&entry.download_url).unwrap_or_default();
    Plan {
        title: entry.name.clone(),
        source_host,
        files: vec![PlanFile {
            name: format!("{}.gguf", entry.id),
            url: entry.download_url.clone(),
            expected_bytes: entry.file_size_bytes,
            sha256: entry.sha256.clone(),
        }],
    }
}
