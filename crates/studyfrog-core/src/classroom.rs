//! Classroom simulation as plain data (EPIC-068). A session is a lecture chat between a teacher,
//! simulated students and the user, with assignments and graded submissions. Behaviour and
//! invariants live in `classroom_ops` and `classroom_work`; this file carries only the shape.

use crate::time::UnixMs;
use crate::uid::Uid;

/// Who takes part in a session. Participants are addressed by their index in
/// [`ClassroomSession::participants`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Participant {
    /// The AI teacher: gives the lecture, sets and grades work.
    Teacher {
        /// Display name.
        name: String,
    },
    /// A simulated student: chats and may be assigned work.
    SimulatedStudent {
        /// Display name.
        name: String,
    },
    /// The person using the app: chats and may be assigned work.
    User {
        /// Display name.
        name: String,
    },
}

/// One message in the lecture chat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTurn {
    /// Position in the chat, starting at 0.
    pub seq: u32,
    /// Index of the speaker in [`ClassroomSession::participants`].
    pub speaker: usize,
    /// When the turn was made.
    pub at: UnixMs,
    /// What was said.
    pub text: String,
}

/// Members of a group assignment, by participant index. Never fewer than two, never a teacher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupRoster {
    /// Name shown for the group, e.g. "Group 2".
    pub name: String,
    /// Participant indices of the members.
    pub members: Vec<usize>,
}

/// Who an assignment is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentKind {
    /// One participant works alone.
    Solo {
        /// Participant index of the student who does the work.
        assignee: usize,
    },
    /// Each roster hands in one shared submission.
    Group {
        /// The rosters; a submission's [`Owner::Group`] indexes this list.
        groups: Vec<GroupRoster>,
    },
}

/// A task set by the teacher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// Stable identity, unique within the session.
    pub id: Uid,
    /// Short name shown in lists.
    pub title: String,
    /// The task text the students see.
    pub prompt: String,
    /// Solo or group.
    pub kind: AssignmentKind,
}

/// Who a submission belongs to: a participant for solo work, a group index for group work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// Solo work, by participant index.
    Participant(usize),
    /// Group work, by index into the assignment's rosters.
    Group(usize),
}

/// The grade of one submission, out of `max` points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradeReport {
    /// Points awarded, at most `max`.
    pub points: u32,
    /// Points the task is worth; never zero.
    pub max: u32,
    /// The teacher's comment; may be empty.
    pub feedback: String,
    /// When the grade was recorded.
    pub graded_at: UnixMs,
}

/// A turned-in answer. It exists only after turn-in, so it is locked from the start; grading
/// fills `grade` once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submission {
    /// The assignment this answers.
    pub assignment: Uid,
    /// Whose answer this is.
    pub owner: Owner,
    /// The answer as turned in; never changes afterwards.
    pub text: String,
    /// When the answer was turned in; the answer is locked from this moment.
    pub submitted_at: UnixMs,
    /// Set once, when the teacher grades it.
    pub grade: Option<GradeReport>,
}

/// One classroom session, from creation to its last turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassroomSession {
    /// Stable identity.
    pub id: Uid,
    /// Shown in the session browser.
    pub title: String,
    /// When the session was created.
    pub created_at: UnixMs,
    /// The teacher, the simulated students and the user; indices point here.
    pub participants: Vec<Participant>,
    /// The lecture chat, in order.
    pub turns: Vec<ChatTurn>,
    /// Tasks set by the teacher.
    pub assignments: Vec<Assignment>,
    /// Turned-in answers, at most one per owner and assignment.
    pub submissions: Vec<Submission>,
}

/// Why a classroom operation was refused. A refused operation leaves the session unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassroomError {
    /// A session needs one teacher to give the lecture.
    NoTeacher,
    /// Text that must not be empty (title, prompt, answer, feedback, name) was blank.
    EmptyText,
    /// No participant has this index.
    UnknownParticipant(usize),
    /// Only a simulated student or the user can be assigned or submit work.
    NotAStudent(usize),
    /// A turn is earlier than the one before it.
    TurnOutOfOrder,
    /// No assignment has this id.
    UnknownAssignment(Uid),
    /// An assignment with this id already exists.
    DuplicateAssignment(Uid),
    /// A group has fewer than two members, or a group assignment has no groups.
    GroupTooSmall,
    /// A participant appears twice in one group.
    DuplicateMember(usize),
    /// A group assignment has no group with this index.
    UnknownGroup(usize),
    /// The submitter is not the assignee or a member of the group.
    NotAssigned(usize),
    /// The owner (participant or group) does not match the assignment's kind.
    WrongOwner,
    /// This owner has already turned in: the answer is locked.
    AlreadySubmitted,
    /// Grading was requested before the owner turned in.
    NotSubmitted,
    /// The submission already has a grade; grades are recorded once.
    AlreadyGraded,
    /// Points are more than the maximum, or the maximum is zero.
    GradeOutOfRange {
        /// Points given.
        points: u32,
        /// Maximum given.
        max: u32,
    },
}
