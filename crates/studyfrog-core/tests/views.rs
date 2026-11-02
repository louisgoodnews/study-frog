#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Recorded views: the stored words.

use studyfrog_core::views::{ViewKind, view_kind_from_name, view_kind_name};

#[test]
fn the_kinds_have_stable_words_that_read_back() {
    assert_eq!(view_kind_name(ViewKind::View), "view");
    assert_eq!(view_kind_name(ViewKind::Edit), "edit");
    for kind in [ViewKind::View, ViewKind::Edit] {
        assert_eq!(view_kind_from_name(view_kind_name(kind)), Some(kind));
    }
    assert_eq!(view_kind_from_name("answer"), None);
}
