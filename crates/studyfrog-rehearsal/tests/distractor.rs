//! Tests for semantic distractor extraction and multiple choice question generation.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use studyfrog_rehearsal::distractor::{
    DistractorCandidate, build_multiple_choice_question, cosine_similarity,
    select_semantic_distractors,
};

#[test]
fn test_cosine_similarity_identities() {
    let a = [1.0, 0.0, 0.0];
    let b = [1.0, 0.0, 0.0];
    assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

    let orthogonal = [0.0, 1.0, 0.0];
    assert!(cosine_similarity(&a, &orthogonal).abs() < 1e-6);

    let opposite = [-1.0, 0.0, 0.0];
    assert!((cosine_similarity(&a, &opposite) - (-1.0)).abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_edge_cases() {
    assert_eq!(cosine_similarity(&[], &[]), 0.0);
    assert_eq!(cosine_similarity(&[1.0], &[]), 0.0);
    assert_eq!(cosine_similarity(&[1.0, 2.0], &[1.0]), 0.0);
    assert_eq!(cosine_similarity(&[0.0, 0.0], &[0.0, 0.0]), 0.0);
    assert_eq!(cosine_similarity(&[1.0, 0.0], &[0.0, 0.0]), 0.0);
}

#[test]
fn test_select_semantic_distractors_ranking_and_deduplication() {
    let target = [1.0, 0.0, 0.0];
    let candidates = vec![
        DistractorCandidate {
            text: "Close".to_string(),
            embedding: vec![0.9, 0.1, 0.0],
        },
        DistractorCandidate {
            text: "close".to_string(), // case-insensitive duplicate of "Close"
            embedding: vec![0.95, 0.05, 0.0],
        },
        DistractorCandidate {
            text: "Medium".to_string(),
            embedding: vec![0.5, 0.5, 0.0],
        },
        DistractorCandidate {
            text: "Far".to_string(),
            embedding: vec![0.1, 0.9, 0.0],
        },
        DistractorCandidate {
            text: "CLOSE".to_string(), // duplicate
            embedding: vec![0.92, 0.08, 0.0],
        },
    ];

    let distractors = select_semantic_distractors(&target, &candidates, 2);
    assert_eq!(distractors.len(), 2);
    // Highest is "close" (0.95), "Close" and "CLOSE" are excluded as case-insensitive duplicates,
    // next highest is "Medium" (0.5).
    assert_eq!(distractors[0].to_lowercase(), "close");
    assert_eq!(distractors[1], "Medium");
}

#[test]
fn test_select_distractors_empty_or_fewer_than_requested() {
    let target = [1.0, 0.0];
    let empty: Vec<DistractorCandidate> = Vec::new();
    assert!(select_semantic_distractors(&target, &empty, 3).is_empty());

    let candidates = vec![DistractorCandidate {
        text: "OnlyOne".to_string(),
        embedding: vec![0.8, 0.2],
    }];
    let result = select_semantic_distractors(&target, &candidates, 5);
    assert_eq!(result, vec!["OnlyOne"]);
}

#[test]
fn test_build_multiple_choice_question_excludes_correct_answer() {
    let target_embedding = [1.0, 0.0, 0.0];
    let candidates = vec![
        DistractorCandidate {
            text: "Paris".to_string(), // exact match to correct_answer
            embedding: vec![1.0, 0.0, 0.0],
        },
        DistractorCandidate {
            text: "paris".to_string(), // case-insensitive match
            embedding: vec![0.99, 0.01, 0.0],
        },
        DistractorCandidate {
            text: "Lyon".to_string(),
            embedding: vec![0.8, 0.2, 0.0],
        },
        DistractorCandidate {
            text: "Marseille".to_string(),
            embedding: vec![0.7, 0.3, 0.0],
        },
        DistractorCandidate {
            text: "Nice".to_string(),
            embedding: vec![0.6, 0.4, 0.0],
        },
        DistractorCandidate {
            text: "Bordeaux".to_string(),
            embedding: vec![0.5, 0.5, 0.0],
        },
    ];

    let question = build_multiple_choice_question(
        "What is the capital of France?",
        "Paris",
        &target_embedding,
        &candidates,
        4,
        12345,
    );

    assert_eq!(question.prompt, "What is the capital of France?");
    assert_eq!(question.correct_answer, "Paris");
    assert_eq!(question.distractors.len(), 3);
    assert_eq!(question.distractors, vec!["Lyon", "Marseille", "Nice"]);
    assert_eq!(question.all_choices.len(), 4);

    // Verify correct_index points to "Paris"
    assert_eq!(
        question.all_choices[question.correct_index],
        question.correct_answer
    );
    // Neither "Paris" nor "paris" should be in distractors
    for d in &question.distractors {
        assert_ne!(d.to_lowercase(), "paris");
    }
}

#[test]
fn test_build_multiple_choice_question_deterministic_shuffle() {
    let target = [1.0, 0.0];
    let candidates = vec![
        DistractorCandidate {
            text: "D1".to_string(),
            embedding: vec![0.9, 0.1],
        },
        DistractorCandidate {
            text: "D2".to_string(),
            embedding: vec![0.8, 0.2],
        },
        DistractorCandidate {
            text: "D3".to_string(),
            embedding: vec![0.7, 0.3],
        },
    ];

    let q1 = build_multiple_choice_question("Prompt", "Answer", &target, &candidates, 4, 42);
    let q2 = build_multiple_choice_question("Prompt", "Answer", &target, &candidates, 4, 42);
    assert_eq!(q1, q2);

    let q3 = build_multiple_choice_question("Prompt", "Answer", &target, &candidates, 4, 9999);
    // Same choices set, potentially different order
    assert_eq!(q1.distractors, q3.distractors);
    assert_eq!(q3.all_choices[q3.correct_index], q3.correct_answer);
}

#[test]
fn test_build_multiple_choice_question_edge_choice_counts() {
    let target = [1.0, 0.0];
    let candidates = vec![DistractorCandidate {
        text: "D1".to_string(),
        embedding: vec![0.9, 0.1],
    }];

    // 1 choice: only correct answer
    let q1 = build_multiple_choice_question("Prompt", "Answer", &target, &candidates, 1, 0);
    assert_eq!(q1.distractors.len(), 0);
    assert_eq!(q1.all_choices, vec!["Answer"]);
    assert_eq!(q1.correct_index, 0);

    // 0 choice: graceful fallback to at least correct answer
    let q0 = build_multiple_choice_question("Prompt", "Answer", &target, &candidates, 0, 0);
    assert_eq!(q0.distractors.len(), 0);
    assert_eq!(q0.all_choices, vec!["Answer"]);
    assert_eq!(q0.correct_index, 0);
}
