//! Mock exam structure: sections of weighted questions (STORY-0185). Pure data and validation.

/// Points a question is worth, from 1 to 10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QuestionPoints(pub u8);

/// Most points one question may carry.
pub const MAX_QUESTION_POINTS: u8 = 10;

/// One question on a mock exam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExamQuestion {
    /// What is asked (one line).
    pub prompt: String,
    /// The course chapter or topic, for the breakdown in the grade report.
    pub topic: String,
    /// Points available.
    pub points: QuestionPoints,
}

/// A part of the exam, such as "Part A: short definitions".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExamSection {
    /// The section's heading.
    pub name: String,
    /// Its questions, in order.
    pub questions: Vec<ExamQuestion>,
}

/// A complete mock exam: its sections and the strict time limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockExam {
    /// Exam title.
    pub title: String,
    /// Total time in minutes.
    pub time_limit_min: u32,
    /// Sections, in order.
    pub sections: Vec<ExamSection>,
}

/// Shortest allowed exam, in minutes.
pub const MIN_TIME_LIMIT_MIN: u32 = 5;
/// Longest allowed exam, in minutes (six hours).
pub const MAX_TIME_LIMIT_MIN: u32 = 360;

/// Every question of the exam, in order, across sections.
pub fn all_questions(exam: &MockExam) -> Vec<&ExamQuestion> {
    exam.sections
        .iter()
        .flat_map(|s| s.questions.iter())
        .collect()
}

/// Total points the exam can award.
pub fn total_points(exam: &MockExam) -> u32 {
    all_questions(exam)
        .iter()
        .map(|q| u32::from(q.points.0))
        .sum()
}

/// Validate a mock exam before it is run.
pub fn validate_mock_exam(exam: &MockExam) -> Result<(), String> {
    if exam.title.trim().is_empty() || exam.title.chars().any(|c| c.is_control()) {
        return Err("the exam needs a title on one line".to_owned());
    }
    if !(MIN_TIME_LIMIT_MIN..=MAX_TIME_LIMIT_MIN).contains(&exam.time_limit_min) {
        return Err(format!(
            "the time limit must be between {MIN_TIME_LIMIT_MIN} and {MAX_TIME_LIMIT_MIN} minutes"
        ));
    }
    if exam.sections.is_empty() {
        return Err("an exam needs at least one section".to_owned());
    }
    for section in &exam.sections {
        if section.name.trim().is_empty() {
            return Err("every section needs a name".to_owned());
        }
        if section.questions.is_empty() {
            return Err(format!("section {:?} has no questions", section.name));
        }
        for q in &section.questions {
            if q.prompt.trim().is_empty() {
                return Err("every question needs a prompt".to_owned());
            }
            if q.points.0 == 0 || q.points.0 > MAX_QUESTION_POINTS {
                return Err(format!(
                    "a question carries between 1 and {MAX_QUESTION_POINTS} points"
                ));
            }
        }
    }
    Ok(())
}
