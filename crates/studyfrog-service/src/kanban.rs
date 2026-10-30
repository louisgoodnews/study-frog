//! The kanban board: apply a pure command, store the result.

use studyfrog_board::model::{Board, board_default, describe_error};
use studyfrog_board::ops::{BoardOp, apply};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::compose_ulid;
use studyfrog_store::board_store::{load_board, save_board};

use crate::create::IdSource;
use crate::session::{ServiceError, VaultSession};

/// An id derived from the source; `salt` makes several ids from one source differ.
fn derive(ids: IdSource, salt: u8) -> Uid {
    let mut random = ids.random;
    random[9] ^= salt;
    compose_ulid(ids.now_ms, random)
}

/// The stored board; the default one is created and stored on first use.
pub fn board_or_default(session: &mut VaultSession, ids: IdSource) -> Result<Board, ServiceError> {
    if let Some(b) = load_board(&session.conn).map_err(ServiceError::Store)? {
        return Ok(b);
    }
    let b = board_default([
        derive(ids, 1),
        derive(ids, 2),
        derive(ids, 3),
        derive(ids, 4),
    ]);
    save_board(&mut session.conn, &b).map_err(ServiceError::Store)?;
    Ok(b)
}

/// Apply a command to the stored board. A refused command changes nothing and says why.
pub fn apply_board_op(
    session: &mut VaultSession,
    op: &BoardOp,
    ids: IdSource,
) -> Result<Board, ServiceError> {
    let board = board_or_default(session, ids)?;
    let next = apply(&board, op, derive(ids, 5))
        .map_err(|e| ServiceError::Invalid(describe_error(&board, &e)))?;
    save_board(&mut session.conn, &next).map_err(ServiceError::Store)?;
    Ok(next)
}
