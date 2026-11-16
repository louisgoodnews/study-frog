//! SQLite storage for vault text embeddings and cosine similarity search.
//!
//! Complies with Invariant P1 (pure free functions) and P6 (<= 250 LOC).

use rusqlite::{Connection, params};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

/// A stored chunk embedding for an item.
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkEmbedding {
    /// Unique identifier of the source vault note or document.
    pub item_id: Uid,
    /// 0-based index of the chunk within the source item.
    pub chunk_idx: usize,
    /// Raw text chunk represented by the vector embedding.
    pub text: String,
    /// Dense float32 vector embedding.
    pub vector: Vec<f32>,
}

/// A similarity search match with cosine score.
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarityMatch {
    /// Unique identifier of the matching vault item.
    pub item_id: Uid,
    /// 0-based chunk index within the vault item.
    pub chunk_idx: usize,
    /// Raw text content of the chunk.
    pub text: String,
    /// Cosine similarity score in range [-1.0, 1.0].
    pub similarity: f32,
}

/// Ensure the `vault_embeddings` SQLite table exists.
pub fn ensure_rag_schema(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS vault_embeddings (
            item_id   TEXT NOT NULL,
            chunk_idx INTEGER NOT NULL,
            text      TEXT NOT NULL,
            vector    BLOB NOT NULL,
            PRIMARY KEY (item_id, chunk_idx)
        );",
    )
    .map_err(sql_err)?;
    Ok(())
}

/// Serialize a slice of f32 floats into a little-endian byte vector.
pub fn encode_vector_blob(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for &val in vector {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

/// Deserialize a little-endian byte vector into a vector of f32 floats.
pub fn decode_vector_blob(blob: &[u8]) -> Result<Vec<f32>, StoreError> {
    let (chunks, remainder) = blob.as_chunks::<4>();
    if !remainder.is_empty() {
        return Err(StoreError::Corrupt(format!(
            "invalid vector blob length: {} is not a multiple of 4",
            blob.len()
        )));
    }
    let mut out = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        out.push(f32::from_le_bytes(*chunk));
    }
    Ok(out)
}

/// Insert or replace a chunk embedding row for an item.
pub fn insert_chunk_embedding(
    conn: &Connection,
    item_id: &Uid,
    chunk_idx: usize,
    text: &str,
    vector: &[f32],
) -> Result<(), StoreError> {
    ensure_rag_schema(conn)?;
    let id_str = format_ulid(*item_id);
    let blob = encode_vector_blob(vector);
    conn.execute(
        "INSERT OR REPLACE INTO vault_embeddings (item_id, chunk_idx, text, vector)
         VALUES (?1, ?2, ?3, ?4)",
        params![id_str, chunk_idx as i64, text, blob],
    )
    .map_err(sql_err)?;
    Ok(())
}

/// Load all chunk embeddings from the `vault_embeddings` table.
pub fn load_chunk_embeddings(conn: &Connection) -> Result<Vec<ChunkEmbedding>, StoreError> {
    ensure_rag_schema(conn)?;
    let mut stmt = conn
        .prepare_cached(
            "SELECT item_id, chunk_idx, text, vector
             FROM vault_embeddings
             ORDER BY item_id, chunk_idx",
        )
        .map_err(sql_err)?;

    let rows = stmt
        .query_map([], |row| {
            let item_id_str: String = row.get(0)?;
            let chunk_idx: i64 = row.get(1)?;
            let text: String = row.get(2)?;
            let blob: Vec<u8> = row.get(3)?;
            Ok((item_id_str, chunk_idx, text, blob))
        })
        .map_err(sql_err)?;

    let mut embeddings = Vec::new();
    for row in rows {
        let (id_str, chunk_idx, text, blob) = row.map_err(sql_err)?;
        let item_id = parse_ulid(&id_str)
            .ok_or_else(|| StoreError::Corrupt(format!("malformed item_id ulid: {id_str}")))?;
        let vector = decode_vector_blob(&blob)?;
        embeddings.push(ChunkEmbedding {
            item_id,
            chunk_idx: chunk_idx as usize,
            text,
            vector,
        });
    }
    Ok(embeddings)
}

/// Delete all chunk embeddings belonging to a specific item.
pub fn delete_item_embeddings(conn: &Connection, item_id: &Uid) -> Result<(), StoreError> {
    ensure_rag_schema(conn)?;
    let id_str = format_ulid(*item_id);
    conn.execute("DELETE FROM vault_embeddings WHERE item_id = ?1", [&id_str])
        .map_err(sql_err)?;
    Ok(())
}

/// Compute cosine similarity between two float vectors.
///
/// Returns a score in `[-1.0, 1.0]`, or `0.0` if lengths differ or norms are zero.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom <= f32::EPSILON || !denom.is_finite() {
        return 0.0;
    }
    (dot / denom).clamp(-1.0, 1.0)
}

/// Rank chunk embeddings by descending cosine similarity against a query vector,
/// returning at most `top_k` matches.
pub fn rank_chunks_by_similarity(
    query_vector: &[f32],
    chunks: &[ChunkEmbedding],
    top_k: usize,
) -> Vec<SimilarityMatch> {
    if top_k == 0 || chunks.is_empty() {
        return Vec::new();
    }
    let mut matches: Vec<SimilarityMatch> = chunks
        .iter()
        .map(|c| SimilarityMatch {
            item_id: c.item_id,
            chunk_idx: c.chunk_idx,
            text: c.text.clone(),
            similarity: cosine_similarity(query_vector, &c.vector),
        })
        .collect();

    matches.sort_by(|a, b| b.similarity.total_cmp(&a.similarity));
    matches.truncate(top_k);
    matches
}
