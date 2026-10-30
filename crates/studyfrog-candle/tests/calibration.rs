#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)] // exploratory: prints its numbers
//! Choosing the semantic runtime's pivot from a labelled set (run with `--release -- --nocapture`).

use std::path::PathBuf;

mod common;

use Class::{OffTopic, Paraphrase, Partial, WrongFact};
use common::{Class, SET, verdict};
use studyfrog_candle::minilm::minilm_load;
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::runtime::SlmRuntime;
use studyfrog_slm::score::{Comparison, calibrate, score_answer};
use studyfrog_slm::vector::cosine;

fn spread(name: &str, cos: &[(Class, f32)]) {
    for class in [Paraphrase, Partial, OffTopic, WrongFact] {
        let mut v: Vec<f32> = cos
            .iter()
            .filter(|(c, _)| *c == class)
            .map(|(_, x)| *x)
            .collect();
        v.sort_by(f32::total_cmp);
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        println!(
            "{name:9} {class:10?} n={:2} min {:.2} mean {mean:.2} max {:.2}",
            v.len(),
            v[0],
            v[v.len() - 1]
        );
    }
    let score = |pivot: f32| {
        cos.iter()
            .filter(|(c, x)| *c != WrongFact && verdict(calibrate(*x, pivot)) == *c)
            .count()
    };
    let (best, hits) = (30..=90)
        .map(|p| (p as f32 / 100.0, score(p as f32 / 100.0)))
        .max_by_key(|(_, h)| *h)
        .unwrap();
    println!(
        "{name}: best pivot {best:.2} ({hits}/33 three-way); at 0.70: {}/33",
        score(0.7)
    );
}

fn accuracy(rt: &dyn SlmRuntime) -> (usize, usize, usize) {
    let (mut right, mut total, mut wrong_fact_accepted) = (0, 0, 0);
    for (reference, answer, class) in SET {
        let c = Comparison {
            reference,
            answer,
            accepted: &[],
            rubric: &[],
            min_score: 0.7,
        };
        let value = score_answer(rt, &c).unwrap().value;
        match class {
            WrongFact => wrong_fact_accepted += usize::from(value >= 0.7),
            _ => {
                total += 1;
                right += usize::from(verdict(value) == class);
            }
        }
    }
    (right, total, wrong_fact_accepted)
}

#[test]
#[cfg_attr(debug_assertions, ignore = "needs a release build")]
fn each_runtime_with_its_pivot_sorts_the_labelled_answers() {
    let home = std::env::var("HOME").unwrap();
    let dir = PathBuf::from(home).join(".local/share/studyfrog/models/all-MiniLM-L6-v2");
    if !dir.join("model.safetensors").exists() {
        println!("SKIPPED: no model");
        return;
    }
    let model = minilm_load(&dir).unwrap();
    let (sem, total, sem_wrong) = accuracy(&model);
    let (lex, _, lex_wrong) = accuracy(&Lexical);
    println!(
        "semantic {sem}/{total}, lexical {lex}/{total}; wrong-fact answers accepted: semantic {sem_wrong}/11, lexical {lex_wrong}/11"
    );
    assert!(sem >= 24, "semantic {sem}/{total}");
    assert!(lex >= 18 && sem > lex, "lexical {lex}/{total}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "needs a release build")]
fn the_pivots_come_from_the_spread_of_the_labelled_cosines() {
    let home = std::env::var("HOME").unwrap();
    let dir = PathBuf::from(home).join(".local/share/studyfrog/models/all-MiniLM-L6-v2");
    if !dir.join("model.safetensors").exists() {
        println!("SKIPPED: no model");
        return;
    }
    let model = minilm_load(&dir).unwrap();
    let cos_of = |rt: &dyn SlmRuntime| -> Vec<(Class, f32)> {
        SET.iter()
            .map(|(r, a, c)| {
                let v = rt.embed(&[r, a]).unwrap();
                (*c, cosine(&v[0], &v[1]).max(0.0))
            })
            .collect()
    };
    spread("semantic", &cos_of(&model));
    spread("lexical", &cos_of(&Lexical));
}
