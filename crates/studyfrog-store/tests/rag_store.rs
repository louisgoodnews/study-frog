#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for SQLite vault embeddings table and vector similarity search (TASK-0275).

use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::rag_store::{
    ChunkEmbedding, cosine_similarity, decode_vector_blob, delete_item_embeddings,
    encode_vector_blob, ensure_rag_schema, insert_chunk_embedding, load_chunk_embeddings,
    rank_chunks_by_similarity,
};

#[test]
fn test_vector_blob_roundtrip() {
    let empty: Vec<f32> = vec![];
    let encoded_empty = encode_vector_blob(&empty);
    assert_eq!(encoded_empty.len(), 0);
    assert_eq!(decode_vector_blob(&encoded_empty).unwrap(), empty);

    let original = vec![0.0, 1.25, -3.5, 42.125, -0.0001];
    let bytes = encode_vector_blob(&original);
    assert_eq!(bytes.len(), original.len() * 4);
    let decoded = decode_vector_blob(&bytes).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_decode_vector_blob_invalid_length() {
    let bad_bytes = vec![1, 2, 3];
    assert!(decode_vector_blob(&bad_bytes).is_err());

    let bad_bytes_5 = vec![1, 2, 3, 4, 5];
    assert!(decode_vector_blob(&bad_bytes_5).is_err());
}

#[test]
fn test_cosine_similarity() {
    // Identical normalized vectors
    let v1 = vec![1.0, 0.0, 0.0];
    assert!((cosine_similarity(&v1, &v1) - 1.0).abs() < 1e-6);

    // Orthogonal vectors
    let v2 = vec![0.0, 1.0, 0.0];
    assert!((cosine_similarity(&v1, &v2)).abs() < 1e-6);

    // Opposite vectors
    let v3 = vec![-1.0, 0.0, 0.0];
    assert!((cosine_similarity(&v1, &v3) - (-1.0)).abs() < 1e-6);

    // Arbitrary vectors
    let va = vec![1.0, 2.0, 3.0];
    let vb = vec![2.0, 4.0, 6.0]; // collinear
    assert!((cosine_similarity(&va, &vb) - 1.0).abs() < 1e-6);

    // Mismatched lengths
    assert_eq!(cosine_similarity(&v1, &[1.0, 0.0]), 0.0);

    // Zero norms
    let zero = vec![0.0, 0.0, 0.0];
    assert_eq!(cosine_similarity(&zero, &v1), 0.0);
    assert_eq!(cosine_similarity(&v1, &zero), 0.0);

    // Empty vectors
    assert_eq!(cosine_similarity(&[], &[]), 0.0);
}

#[test]
fn test_sqlite_embeddings_insert_and_load() {
    let conn = open_memory().unwrap();
    ensure_rag_schema(&conn).unwrap();
    // Idempotent schema call
    ensure_rag_schema(&conn).unwrap();

    let item1 = Uid(1001);
    let item2 = Uid(1002);

    let v1 = vec![0.1, 0.2, 0.3];
    let v2 = vec![0.4, 0.5, 0.6];
    let v3 = vec![0.7, 0.8, 0.9];

    insert_chunk_embedding(&conn, &item1, 0, "Item 1 Chunk 0", &v1).unwrap();
    insert_chunk_embedding(&conn, &item1, 1, "Item 1 Chunk 1", &v2).unwrap();
    insert_chunk_embedding(&conn, &item2, 0, "Item 2 Chunk 0", &v3).unwrap();

    let all = load_chunk_embeddings(&conn).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].item_id, item1);
    assert_eq!(all[0].chunk_idx, 0);
    assert_eq!(all[0].text, "Item 1 Chunk 0");
    assert_eq!(all[0].vector, v1);

    assert_eq!(all[1].item_id, item1);
    assert_eq!(all[1].chunk_idx, 1);
    assert_eq!(all[1].text, "Item 1 Chunk 1");
    assert_eq!(all[1].vector, v2);

    assert_eq!(all[2].item_id, item2);
    assert_eq!(all[2].chunk_idx, 0);
    assert_eq!(all[2].text, "Item 2 Chunk 0");
    assert_eq!(all[2].vector, v3);
}

#[test]
fn test_sqlite_embeddings_replace_and_delete() {
    let conn = open_memory().unwrap();
    ensure_rag_schema(&conn).unwrap();

    let item1 = Uid(1001);
    let item2 = Uid(1002);
    let v1 = vec![0.1, 0.2, 0.3];
    let v2 = vec![0.7, 0.8, 0.9];

    insert_chunk_embedding(&conn, &item1, 0, "Item 1 Chunk 0", &v1).unwrap();
    insert_chunk_embedding(&conn, &item2, 0, "Item 2 Chunk 0", &v2).unwrap();

    // Replace chunk
    let v1_updated = vec![0.9, 0.8, 0.7];
    insert_chunk_embedding(&conn, &item1, 0, "Item 1 Chunk 0 Updated", &v1_updated).unwrap();
    let updated_all = load_chunk_embeddings(&conn).unwrap();
    assert_eq!(updated_all.len(), 2);
    assert_eq!(updated_all[0].text, "Item 1 Chunk 0 Updated");
    assert_eq!(updated_all[0].vector, v1_updated);

    // Delete item1 embeddings
    delete_item_embeddings(&conn, &item1).unwrap();
    let remaining = load_chunk_embeddings(&conn).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].item_id, item2);
    assert_eq!(remaining[0].chunk_idx, 0);
}

#[test]
fn test_rank_chunks_by_similarity() {
    let item1 = Uid(1);
    let item2 = Uid(2);
    let item3 = Uid(3);

    let chunks = vec![
        ChunkEmbedding {
            item_id: item1,
            chunk_idx: 0,
            text: "Biology: Photosynthesis in plants".to_string(),
            vector: vec![1.0, 0.0, 0.0],
        },
        ChunkEmbedding {
            item_id: item2,
            chunk_idx: 0,
            text: "Physics: Quantum electrodynamics".to_string(),
            vector: vec![0.0, 1.0, 0.0],
        },
        ChunkEmbedding {
            item_id: item3,
            chunk_idx: 0,
            text: "Botany: Chlorophyll and sunlight".to_string(),
            vector: vec![0.8, 0.2, 0.0],
        },
    ];

    let query = vec![1.0, 0.1, 0.0];

    // top_k = 2
    let matches = rank_chunks_by_similarity(&query, &chunks, 2);
    assert_eq!(matches.len(), 2);
    // Closest should be item1 (1.0, 0.0, 0.0) or item3 (0.8, 0.2, 0.0)
    assert_eq!(matches[0].item_id, item1);
    assert_eq!(matches[1].item_id, item3);
    assert!(matches[0].similarity > matches[1].similarity);

    // top_k = 0 returns empty
    let empty_matches = rank_chunks_by_similarity(&query, &chunks, 0);
    assert_eq!(empty_matches.len(), 0);

    // top_k larger than chunks returns all chunks sorted
    let all_matches = rank_chunks_by_similarity(&query, &chunks, 10);
    assert_eq!(all_matches.len(), 3);
    assert_eq!(all_matches[0].item_id, item1);
    assert_eq!(all_matches[1].item_id, item3);
    assert_eq!(all_matches[2].item_id, item2);
}
