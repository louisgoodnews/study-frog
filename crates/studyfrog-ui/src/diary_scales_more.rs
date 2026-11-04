//! Standard scale definitions 7 to 13 (see `docs/specs/diary.md`).

use crate::diary_scales::StandardScaleInfo;

/// Satisfaction & Retention scale.
pub const SATISFACTION: StandardScaleInfo = StandardScaleInfo {
    id: "satisfaction",
    name: "Satisfaction & Retention",
    question: "How satisfied are you with your session and how well it will stick?",
    levels: [
        (
            "Disappointed",
            "Frustrated; feeling that time was largely wasted.",
        ),
        (
            "Unsatisfied",
            "Discontented; doubt that the material will stick.",
        ),
        (
            "Content",
            "Reasonable progress; adequately satisfied with the outcome.",
        ),
        (
            "Satisfied",
            "Solid accomplishment; confident the knowledge will stick.",
        ),
        (
            "Delighted",
            "Thrilled with progress; deep sense of fulfillment and durable memory.",
        ),
    ],
};

/// Conceptual understanding scale.
pub const UNDERSTANDING: StandardScaleInfo = StandardScaleInfo {
    id: "understanding",
    name: "Conceptual understanding",
    question: "How deeply did you comprehend the underlying concepts?",
    levels: [
        (
            "Surface rote",
            "Pure surface memorization without grasping the underlying logic.",
        ),
        (
            "Vague grasp",
            "Foggy comprehension; can recognize patterns but cannot explain why.",
        ),
        (
            "Working understanding",
            "Understand the main mechanisms and standard cases.",
        ),
        (
            "Thorough comprehension",
            "Clear understanding of concepts and their relations.",
        ),
        (
            "Deep insight",
            "Comprehensive mastery; can explain, synthesize, and transfer easily.",
        ),
    ],
};

/// Mood scale.
pub const MOOD: StandardScaleInfo = StandardScaleInfo {
    id: "mood",
    name: "Mood",
    question: "What was your emotional disposition while studying?",
    levels: [
        (
            "Frustrated / Negative",
            "Irritable, bored, or resentful toward studying.",
        ),
        (
            "Subdued",
            "Unenthusiastic or flat; treating study as a chore.",
        ),
        (
            "Neutral",
            "Calm and even-keeled; steady emotional baseline.",
        ),
        ("Positive", "Cheerful and receptive; enjoying the process."),
        (
            "Joyful / Inspired",
            "Enthusiastic, curious, and excited by learning.",
        ),
    ],
};

/// Stress & Pressure scale.
pub const STRESS: StandardScaleInfo = StandardScaleInfo {
    id: "stress",
    name: "Stress & Pressure",
    question: "What degree of stress, pressure, or anxiety did you feel?",
    levels: [
        (
            "Completely serene",
            "Total peace and relaxation; zero tension.",
        ),
        (
            "Low stress",
            "Mild awareness of deadlines, but comfortably calm.",
        ),
        (
            "Moderate pressure",
            "Noticeable deadline or test pressure; manageable arousal.",
        ),
        (
            "High stress",
            "Pronounced anxiety or worry; uneasy tension.",
        ),
        ("Overwhelmed", "Acute anxiety or panic; paralyzing stress."),
    ],
};

/// Motivation scale.
pub const MOTIVATION: StandardScaleInfo = StandardScaleInfo {
    id: "motivation",
    name: "Motivation",
    question: "How strong was your drive and desire to study?",
    levels: [
        (
            "Resistant",
            "Severe procrastination; had to force every minute.",
        ),
        (
            "Reluctant",
            "Low inclination; studied purely out of obligation.",
        ),
        (
            "Willing",
            "Neutral willingness; ready to get the work done.",
        ),
        ("Motivated", "Keen interest and proactive engagement."),
        (
            "Passionate",
            "Intense drive and curiosity; eager to explore and learn.",
        ),
    ],
};

/// Restedness & Sleep scale.
pub const SLEEP: StandardScaleInfo = StandardScaleInfo {
    id: "sleep",
    name: "Restedness & Sleep",
    question: "How rested and refreshed did you feel from your prior sleep?",
    levels: [
        (
            "Severely sleep-deprived",
            "Barely slept or heavy insomnia; cognitive fog.",
        ),
        ("Poorly rested", "Fragmented or brief sleep; woke up tired."),
        (
            "Adequately rested",
            "Sufficient rest; functioning normally.",
        ),
        ("Well-rested", "Good night's sleep; alert and refreshed."),
        (
            "Fully rejuvenated",
            "Ideal, uninterrupted deep sleep; peak physical readiness.",
        ),
    ],
};
