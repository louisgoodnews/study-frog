//! Calendar sessions, timetable and board effects.

use {
    studyfrog_board::ops::BoardOp, studyfrog_calendar::session::Session, studyfrog_core::uid::Uid,
};

/// Calendar sessions, timetable and board effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectPlanner {
    /// Read all sessions and today's local date.
    LoadSessions,
    /// Store a session; id `0` means a new one.
    SaveSession(Session),
    /// Delete a session.
    DeleteSession(Uid),
    /// Skip one occurrence of a session.
    CancelOccurrence(Uid, studyfrog_core::civil::Date),
    /// Read the board (creating the default one on first use).
    LoadBoard,
    /// Apply a board command and store the result.
    BoardCommand(BoardOp),
    /// Save a weekly recurring timetable entry.
    SaveTimetable(studyfrog_calendar::timetable::TimetableEntry),
    /// Delete a timetable entry.
    DeleteTimetable(Uid),
    /// Exam Scheduler effects (EPIC-076).
    ExamSched(crate::planner::exam_sched::ExamSchedEffect),
}
