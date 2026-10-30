#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `parse(render(x)) == x` for generated documents of every kind (TASK-0022).

use proptest::prelude::*;
use studyfrog_core::meta::Meta;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{CardDoc, Doc, NoteDoc, QuestionDoc, RawEntry, StackDoc};
use studyfrog_md::error::RenderError;
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;

const MAX_MS: i64 = 253_402_300_799_999;

/// Single-line, non-blank text as the schema requires for titles, tags and aliases.
fn line() -> impl Strategy<Value = String> {
    "\\PC{1,20}".prop_filter("blank or line-breaking", |s| {
        !s.trim().is_empty()
            && !s
                .chars()
                .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    })
}

fn free_text() -> impl Strategy<Value = String> {
    "(\\PC|\n){0,60}"
}

fn meta() -> impl Strategy<Value = Meta> {
    (
        any::<u128>(),
        line(),
        prop::collection::vec(line(), 0..4),
        prop::collection::vec(line(), 0..3),
        prop::option::of(0..=MAX_MS),
        prop::option::of(0..=MAX_MS),
    )
        .prop_map(|(uid, title, tags, aliases, created, updated)| Meta {
            uid: Uid(uid),
            title,
            tags,
            aliases,
            created: created.map(UnixMs),
            updated: updated.map(UnixMs),
        })
}

fn extras() -> impl Strategy<Value = Vec<RawEntry>> {
    prop::collection::btree_map("x_[a-z]{1,5}", "[a-z0-9]{1,6}", 0..3).prop_map(|m| {
        m.into_iter()
            .map(|(key, v)| RawEntry {
                text: format!("{key}: {v}\n"),
                key,
            })
            .collect()
    })
}

fn percent() -> impl Strategy<Value = f32> {
    (0_u32..=100).prop_map(|n| n as f32 / 100.0)
}

fn stack_doc() -> impl Strategy<Value = Doc> {
    (meta(), line(), free_text(), extras()).prop_map(|(meta, description, body, extra)| {
        Doc::Stack(StackDoc {
            meta,
            description,
            body,
            extra,
        })
    })
}

fn note_doc() -> impl Strategy<Value = Doc> {
    (meta(), any::<bool>(), free_text(), extras()).prop_map(
        |(meta, rehearse_title, body, extra)| {
            Doc::Note(NoteDoc {
                meta,
                rehearse_title,
                body,
                extra,
            })
        },
    )
}

fn card_doc() -> impl Strategy<Value = Doc> {
    (meta(), any::<bool>(), free_text(), free_text(), extras()).prop_map(
        |(meta, reversible, front, back, extra)| {
            Doc::Card(CardDoc {
                meta,
                reversible,
                front,
                back,
                extra,
            })
        },
    )
}

fn question_doc() -> impl Strategy<Value = Doc> {
    (
        meta(),
        free_text(),
        free_text(),
        prop::collection::vec(line(), 0..3),
        prop::collection::vec(line(), 0..3),
        percent(),
        extras(),
    )
        .prop_map(
            |(meta, prompt, reference, accepted, rubric, min_score, extra)| {
                Doc::Question(QuestionDoc {
                    meta,
                    prompt,
                    reference,
                    accepted,
                    rubric,
                    min_score,
                    extra,
                })
            },
        )
}

fn any_doc() -> impl Strategy<Value = Doc> {
    prop_oneof![stack_doc(), note_doc(), card_doc(), question_doc()]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn render_then_parse_is_identity(doc in any_doc()) {
        match render_doc(&doc) {
            Ok(text) => prop_assert_eq!(parse_doc(&text), Ok(doc), "{}", text),
            Err(RenderError::AmbiguousSeparator) => {}
            Err(e) => prop_assert!(false, "{e}"),
        }
    }

    #[test]
    fn rendering_is_stable(doc in any_doc()) {
        if let Ok(text) = render_doc(&doc) {
            let reparsed = parse_doc(&text).unwrap();
            prop_assert_eq!(render_doc(&reparsed).unwrap(), text);
        }
    }

    #[test]
    fn parser_never_panics_on_arbitrary_input(text in "(\\PC|\n){0,200}") {
        let _ = parse_doc(&text);
    }
}
