#![allow(missing_docs, clippy::expect_used, clippy::unwrap_used)] // criterion_group! generates an undocumented fn; benches may panic

//! 10k-note vault: scan, cold index and warm reindex (TASK-0028). The vault is generated once
//! under the system temp dir with deterministic content and removed at the end.

use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::reindex::reindex;
use studyfrog_store::scan::scan_vault;
use studyfrog_store::schema::MIGRATIONS;

const NOTES: u128 = 10_000;
const STACKS: u128 = 100;

fn note_text(n: u128) -> String {
    let filler =
        "Retrieval practice beats rereading; spacing the reviews slows forgetting. ".repeat(18);
    format!(
        "---\nid: {}\nkind: note\ntitle: Note number {n}\ntags: [bench, topic{}]\ncreated: 2026-10-30T12:00:00.000Z\n---\n\
         # Note {n}\n\nSee [[Note number {}]] and [[Note number {}#Part|next]].\n\n{filler}\n",
        format_ulid(Uid(n + 1000)),
        n % 10,
        (n + 1) % NOTES,
        (n + 7) % NOTES,
    )
}

fn build_vault() -> PathBuf {
    let root = std::env::temp_dir().join(format!("studyfrog-bench-vault-{}", std::process::id()));
    for s in 0..STACKS {
        let dir = root.join(format!("stack{s:03}"));
        fs::create_dir_all(&dir).expect("mkdir");
        let stack = format!(
            "---\nid: {}\nkind: stack\ntitle: Stack {s}\n---\n",
            format_ulid(Uid(s + 1))
        );
        fs::write(dir.join("_stack.md"), stack).expect("stack file");
    }
    for n in 0..NOTES {
        let path = root.join(format!("stack{:03}/note{n:05}.md", n % STACKS));
        fs::write(path, note_text(n)).expect("note file");
    }
    root
}

fn fresh_db() -> rusqlite::Connection {
    let mut conn = open_memory().expect("db");
    migrate(&mut conn, MIGRATIONS).expect("migrate");
    conn
}

fn bench_vault(c: &mut Criterion) {
    let root = build_vault();
    let mut group = c.benchmark_group("vault_10k");
    group.sample_size(10);
    group.bench_function("scan_parse_only", |b| {
        b.iter(|| black_box(scan_vault(&root).unwrap()))
    });
    group.bench_function("cold_reindex", |b| {
        b.iter_batched(
            fresh_db,
            |mut conn| reindex(&mut conn, &root).unwrap(),
            BatchSize::PerIteration,
        )
    });
    let mut warm = fresh_db();
    reindex(&mut warm, &root).expect("prime");
    group.bench_function("warm_reindex_no_changes", |b| {
        b.iter(|| reindex(&mut warm, &root).unwrap())
    });
    group.finish();
    cleanup(&root);
}

fn cleanup(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

criterion_group!(benches, bench_vault);
criterion_main!(benches);
