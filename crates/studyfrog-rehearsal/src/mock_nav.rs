//! Navigation through a mock exam: jumping, flagging for review, and the review overview (STORY-0185).
//! Pure state; the caller keeps the progress between screens.

/// Where the student is in the exam: the current question and per-question marks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExamProgress {
    /// Index of the question on screen.
    pub current: usize,
    /// Whether each question has an answer.
    pub answered: Vec<bool>,
    /// Whether each question is flagged for review.
    pub flagged: Vec<bool>,
}

/// What the review overview shows before submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewOverview {
    /// How many questions are answered.
    pub answered: usize,
    /// Indices of the unanswered questions.
    pub unanswered: Vec<usize>,
    /// Indices of the flagged questions.
    pub flagged: Vec<usize>,
}

/// Fresh progress for an exam with `count` questions: on the first, nothing answered or flagged.
pub fn new_progress(count: usize) -> ExamProgress {
    ExamProgress {
        current: 0,
        answered: vec![false; count],
        flagged: vec![false; count],
    }
}

/// A copy of the progress with the question at `index` on screen, or `None` when out of range.
pub fn jump_to(progress: &ExamProgress, index: usize) -> Option<ExamProgress> {
    (index < progress.answered.len()).then(|| ExamProgress {
        current: index,
        ..progress.clone()
    })
}

/// A copy with the question at `index` answered or not.
pub fn set_answered(progress: &ExamProgress, index: usize, answered: bool) -> Option<ExamProgress> {
    let mut next = progress.clone();
    *next.answered.get_mut(index)? = answered;
    Some(next)
}

/// A copy with the question at `index` flagged or unflagged.
pub fn toggle_flag(progress: &ExamProgress, index: usize) -> Option<ExamProgress> {
    let mut next = progress.clone();
    let flag = next.flagged.get_mut(index)?;
    *flag = !*flag;
    Some(next)
}

/// The next unanswered question after the current one, wrapping around to the start. `None` when
/// everything is answered.
pub fn next_unanswered(progress: &ExamProgress) -> Option<usize> {
    let n = progress.answered.len();
    (1..=n)
        .map(|step| (progress.current + step) % n.max(1))
        .find(|&i| !progress.answered[i])
}

/// The review overview: counts and the lists the student should look at before submitting.
pub fn review_overview(progress: &ExamProgress) -> ReviewOverview {
    ReviewOverview {
        answered: progress.answered.iter().filter(|&&a| a).count(),
        unanswered: indices_where(&progress.answered, false),
        flagged: indices_where(&progress.flagged, true),
    }
}

fn indices_where(flags: &[bool], want: bool) -> Vec<usize> {
    flags
        .iter()
        .enumerate()
        .filter(|(_, f)| **f == want)
        .map(|(i, _)| i)
        .collect()
}
