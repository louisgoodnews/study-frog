//! Mock exam messages, effects, and UI state data types (STORY-0186, STORY-0187, STORY-0207).

use studyfrog_agent::provider::Provider;
use studyfrog_rehearsal::curriculum_build::AnswerKey;
use studyfrog_rehearsal::exam_answer::ExamAnswer;
use studyfrog_rehearsal::exam_format::ExamFormatMix;
use studyfrog_rehearsal::mock_exam::MockExam;
use studyfrog_service::exam_gen::Generated;

use super::format_modal::FormatModalMsg;
use super::grading;

/// Messages of the mock exam.
#[derive(Debug, Clone, PartialEq)]
pub enum MockExamMsg {
    /// Validate stack and open the format selection modal.
    Open,
    /// List the lecture PDFs of the vault, to make an exam from one of them.
    OpenSlides,
    /// The lecture PDFs listed (or why they could not be).
    Slides(Result<Vec<SlideChoice>, String>),
    /// Open the format selection modal for a chosen lecture PDF.
    FromSlide(String),
    /// Action inside the format selection modal.
    FormatModal(FormatModalMsg),
    /// The exam was made (or why it was not).
    Generated(Result<Box<Generated>, String>),
    /// Save the exam as a note.
    Save,
    /// The note was saved (its path) or not (why).
    Saved(Result<String, String>),
    /// Show or hide the reference answers.
    ToggleAnswers,
    /// Start writing the answers.
    Start,
    /// Update a free-text answer.
    AnswerText(usize, String),
    /// Update one gap in a fill-in-the-gaps answer.
    AnswerGap(usize, usize, String),
    /// Choose an option in a single-select answer.
    AnswerSingle(usize, usize),
    /// Toggle an option in a multiple-select answer.
    AnswerMultiple(usize, usize, bool),
    /// Grade the answers written so far.
    Finish,
    /// Set the pass mark of the exam, in whole percent (STORY-0187).
    SetPassMark(f32),
    /// The answers were graded by the local model.
    Graded(Vec<grading::Graded>),
    /// Close the exam view.
    Close,
}

/// A lecture PDF of the vault that an exam can be made from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideChoice {
    /// Vault-relative path.
    pub path: String,
    /// Its title.
    pub title: String,
}

/// Where an exam's text comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExamSource {
    /// The notes of a folder (the inbox when `None`).
    Notes(Option<String>),
    /// The text of a lecture PDF, by its vault path.
    Slide(String),
}

/// Work for the executor.
#[derive(Debug, Clone, PartialEq)]
pub enum MockExamEffect {
    /// Make the exam from a source with the configured item format mix.
    Generate {
        /// Where the text comes from.
        source: ExamSource,
        /// Format mix requested by the student.
        mix: ExamFormatMix,
        /// The exam's title.
        title: String,
        /// The model that writes the questions, if one may see the text.
        provider: Option<Provider>,
    },
    /// List the lecture PDFs of the vault.
    ListSlides,
    /// Grade the answers with the local model (objective items are graded deterministically).
    Grade {
        /// The exam.
        exam: Box<MockExam>,
        /// Its reference answers and rubrics.
        keys: Vec<AnswerKey>,
        /// Structured student answers, one per question.
        answers: Vec<ExamAnswer>,
        /// The local model that judges open-ended answers.
        provider: Provider,
    },
    /// Save the exam as a note in its folder.
    Save {
        /// The folder to save into.
        folder: Option<String>,
        /// The exam's title.
        title: String,
        /// The exam.
        exam: Box<MockExam>,
        /// Its reference answers.
        keys: Vec<AnswerKey>,
    },
}

/// The exam view: the exam made, and whether its answers are shown.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MockExamUi {
    /// The folder the notes came from.
    pub folder: Option<String>,
    /// The exam's title.
    pub title: String,
    /// The exam is being written, or the PDFs are being listed.
    pub loading: bool,
    /// The answers are being graded by the local model.
    pub judging: bool,
    /// Format mix used for this exam.
    pub format_mix: ExamFormatMix,
    /// The exam, once made.
    pub result: Option<Box<Generated>>,
    /// Why there is no exam.
    pub error: Option<String>,
    /// What the student should know about how the exam was made.
    pub note: Option<String>,
    /// The lecture PDFs to choose from.
    pub slides: Option<Vec<SlideChoice>>,
    /// The reference answers are shown.
    pub show_answers: bool,
    /// The vault path of the saved note.
    pub saved: Option<String>,
    /// The student is writing the answers.
    pub started: bool,
    /// Structured answers, one per question.
    pub answers: Vec<ExamAnswer>,
    /// The graded questions, once the exam is finished.
    pub graded: Option<Vec<grading::Graded>>,
    /// When the exam was started, on the app clock in seconds.
    pub started_at: Option<f64>,
    /// Whole seconds of the exam's time used.
    pub elapsed_sec: u32,
    /// The answers are locked: submitted, and the result not yet shown.
    pub locked: bool,
    /// The pass mark in percent, when set; `None` is the default.
    pub pass_mark: Option<f32>,
}
