#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Property tests for the ULID, RFC 3339 and fence codecs.

use proptest::prelude::*;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::fence::{split_blocks, split_front_matter};
use studyfrog_md::rfc3339::{format_rfc3339, parse_rfc3339};
use studyfrog_md::ulid::{format_ulid, parse_ulid};

const MAX_MS: i64 = 253_402_300_799_999; // 9999-12-31T23:59:59.999Z

proptest! {
    #[test]
    fn ulid_round_trips(bits in any::<u128>()) {
        let text = format_ulid(Uid(bits));
        prop_assert_eq!(text.len(), 26);
        prop_assert_eq!(parse_ulid(&text), Some(Uid(bits)));
        prop_assert_eq!(parse_ulid(&text.to_lowercase()), Some(Uid(bits)));
    }

    #[test]
    fn ulid_rejects_wrong_lengths_and_overflow(s in "[0-9A-Z]{0,25}") {
        prop_assert_eq!(parse_ulid(&s), None);
        prop_assert_eq!(parse_ulid(&format!("8{s}0")), None);
    }

    #[test]
    fn rfc3339_round_trips(ms in 0_i64..=MAX_MS) {
        let text = format_rfc3339(UnixMs(ms)).unwrap();
        prop_assert_eq!(parse_rfc3339(&text), Some(UnixMs(ms)));
    }

    #[test]
    fn rfc3339_out_of_range_has_no_text(ms in (MAX_MS + 1)..i64::MAX / 2) {
        prop_assert_eq!(format_rfc3339(UnixMs(ms)), None);
    }

    #[test]
    fn fence_splits_any_body(yaml in "([a-z]{1,5}: [a-z0-9]{0,5}\n){0,5}", body in "\\PC{0,80}") {
        let text = format!("---\n{yaml}---\n{body}");
        let split = split_front_matter(&text).unwrap();
        prop_assert_eq!((split.yaml, split.body), (yaml.as_str(), body.as_str()));
    }

    #[test]
    fn blocks_cover_the_yaml_exactly(pairs in prop::collection::vec(("[a-z]{1,6}", "[a-z0-9]{0,6}"), 0..8)) {
        let yaml: String = pairs.iter().map(|(k, v)| format!("{k}: {v}\n")).collect();
        let blocks = split_blocks(&yaml);
        let joined: String = blocks.iter().map(|b| b.text).collect();
        prop_assert_eq!(joined.as_str(), yaml.as_str());
        prop_assert_eq!(blocks.len(), pairs.len());
    }
}

#[test]
fn rfc3339_known_values_and_offsets() {
    assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z"), Some(UnixMs(0)));
    assert_eq!(
        parse_rfc3339("2026-10-30T12:00:00.5+02:00"),
        Some(UnixMs(1_793_354_400_500))
    );
    assert!(parse_rfc3339("2024-02-29T00:00:00Z").is_some());
    for bad in [
        "2023-02-29T00:00:00Z",
        "2026-13-01T00:00:00Z",
        "2026-10-30T24:00:00Z",
        "2026-10-30",
        "2026-10-30T00:00:00",
    ] {
        assert_eq!(parse_rfc3339(bad), None, "{bad}");
    }
}

#[test]
fn fence_edge_cases() {
    let crlf = "---\r\nid: x\r\n---\r\nbody";
    assert_eq!(split_front_matter(crlf).map(|s| s.body), Some("body"));
    assert_eq!(
        split_front_matter("---\nid: x\n---").map(|s| s.body),
        Some("")
    );
    assert_eq!(
        split_front_matter("\u{feff}---\na: 1\n---\nb").map(|s| s.body),
        Some("b")
    );
    assert_eq!(split_front_matter("no front matter"), None);
    assert_eq!(split_front_matter("---\nunclosed: 1\n"), None);
    let blocks = split_blocks("# c\nid: 1\ntags:\n  - a\n- b\n\n# note\ntitle: \"x: y\"\n");
    assert_eq!(
        blocks.iter().map(|b| b.key).collect::<Vec<_>>(),
        ["id", "tags", "title"]
    );
    assert_eq!(blocks[1].text, "tags:\n  - a\n- b\n\n# note\n");
    assert_eq!(blocks[2].line, 8);
}
