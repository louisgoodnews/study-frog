//! Writes one whole classroom session as rows. Called inside a transaction by `classroom_store`.

use rusqlite::{Connection, params};
use studyfrog_core::classroom::{
    Assignment, AssignmentKind, ChatTurn, ClassroomSession, GroupRoster, Owner, Participant,
    SourceKind, SourceRef, Submission,
};
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};

/// Inserts the header and every child row of a session. The session row must not exist yet.
pub(crate) fn write_session(conn: &Connection, s: &ClassroomSession) -> Result<(), StoreError> {
    let uid = format_ulid(s.id);
    conn.execute(
        "INSERT INTO classroom_session (uid, title, created_at_ms) VALUES (?1, ?2, ?3)",
        params![uid, s.title, s.created_at.0],
    )
    .map_err(sql_err)?;
    write_participants(conn, &uid, &s.participants)?;
    write_turns(conn, &uid, &s.turns)?;
    write_assignments(conn, &uid, &s.assignments)?;
    write_submissions(conn, &uid, &s.submissions)?;
    write_sources(conn, &uid, &s.sources)
}

fn write_sources(conn: &Connection, uid: &str, sources: &[SourceRef]) -> Result<(), StoreError> {
    for (idx, source) in sources.iter().enumerate() {
        conn.execute(
            "INSERT INTO classroom_source (session_uid, idx, kind, item_uid) VALUES (?1, ?2, ?3, ?4)",
            params![uid, db_int(idx), source_kind_name(source.kind), format_ulid(source.uid)],
        )
        .map_err(sql_err)?;
    }
    Ok(())
}

/// The name a source kind is stored under.
fn source_kind_name(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::Stack => "stack",
        SourceKind::Note => "note",
        SourceKind::Pdf => "pdf",
    }
}

/// A count or index as the database stores it.
pub(crate) fn db_int(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// The kind name and display name of a participant.
fn participant_parts(p: &Participant) -> (&'static str, &str) {
    match p {
        Participant::Teacher { name } => ("teacher", name),
        Participant::SimulatedStudent { name } => ("simulated_student", name),
        Participant::User { name } => ("user", name),
    }
}

fn write_participants(
    conn: &Connection,
    uid: &str,
    participants: &[Participant],
) -> Result<(), StoreError> {
    for (idx, p) in participants.iter().enumerate() {
        let (kind, name) = participant_parts(p);
        conn.execute(
            "INSERT INTO classroom_participant (session_uid, idx, kind, name) VALUES (?1, ?2, ?3, ?4)",
            params![uid, db_int(idx), kind, name],
        )
        .map_err(sql_err)?;
    }
    Ok(())
}

fn write_turns(conn: &Connection, uid: &str, turns: &[ChatTurn]) -> Result<(), StoreError> {
    for t in turns {
        conn.execute(
            "INSERT INTO classroom_turn (session_uid, seq, speaker, at_ms, text) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![uid, t.seq, db_int(t.speaker), t.at.0, t.text],
        )
        .map_err(sql_err)?;
    }
    Ok(())
}

fn write_assignments(
    conn: &Connection,
    uid: &str,
    assignments: &[Assignment],
) -> Result<(), StoreError> {
    for (seq, a) in assignments.iter().enumerate() {
        let (kind, assignee) = match &a.kind {
            AssignmentKind::Solo { assignee } => ("solo", Some(db_int(*assignee))),
            AssignmentKind::Group { .. } => ("group", None),
        };
        let a_uid = format_ulid(a.id);
        conn.execute(
            "INSERT INTO classroom_assignment (session_uid, uid, seq, title, prompt, kind, assignee)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![uid, a_uid, db_int(seq), a.title, a.prompt, kind, assignee],
        )
        .map_err(sql_err)?;
        if let AssignmentKind::Group { groups } = &a.kind {
            write_groups(conn, uid, &a_uid, groups)?;
        }
    }
    Ok(())
}

fn write_groups(
    conn: &Connection,
    uid: &str,
    assignment: &str,
    groups: &[GroupRoster],
) -> Result<(), StoreError> {
    for (idx, g) in groups.iter().enumerate() {
        conn.execute(
            "INSERT INTO classroom_group (session_uid, assignment_uid, idx, name) VALUES (?1, ?2, ?3, ?4)",
            params![uid, assignment, db_int(idx), g.name],
        )
        .map_err(sql_err)?;
        for (seq, &member) in g.members.iter().enumerate() {
            conn.execute(
                "INSERT INTO classroom_group_member (session_uid, assignment_uid, group_idx, seq, participant)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![uid, assignment, db_int(idx), db_int(seq), db_int(member)],
            )
            .map_err(sql_err)?;
        }
    }
    Ok(())
}

/// The owner kind name and index as stored.
pub(crate) fn owner_parts(owner: Owner) -> (&'static str, usize) {
    match owner {
        Owner::Participant(p) => ("participant", p),
        Owner::Group(g) => ("group", g),
    }
}

fn write_submissions(
    conn: &Connection,
    uid: &str,
    submissions: &[Submission],
) -> Result<(), StoreError> {
    for (seq, s) in submissions.iter().enumerate() {
        let a_uid = format_ulid(s.assignment);
        let (kind, idx) = owner_parts(s.owner);
        conn.execute(
            "INSERT INTO classroom_submission (session_uid, seq, assignment_uid, owner_kind, owner_idx, text, submitted_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![uid, db_int(seq), a_uid, kind, db_int(idx), s.text, s.submitted_at.0],
        )
        .map_err(sql_err)?;
        if let Some(g) = &s.grade {
            conn.execute(
                "INSERT INTO classroom_grade (session_uid, assignment_uid, owner_kind, owner_idx, points, max, feedback, graded_at_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![uid, a_uid, kind, db_int(idx), g.points, g.max, g.feedback, g.graded_at.0],
            )
            .map_err(sql_err)?;
        }
    }
    Ok(())
}
