#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Unit tests for personalized item-complexity baseline and WIP draft warnings (EPIC-077, STORY-0252).

use studyfrog_report::complexity_baseline::{
    ItemRetentionSample, check_wip_complexity, estimate_complexity_baseline,
};

#[test]
fn insufficient_history_returns_none() {
    let samples: Vec<ItemRetentionSample> = (0..5)
        .map(|_| ItemRetentionSample {
            word_count: 15,
            list_items: 0,
            familiarity: 0.9,
            reviews: 3,
        })
        .collect();
    assert!(estimate_complexity_baseline(&samples).is_none());
}

#[test]
fn warns_when_wip_draft_exceeds_retained_complexity_ceiling() {
    let samples: Vec<ItemRetentionSample> = (0..12)
        .map(|_| ItemRetentionSample {
            word_count: 18,
            list_items: 0,
            familiarity: 0.85,
            reviews: 4,
        })
        .collect();
    let profile = estimate_complexity_baseline(&samples).expect("enough samples");
    assert!(check_wip_complexity(&profile, "Short prompt?", "Short answer.").is_none());

    let dense_prompt = "Explain all stages of cellular respiration including glycolysis, pyruvate oxidation, the citric acid cycle, oxidative phosphorylation, electron transport chain complexes I through IV, chemiosmosis, ATP synthase mechanics, and substrate-level phosphorylation.";
    let dense_answer = "- Glycolysis in cytosol yields 2 ATP and 2 NADH.\n- Pyruvate oxidation in matrix.\n- Krebs cycle yields NADH and FADH2.\n- Electron transport chain drives proton motive force.";
    let warn = check_wip_complexity(&profile, dense_prompt, dense_answer)
        .expect("dense draft should trigger complexity warning");
    assert!(warn.draft_words > warn.retained_avg_words);
    assert!(warn.reason.contains("Consider splitting"));
}
