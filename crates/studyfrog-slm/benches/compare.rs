//! Latency of comparing a typed answer with its reference (budget: p95 under 150 ms; ADR 0004).

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::score::{Comparison, score_answer};

fn compare(c: &mut Criterion) {
    let reference =
        "Mitochondria produce most of the chemical energy a cell needs in the form of ATP";
    let answer = "they are the part of the cell that makes ATP, so most of its usable energy";
    let rubric = [
        "ATP".to_owned(),
        "energy".to_owned(),
        "inside the cell".to_owned(),
    ];
    let cmp = Comparison {
        reference,
        answer,
        accepted: &[],
        rubric: &rubric,
        min_score: 0.7,
    };
    c.bench_function("score a 30-word answer, lexical", |b| {
        b.iter(|| score_answer(&Lexical, black_box(&cmp)));
    });
}

criterion_group!(benches, compare);
criterion_main!(benches);
