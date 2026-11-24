//! Classroom screen state, messages and effects as plain data (EPIC-068, TASK-0293). The reducer is in
//! `classroom_ui`, the worker side in `exec_classroom`, and the drawing in `view/classroom*`.

use studyfrog_agent::provider::Provider;
use studyfrog_core::classroom::{ClassroomSession, Owner, Participant, SourceRef};
use studyfrog_core::uid::Uid;
use studyfrog_service::classroom_task::Assignees;

/// What the Classroom screen remembers between frames.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClassroomState {
    /// The title typed for a new session.
    pub title: String,
    /// The stacks and PDFs the lecture is grounded in.
    pub sources: Vec<SourceRef>,
    /// How many simulated students join (the first presets, at most three).
    pub students: usize,
    /// The model connection chosen for this screen, by name.
    pub provider: Option<String>,
    /// The composer text: a question or comment for the teacher.
    pub input: String,
    /// The session on screen, once one is started.
    pub session: Option<ClassroomSession>,
    /// A model turn is running on the worker; controls wait for it.
    pub busy: bool,
    /// What went wrong last, shown above the controls.
    pub error: Option<String>,
    /// The assignment open in the task drawer, if one is.
    pub open_task: Option<Uid>,
    /// The answer being drafted for each assignment, not yet turned in.
    pub drafts: Vec<(Uid, String)>,
    /// The composer text for the open task's group thread.
    pub group_input: String,
}

/// A message of the Classroom screen.
#[derive(Debug, Clone, PartialEq)]
pub enum ClassroomMsg {
    /// The session title was edited.
    Title(String),
    /// A stack or PDF was ticked or unticked.
    ToggleSource(SourceRef),
    /// The number of simulated students was set.
    Students(usize),
    /// A model connection was chosen, by name.
    ChooseProvider(String),
    /// The composer text was edited.
    Input(String),
    /// Create a session from the form.
    Start,
    /// The teacher explains the next segment.
    Advance,
    /// The simulated student at this participant index asks a question.
    AskStudent(usize),
    /// The composer text is sent to the teacher as the user.
    Say,
    /// The teacher sets one task for the user.
    AssignTask,
    /// The teacher sets one group task for everyone: the students are split into groups of two.
    AssignGroups,
    /// Opens the task with this id in the drawer, or closes the drawer.
    OpenTask(Option<Uid>),
    /// The answer being drafted for the open task was edited.
    DraftEdit(String),
    /// The group composer text was edited.
    GroupInput(String),
    /// The group composer's message is sent to the open group task's thread.
    SendGroup,
    /// The drafted answer for the open task is turned in and locked.
    TurnIn,
    /// The teacher grades the user's turned-in answer to the open task.
    AskGrade,
    /// Back to the session form.
    Leave,
    /// A worker effect finished: the session as stored now, and what went wrong if anything.
    Done {
        /// The stored session after the effect (`None` when it could not be read).
        session: Option<ClassroomSession>,
        /// The reason the effect failed, if it did.
        error: Option<String>,
    },
}

/// Work the Classroom screen asks the worker to do. Every model turn goes through the service.
#[derive(Debug, Clone, PartialEq)]
pub enum ClassroomEffect {
    /// Create a session from the sources and participants. No model is called.
    Start {
        /// The session title.
        title: String,
        /// The stacks and PDFs to teach from.
        sources: Vec<SourceRef>,
        /// Teacher, user and simulated students, in that order.
        participants: Vec<Participant>,
        /// The model that teaches.
        provider: Provider,
    },
    /// The teacher explains the next segment.
    Advance {
        /// The session.
        id: Uid,
        /// The model.
        provider: Provider,
    },
    /// A simulated student asks a question and the teacher answers it.
    AskStudent {
        /// The session.
        id: Uid,
        /// The model.
        provider: Provider,
        /// Participant index of the student.
        student: usize,
    },
    /// The user says something and the teacher answers.
    Say {
        /// The session.
        id: Uid,
        /// The model.
        provider: Provider,
        /// Participant index of the user.
        user: usize,
        /// What the user said.
        text: String,
    },
    /// The teacher sets a task.
    AssignTask {
        /// The session.
        id: Uid,
        /// The model.
        provider: Provider,
        /// Who the task is for.
        assignees: Assignees,
    },
    /// The user writes to their group's thread and the simulated teammates answer.
    GroupMessage {
        /// The session.
        id: Uid,
        /// The model.
        provider: Provider,
        /// The group task.
        assignment: Uid,
        /// Index of the group in the task's rosters.
        group: usize,
        /// What the user wrote.
        text: String,
    },
    /// The user turns in an answer for their own work. No model is called.
    TurnIn {
        /// The session.
        id: Uid,
        /// The model, used only to build the lecture context; no call is made.
        provider: Provider,
        /// The task.
        assignment: Uid,
        /// Participant index of the user.
        submitter: usize,
        /// The answer as drafted.
        text: String,
    },
    /// The teacher grades a turned-in answer.
    Grade {
        /// The session.
        id: Uid,
        /// The model.
        provider: Provider,
        /// The task.
        assignment: Uid,
        /// Whose answer is graded.
        owner: Owner,
    },
}
