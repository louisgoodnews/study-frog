//! Reads one classroom session back from its rows, in the order it was written.

use rusqlite::{Connection, OptionalExtension, Params, Row, params};
use studyfrog_core::classroom::{
    Assignment, AssignmentKind, ChatTurn, ClassroomSession, GroupRoster, Participant, SourceKind,
    SourceRef,
};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

use crate::classroom_read_work::read_submissions;
use crate::error::{StoreError, sql_err};
use crate::runs::{corrupt, uid_of};

/// The session with this id, or `None` when no such session is stored.
pub(crate) fn read_session(
    conn: &Connection,
    id: Uid,
) -> Result<Option<ClassroomSession>, StoreError> {
    let key = format_ulid(id);
    let header: Option<(String, i64)> = conn
        .query_row(
            "SELECT title, created_at_ms FROM classroom_session WHERE uid = ?1",
            [&key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(sql_err)?;
    let Some((title, created)) = header else {
        return Ok(None);
    };
    Ok(Some(ClassroomSession {
        id,
        title,
        created_at: UnixMs(created),
        participants: read_participants(conn, &key)?,
        turns: read_turns(conn, &key)?,
        assignments: read_assignments(conn, &key)?,
        submissions: read_submissions(conn, &key)?,
        sources: read_sources(conn, &key)?,
    }))
}

/// Runs a query and collects the rows, mapping each with `map`.
pub(crate) fn query_all<P: Params, T>(
    conn: &Connection,
    sql: &str,
    p: P,
    map: impl FnMut(&Row<'_>) -> rusqlite::Result<T>,
) -> Result<Vec<T>, StoreError> {
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt.query_map(p, map).map_err(sql_err)?;
    rows.map(|r| r.map_err(sql_err)).collect()
}

/// A stored index, which must not be negative.
pub(crate) fn index_of(value: i64) -> Result<usize, StoreError> {
    usize::try_from(value).map_err(|_| corrupt("index", &value.to_string()))
}

fn participant_of(kind: &str, name: String) -> Result<Participant, StoreError> {
    match kind {
        "teacher" => Ok(Participant::Teacher { name }),
        "simulated_student" => Ok(Participant::SimulatedStudent { name }),
        "user" => Ok(Participant::User { name }),
        other => Err(corrupt("participant kind", other)),
    }
}

fn read_participants(conn: &Connection, key: &str) -> Result<Vec<Participant>, StoreError> {
    let rows = query_all(
        conn,
        "SELECT kind, name FROM classroom_participant WHERE session_uid = ?1 ORDER BY idx",
        [key],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )?;
    rows.into_iter()
        .map(|(kind, name)| participant_of(&kind, name))
        .collect()
}

fn source_of(kind: &str, item: &str) -> Result<SourceRef, StoreError> {
    let kind = match kind {
        "stack" => SourceKind::Stack,
        "note" => SourceKind::Note,
        "pdf" => SourceKind::Pdf,
        other => return Err(corrupt("source kind", other)),
    };
    Ok(SourceRef {
        kind,
        uid: uid_of(item)?,
    })
}

fn read_sources(conn: &Connection, key: &str) -> Result<Vec<SourceRef>, StoreError> {
    let rows = query_all(
        conn,
        "SELECT kind, item_uid FROM classroom_source WHERE session_uid = ?1 ORDER BY idx",
        [key],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )?;
    rows.into_iter()
        .map(|(kind, item)| source_of(&kind, &item))
        .collect()
}

fn read_turns(conn: &Connection, key: &str) -> Result<Vec<ChatTurn>, StoreError> {
    let rows = query_all(
        conn,
        "SELECT seq, speaker, at_ms, text FROM classroom_turn WHERE session_uid = ?1 ORDER BY seq",
        [key],
        |r| {
            Ok((
                r.get::<_, u32>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
            ))
        },
    )?;
    rows.into_iter()
        .map(|(seq, speaker, at, text)| {
            Ok(ChatTurn {
                seq,
                speaker: index_of(speaker)?,
                at: UnixMs(at),
                text,
            })
        })
        .collect()
}

fn read_assignments(conn: &Connection, key: &str) -> Result<Vec<Assignment>, StoreError> {
    let rows = query_all(
        conn,
        "SELECT uid, title, prompt, kind, assignee FROM classroom_assignment
         WHERE session_uid = ?1 ORDER BY seq",
        [key],
        |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<i64>>(4)?,
            ))
        },
    )?;
    rows.into_iter()
        .map(|(uid, title, prompt, kind, assignee)| {
            let kind = assignment_kind(conn, key, &uid, &kind, assignee)?;
            Ok(Assignment {
                id: uid_of(&uid)?,
                title,
                prompt,
                kind,
            })
        })
        .collect()
}

fn assignment_kind(
    conn: &Connection,
    key: &str,
    uid: &str,
    kind: &str,
    assignee: Option<i64>,
) -> Result<AssignmentKind, StoreError> {
    match (kind, assignee) {
        ("solo", Some(a)) => Ok(AssignmentKind::Solo {
            assignee: index_of(a)?,
        }),
        ("group", None) => Ok(AssignmentKind::Group {
            groups: read_groups(conn, key, uid)?,
        }),
        _ => Err(corrupt("assignment kind", kind)),
    }
}

fn read_groups(
    conn: &Connection,
    key: &str,
    assignment: &str,
) -> Result<Vec<GroupRoster>, StoreError> {
    let rows = query_all(
        conn,
        "SELECT idx, name FROM classroom_group WHERE session_uid = ?1 AND assignment_uid = ?2 ORDER BY idx",
        params![key, assignment],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
    )?;
    rows.into_iter()
        .map(|(idx, name)| {
            Ok(GroupRoster {
                name,
                members: read_members(conn, key, assignment, idx)?,
            })
        })
        .collect()
}

fn read_members(
    conn: &Connection,
    key: &str,
    assignment: &str,
    group: i64,
) -> Result<Vec<usize>, StoreError> {
    let rows = query_all(
        conn,
        "SELECT participant FROM classroom_group_member
         WHERE session_uid = ?1 AND assignment_uid = ?2 AND group_idx = ?3 ORDER BY seq",
        params![key, assignment, group],
        |r| r.get::<_, i64>(0),
    )?;
    rows.into_iter().map(index_of).collect()
}
