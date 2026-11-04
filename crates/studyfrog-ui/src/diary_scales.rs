//! Standard rating scales and exact wording for each scale level (1 to 5).
//! See Section 2 of `docs/specs/diary.md`.

/// Information about one standard self-assessment scale.
pub struct StandardScaleInfo {
    /// Scale key matching front-matter (e.g. "performance").
    pub id: &'static str,
    /// Display name (e.g. "Performance").
    pub name: &'static str,
    /// Explanatory question prompt.
    pub question: &'static str,
    /// Exact level wording (label, description) for levels 1 to 5.
    pub levels: [(&'static str, &'static str); 5],
}

const PERFORMANCE: StandardScaleInfo = StandardScaleInfo {
    id: "performance",
    name: "Performance",
    question: "How well did you perform overall during this session?",
    levels: [
        (
            "Poor",
            "Struggled significantly; remembered very little and made frequent errors.",
        ),
        (
            "Below average",
            "Below expectations; hesitant recall with notable gaps.",
        ),
        (
            "Average",
            "Acceptable performance; remembered core material with expected slip-ups.",
        ),
        (
            "Good",
            "Strong performance; prompt and accurate responses across most material.",
        ),
        (
            "Outstanding",
            "Flawless recall; effortless mastery and immediate recall throughout.",
        ),
    ],
};

const SUCCESS: StandardScaleInfo = StandardScaleInfo {
    id: "success",
    name: "Goal success",
    question: "Did you achieve your intended study goal for this session?",
    levels: [
        (
            "Not achieved",
            "Abandoned early or failed to complete intended targets.",
        ),
        (
            "Partially achieved",
            "Completed less than half of what was planned.",
        ),
        (
            "Mostly achieved",
            "Finished the core task, leaving minor items unfinished.",
        ),
        (
            "Fully achieved",
            "Completed all planned items and reviews as intended.",
        ),
        (
            "Exceeded",
            "Surpassed the study goal, completing extra material or deep reviews.",
        ),
    ],
};

const CONFIDENCE: StandardScaleInfo = StandardScaleInfo {
    id: "confidence",
    name: "Confidence",
    question: "How certain and confident are you in your grasp of the material?",
    levels: [
        (
            "Very doubtful",
            "Constantly guessing; low belief in retention.",
        ),
        ("Insecure", "Unsteady grasp; frequent second-guessing."),
        (
            "Moderately confident",
            "Reasonable confidence in familiar areas, uncertain on edges.",
        ),
        (
            "Confident",
            "Solid assurance; sure of answers without hesitation.",
        ),
        (
            "Fully confident",
            "Complete conviction; total trust in retention and mastery.",
        ),
    ],
};

const EFFORT: StandardScaleInfo = StandardScaleInfo {
    id: "effort",
    name: "Mental effort",
    question: "How much mental exertion was required?",
    levels: [
        ("Minimal effort", "Breezed through without active strain."),
        (
            "Light effort",
            "Modest concentration required; smooth pacing.",
        ),
        (
            "Moderate effort",
            "Balanced mental exertion; healthy working engagement.",
        ),
        (
            "High effort",
            "Intensive cognitive strain; sustained hard concentration.",
        ),
        (
            "Exhausting effort",
            "Maximum cognitive exertion; pushed mental capacity to the limit.",
        ),
    ],
};

const FOCUS: StandardScaleInfo = StandardScaleInfo {
    id: "focus",
    name: "Focus & Attention",
    question: "How well did you maintain concentration and resist distractions?",
    levels: [
        (
            "Highly distracted",
            "Constant interruptions, daydreaming, or multitasking.",
        ),
        (
            "Wandering",
            "Frequent lapses in focus; difficult to stay on track.",
        ),
        (
            "Moderate focus",
            "Mostly on task with periodic brief distractions.",
        ),
        (
            "Sharp focus",
            "Sustained attention; easily ignored minor distractions.",
        ),
        (
            "Deep immersion",
            "Total immersion / flow state; completely undivided attention.",
        ),
    ],
};

const ENERGY: StandardScaleInfo = StandardScaleInfo {
    id: "energy",
    name: "Energy level",
    question: "What was your energy and vitality level during the session?",
    levels: [
        (
            "Drained",
            "Exhausted and fatigued; struggling to stay awake.",
        ),
        (
            "Low",
            "Sluggish and lethargic; low physical/mental stamina.",
        ),
        (
            "Steady",
            "Normal baseline energy; sufficient to study comfortably.",
        ),
        ("Energetic", "Lively and alert; active mental presence."),
        (
            "Peaked",
            "High vigor and peak alertness; brimming with vitality.",
        ),
    ],
};

const DIFFICULTY: StandardScaleInfo = StandardScaleInfo {
    id: "difficulty",
    name: "Perceived difficulty",
    question: "How challenging did the session content feel?",
    levels: [
        (
            "Very easy",
            "Trivial; basic recall requiring no problem-solving.",
        ),
        (
            "Easy",
            "Straightforward; comfortable material with few obstacles.",
        ),
        (
            "Moderate",
            "Balanced; appropriate level of challenge and friction.",
        ),
        (
            "Hard",
            "Demanding; complex concepts requiring deliberate analysis.",
        ),
        (
            "Very hard",
            "Extremely difficult; conceptually dense or unfamiliar obstacles.",
        ),
    ],
};

use crate::diary_scales_more::{MOOD, MOTIVATION, SATISFACTION, SLEEP, STRESS, UNDERSTANDING};

const SCALES: [StandardScaleInfo; 13] = [
    PERFORMANCE,
    SUCCESS,
    CONFIDENCE,
    EFFORT,
    FOCUS,
    ENERGY,
    DIFFICULTY,
    SATISFACTION,
    UNDERSTANDING,
    MOOD,
    STRESS,
    MOTIVATION,
    SLEEP,
];

/// Retrieve scale metadata and wording by ID.
pub fn standard_scale_info(id: &str) -> Option<&'static StandardScaleInfo> {
    SCALES.iter().find(|s| s.id == id)
}

/// All 13 standard scale specifications in order.
pub fn all_standard_scale_infos() -> &'static [StandardScaleInfo] {
    &SCALES
}

/// Retrieve the label and description for a given scale and 1-based level.
pub fn scale_level_label(id: &str, level: u8) -> Option<(&'static str, &'static str)> {
    if !(1..=5).contains(&level) {
        return None;
    }
    let info = standard_scale_info(id)?;
    Some(info.levels[usize::from(level - 1)])
}

/// Descriptive tooltip for a scale button.
pub fn scale_level_tooltip(id: &str, level: u8) -> String {
    scale_level_label(id, level).map_or_else(
        || format!("Level {level}"),
        |(label, desc)| format!("{level} – {label}: {desc}"),
    )
}
