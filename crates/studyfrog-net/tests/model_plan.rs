//! Download plans, the catalog's pinned digests and the confirmation gate (TASK-0298).

use std::path::PathBuf;

use studyfrog_agent::model_catalog::default_model_catalog;
use studyfrog_net::model_fetch::fetch_confirmed;
use studyfrog_net::model_plan::{gguf_plan, host_of, minilm_plan, total_bytes};

fn is_hex_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

#[test]
fn the_minilm_plan_lists_three_files_from_hugging_face_with_their_sizes() {
    let plan = minilm_plan();
    assert_eq!(plan.source_host, "huggingface.co");
    let names: Vec<&str> = plan.files.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(
        names,
        ["config.json", "tokenizer.json", "model.safetensors"]
    );
    assert_eq!(total_bytes(&plan), 612 + 466_247 + 90_868_376);
    for file in &plan.files {
        assert!(is_hex_digest(&file.sha256), "{}", file.name);
        assert_eq!(host_of(&file.url).as_deref(), Some("huggingface.co"));
    }
}

#[test]
fn every_catalog_entry_has_a_real_digest_and_a_matching_source() {
    for entry in default_model_catalog() {
        assert!(is_hex_digest(&entry.sha256), "{}", entry.id);
        assert_ne!(
            entry.sha256, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "{} carries the digest of an empty file",
            entry.id
        );
        let plan = gguf_plan(&entry);
        assert_eq!(plan.source_host, "huggingface.co", "{}", entry.id);
        assert_eq!(total_bytes(&plan), entry.file_size_bytes, "{}", entry.id);
    }
}

#[test]
fn host_of_reads_https_urls_and_refuses_the_rest() {
    assert_eq!(
        host_of("https://Huggingface.co/a/b").as_deref(),
        Some("huggingface.co")
    );
    assert_eq!(host_of("http://example.com/x"), None);
    assert_eq!(host_of("https:///x"), None);
}

#[test]
fn declining_the_plan_contacts_nothing_and_writes_nothing() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("sf-plan-declined-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut asked = None;
    let res = fetch_confirmed(
        &minilm_plan(),
        &dir,
        |plan| {
            asked = Some((plan.source_host.clone(), total_bytes(plan)));
            false
        },
        |_, _| {},
    );
    assert_eq!(res.unwrap_err(), "the download was not confirmed");
    assert_eq!(
        asked,
        Some(("huggingface.co".to_string(), 612 + 466_247 + 90_868_376))
    );
    assert!(
        !dir.exists(),
        "no folder is created before the download is confirmed"
    );
}

/// Fetches the real MiniLM files into a scratch folder: about 91 MB from huggingface.co. Run on demand with
/// `cargo test -p studyfrog-net --test model_plan -- --ignored`.
#[test]
#[ignore = "downloads about 91 MB from huggingface.co"]
fn the_minilm_plan_downloads_and_verifies_every_file() {
    let dir: PathBuf = std::env::temp_dir().join(format!("sf-minilm-fetch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    fetch_confirmed(&minilm_plan(), &dir, |_| true, |_, _| {}).expect("download");
    for file in minilm_plan().files {
        let ok =
            studyfrog_net::model_download::verify_file_sha256(&dir.join(&file.name), &file.sha256)
                .expect("read");
        assert!(ok, "{}", file.name);
    }
    // A second call finds every file verified and fetches nothing.
    let mut touched = 0;
    fetch_confirmed(&minilm_plan(), &dir, |_| true, |_, _| touched += 1).expect("second call");
    assert_eq!(touched, 0);
    let _ = std::fs::remove_dir_all(&dir);
}
