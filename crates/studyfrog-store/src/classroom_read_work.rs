//! Reads the submissions and grades of a classroom session (the read side of `classroom_write`).

use rusqlite::Connection;
use studyfrog_core::classroom::{CriterionMark, GradeReport, Owner, Submission};
use studyfrog_core::time::UnixMs;

use crate::classroom_read::{index_of, query_all};
use crate::error::StoreError;
use crate::runs::{corrupt, uid_of};

/// The owner from its stored kind and index.
fn owner_of(kind: &str, idx: i64) -> Result<Owner, StoreError> {
    let idx = index_of(idx)?;
    match kind {
        "participant" => Ok(Owner::Participant(idx)),
        "group" => Ok(Owner::Group(idx)),
        other => Err(corrupt("owner kind", other)),
    }
}

/// One submission row joined with its grade, if it has one. Every grade column is set or none is.
type RawSubmission = (
    String,
    i64,
    String,
    i64,
    String,
    i64,
    Option<u32>,
    Option<u32>,
    Option<String>,
    Option<i64>,
);

pub(crate) fn read_submissions(
    conn: &Connection,
    key: &str,
) -> Result<Vec<Submission>, StoreError> {
    let rows = query_all(
        conn,
        "SELECT s.assignment_uid, s.seq, s.owner_kind, s.owner_idx, s.text, s.submitted_at_ms,
                g.points, g.max, g.feedback, g.graded_at_ms
         FROM classroom_submission s
         LEFT JOIN classroom_grade g
           ON g.session_uid = s.session_uid AND g.assignment_uid = s.assignment_uid
          AND g.owner_kind = s.owner_kind AND g.owner_idx = s.owner_idx
         WHERE s.session_uid = ?1 ORDER BY s.seq",
        [key],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
                r.get(7)?,
                r.get(8)?,
                r.get(9)?,
            ))
        },
    )?;
    let mut submissions = rows
        .into_iter()
        .map(submission_of)
        .collect::<Result<Vec<_>, _>>()?;
    attach_marks(conn, key, &mut submissions)?;
    Ok(submissions)
}

fn submission_of(row: RawSubmission) -> Result<Submission, StoreError> {
    let (assignment, _seq, kind, idx, text, at, points, max, feedback, graded_at) = row;
    Ok(Submission {
        assignment: uid_of(&assignment)?,
        owner: owner_of(&kind, idx)?,
        text,
        submitted_at: UnixMs(at),
        grade: grade_of(points, max, feedback, graded_at, Vec::new()),
    })
}

/// A rubric mark row: the assignment and owner it belongs to, and the mark.
type RawMark = (String, String, i64, String, bool, bool, String);

/// Fills in the rubric marks of every graded submission, in rubric order.
fn attach_marks(
    conn: &Connection,
    key: &str,
    submissions: &mut [Submission],
) -> Result<(), StoreError> {
    let rows: Vec<RawMark> = query_all(
        conn,
        "SELECT assignment_uid, owner_kind, owner_idx, criterion, met, checked, feedback
         FROM classroom_grade_criterion WHERE session_uid = ?1 ORDER BY seq",
        [key],
        |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        },
    )?;
    for (assignment, kind, idx, criterion, met, checked, feedback) in rows {
        let (assignment, owner) = (uid_of(&assignment)?, owner_of(&kind, idx)?);
        let mark = CriterionMark {
            criterion,
            met,
            checked,
            feedback,
        };
        if let Some(grade) = submissions
            .iter_mut()
            .find(|s| s.assignment == assignment && s.owner == owner)
            .and_then(|s| s.grade.as_mut())
        {
            grade.criteria.push(mark);
        }
    }
    Ok(())
}

fn grade_of(
    points: Option<u32>,
    max: Option<u32>,
    feedback: Option<String>,
    graded_at: Option<i64>,
    criteria: Vec<CriterionMark>,
) -> Option<GradeReport> {
    match (points, max, feedback, graded_at) {
        (Some(points), Some(max), Some(feedback), Some(at)) => Some(GradeReport {
            points,
            max,
            feedback,
            graded_at: UnixMs(at),
            criteria,
        }),
        _ => None,
    }
}
