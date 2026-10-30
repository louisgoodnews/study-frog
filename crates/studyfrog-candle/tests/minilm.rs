//! The semantic runtime against a real model (run with `-- --nocapture` to see the numbers).
//! The two heavy tests are ignored in debug builds; run them with `cargo test --release -p studyfrog-candle`.
//! Skips with a message when no model directory exists: set `STUDYFROG_MODEL_DIR` or install
//! all-MiniLM-L6-v2 under `~/.local/share/studyfrog/models/all-MiniLM-L6-v2`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)] // opt-in spike: prints its numbers

use std::path::PathBuf;
use std::time::Instant;

use studyfrog_candle::minilm::{Minilm, minilm_load};
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::runtime::SlmRuntime;
use studyfrog_slm::score::{Comparison, Verdict, score_answer};

fn model_dir() -> PathBuf {
    std::env::var_os("STUDYFROG_MODEL_DIR").map_or_else(
        || {
            let home = std::env::var("HOME").unwrap_or_default();
            PathBuf::from(home).join(".local/share/studyfrog/models/all-MiniLM-L6-v2")
        },
        PathBuf::from,
    )
}

fn load() -> Option<Minilm> {
    let dir = model_dir();
    if !dir.join("model.safetensors").exists() {
        println!("SKIPPED: no model in {}", dir.display());
        return None;
    }
    Some(minilm_load(&dir).unwrap())
}

fn rss_mb() -> f64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let kb = s
        .lines()
        .find_map(|l| l.strip_prefix("VmRSS:"))
        .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<f64>().ok());
    kb.unwrap_or(0.0) / 1024.0
}

/// (reference, answer, should be accepted as correct)
const LABELLED: [(&str, &str, bool); 10] = [
    (
        "The mitochondria is the powerhouse of the cell",
        "it produces most of the cell's energy",
        true,
    ),
    (
        "Paris is the capital of France",
        "France's capital city is Paris",
        true,
    ),
    (
        "Water boils at 100 degrees Celsius at sea level",
        "at sea level water reaches its boiling point at 100 C",
        true,
    ),
    (
        "Photosynthesis converts light into chemical energy",
        "plants turn sunlight into stored sugar energy",
        true,
    ),
    (
        "The heart pumps blood around the body",
        "it moves blood through the body",
        true,
    ),
    (
        "The mitochondria is the powerhouse of the cell",
        "the capital of France is Paris",
        false,
    ),
    (
        "Paris is the capital of France",
        "Berlin is the capital of Germany",
        false,
    ),
    (
        "Water boils at 100 degrees Celsius at sea level",
        "ice melts at zero degrees",
        false,
    ),
    (
        "Photosynthesis converts light into chemical energy",
        "the heart pumps blood",
        false,
    ),
    (
        "The heart pumps blood around the body",
        "bananas are yellow",
        false,
    ),
];

fn cmp<'a>(reference: &'a str, answer: &'a str) -> Comparison<'a> {
    Comparison {
        reference,
        answer,
        accepted: &[],
        rubric: &[],
        min_score: 0.7,
    }
}

fn accuracy<R: SlmRuntime>(rt: &R, min: f32) -> (usize, Vec<f32>) {
    let mut right = 0;
    let mut values = Vec::new();
    for (r, a, want) in LABELLED {
        let s = score_answer(
            rt,
            &Comparison {
                min_score: min,
                ..cmp(r, a)
            },
        )
        .unwrap();
        values.push(s.value);
        right += usize::from((s.verdict == Verdict::Correct) == want);
    }
    (right, values)
}

#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "needs a release build: debug candle is far too slow"
)]
fn the_semantic_runtime_beats_the_lexical_one_on_paraphrases() {
    let before = rss_mb();
    let started = Instant::now();
    let Some(m) = load() else { return };
    println!(
        "load: {:.0} ms, RSS {:.0} MB -> {:.0} MB",
        started.elapsed().as_secs_f64() * 1e3,
        before,
        rss_mb()
    );
    let (sem, sem_values) = accuracy(&m, 0.55);
    let (lex, lex_values) = accuracy(&Lexical, 0.55);
    println!("semantic ({sem}/10): {sem_values:.2?}\nlexical  ({lex}/10): {lex_values:.2?}");
    assert!(sem == 10 && sem > lex, "semantic {sem}, lexical {lex}");
}

#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "needs a release build: debug candle is far too slow"
)]
fn comparing_a_thirty_word_answer_fits_the_latency_budget() {
    let Some(m) = load() else { return };
    let reference =
        "Mitochondria produce most of the chemical energy a cell needs in the form of ATP";
    let answer = "they are the part of the cell that makes ATP, so most of its usable energy comes from them";
    let rubric = [
        "ATP".to_owned(),
        "energy".to_owned(),
        "inside the cell".to_owned(),
    ];
    let c = Comparison {
        rubric: &rubric,
        ..cmp(reference, answer)
    };
    for _ in 0..10 {
        score_answer(&m, &c).unwrap(); // warm up
    }
    let mut ms: Vec<f64> = (0..200)
        .map(|_| {
            let t = Instant::now();
            score_answer(&m, &c).unwrap();
            t.elapsed().as_secs_f64() * 1e3
        })
        .collect();
    ms.sort_by(f64::total_cmp);
    let (p50, p95) = (ms[99], ms[189]);
    println!(
        "compare latency: p50 {p50:.1} ms, p95 {p95:.1} ms, max {:.1} ms, RSS {:.0} MB",
        ms[199],
        rss_mb()
    );
    assert!(p95 < 150.0, "p95 {p95} ms is over the 150 ms budget");
}

#[test]
fn the_installed_model_is_picked_up_without_a_fallback_reason() {
    let dir = model_dir();
    let Some(models) = dir.parent() else { return };
    if !dir.join("model.safetensors").exists() {
        println!("SKIPPED: no model in {}", dir.display());
        return;
    }
    let (runtime, why) =
        studyfrog_slm::models::open_runtime(Some(models), Some(studyfrog_candle::minilm_loader));
    assert_eq!(why, None);
    assert!(runtime.name().contains("MiniLM"), "{}", runtime.name());
}
