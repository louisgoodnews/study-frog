#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for virtual inline card indexing and lifecycle sync (TASK-0270).

use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};
use studyfrog_store::db::open_memory;
use studyfrog_store::inline_index::{
    derive_virtual_uid, ensure_inline_cards_table, extract_virtual_cards, load_inline_cards,
    sync_inline_cards,
};

#[test]
fn derives_deterministic_virtual_uid_and_valid_ulid() {
    let parent = Uid(0x018d_1234_5678_9abc_def0_1234_5678_9abc);
    let uid1 = derive_virtual_uid(&parent, 10, "Mitochondria");
    let uid2 = derive_virtual_uid(&parent, 10, "Mitochondria");
    assert_eq!(uid1, uid2, "Same inputs must produce identical virtual UID");

    // Different line number produces different UID
    let uid3 = derive_virtual_uid(&parent, 11, "Mitochondria");
    assert_ne!(uid1, uid3);

    // Different term produces different UID
    let uid4 = derive_virtual_uid(&parent, 10, "Ribosome");
    assert_ne!(uid1, uid4);

    // Different parent produces different UID
    let parent2 = Uid(0x018d_9999_5678_9abc_def0_1234_5678_9abc);
    let uid5 = derive_virtual_uid(&parent2, 10, "Mitochondria");
    assert_ne!(uid1, uid5);

    // Verify valid Crockford base32 ULID round-trip
    let text = format_ulid(uid1);
    assert_eq!(text.len(), 26);
    let roundtrip = parse_ulid(&text);
    assert_eq!(roundtrip, Some(uid1));
}

#[test]
fn extracts_virtual_cards_from_markdown() {
    let parent = Uid(42);
    let md = "# Notes\n\n- Mitochondria :: Powerhouse\n\n```\nSkip :: Code\n```\n";
    let cards = extract_virtual_cards(parent, md);

    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].parent_uid, parent);
    assert_eq!(cards[0].term, "Mitochondria");
    assert_eq!(cards[0].definition, "Powerhouse");
    assert_eq!(cards[0].line_number, 3);
    assert_eq!(
        cards[0].virtual_uid,
        derive_virtual_uid(&parent, 3, "Mitochondria")
    );
}

#[test]
fn extracts_reversed_virtual_cards() {
    let parent = Uid(42);
    let md = "ATP ::: Adenosine triphosphate\n";
    let cards = extract_virtual_cards(parent, md);

    assert_eq!(cards.len(), 2);
    assert_eq!(cards[0].term, "ATP");
    assert_eq!(cards[0].definition, "Adenosine triphosphate");
    assert_eq!(cards[0].line_number, 1);

    assert_eq!(cards[1].term, "Adenosine triphosphate");
    assert_eq!(cards[1].definition, "ATP");
    assert_eq!(cards[1].line_number, 1);
    assert_ne!(cards[0].virtual_uid, cards[1].virtual_uid);
}

#[test]
fn sync_and_load_lifecycle_with_edits_and_deletions() {
    let conn = open_memory().unwrap();
    ensure_inline_cards_table(&conn).unwrap();

    let parent_a = Uid(100);
    let parent_b = Uid(200);

    let doc_v1 = "Alpha :: First letter\nBeta :: Second letter\n";
    let cards_v1 = extract_virtual_cards(parent_a, doc_v1);
    assert_eq!(cards_v1.len(), 2);

    sync_inline_cards(&conn, parent_a, &cards_v1).unwrap();
    let loaded_a1 = load_inline_cards(&conn, parent_a).unwrap();
    assert_eq!(loaded_a1, cards_v1);

    // Edit definition of Alpha: virtual_uid must remain stable!
    let doc_v2 = "Alpha :: First letter of Greek alphabet\nBeta :: Second letter\n";
    let cards_v2 = extract_virtual_cards(parent_a, doc_v2);
    assert_eq!(cards_v2.len(), 2);
    assert_eq!(
        cards_v1[0].virtual_uid, cards_v2[0].virtual_uid,
        "Virtual UID should remain unchanged when definition is edited"
    );

    sync_inline_cards(&conn, parent_a, &cards_v2).unwrap();
    let loaded_a2 = load_inline_cards(&conn, parent_a).unwrap();
    assert_eq!(loaded_a2, cards_v2);
    assert_eq!(loaded_a2[0].definition, "First letter of Greek alphabet");

    // Isolate by parent: sync parent B
    let doc_b = "Gamma :: Third letter\n";
    let cards_b = extract_virtual_cards(parent_b, doc_b);
    sync_inline_cards(&conn, parent_b, &cards_b).unwrap();

    let loaded_b = load_inline_cards(&conn, parent_b).unwrap();
    assert_eq!(loaded_b, cards_b);

    // Parent A still has 2 cards
    let loaded_a_check = load_inline_cards(&conn, parent_a).unwrap();
    assert_eq!(loaded_a_check.len(), 2);

    // Delete one card from parent A note
    let doc_v3 = "Beta :: Second letter\n";
    let cards_v3 = extract_virtual_cards(parent_a, doc_v3);
    assert_eq!(cards_v3.len(), 1);

    sync_inline_cards(&conn, parent_a, &cards_v3).unwrap();
    let loaded_a3 = load_inline_cards(&conn, parent_a).unwrap();
    assert_eq!(loaded_a3.len(), 1);
    assert_eq!(loaded_a3[0].term, "Beta");

    // Syncing empty cards removes all for that parent
    sync_inline_cards(&conn, parent_a, &[]).unwrap();
    let loaded_a_empty = load_inline_cards(&conn, parent_a).unwrap();
    assert!(loaded_a_empty.is_empty());

    // Parent B remains unaffected
    let loaded_b_still = load_inline_cards(&conn, parent_b).unwrap();
    assert_eq!(loaded_b_still.len(), 1);
}
