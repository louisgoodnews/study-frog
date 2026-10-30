#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Parsing: the spec examples, error locations and preservation of unknown keys.

use studyfrog_core::uid::Uid;
use studyfrog_md::doc::Doc;
use studyfrog_md::error::ParseErrorKind;
use studyfrog_md::error::ParseErrorKind::{
    InvalidValue, MissingFrontMatter, MissingKey, MissingSeparator, UnknownKind,
};
use studyfrog_md::parse::parse_doc;
use studyfrog_md::render::render_doc;
use studyfrog_md::ulid::format_ulid;
use studyfrog_md::validate::find_duplicate_ids;

type Check = fn(&ParseErrorKind) -> bool;
type Case = (String, Option<usize>, Check);

const ID: &str = "01JABCDEF0123456789ABCDEFG";

fn file(front: &str, body: &str) -> String {
    format!("---\nid: {ID}\n{front}---\n{body}")
}

#[test]
fn spec_card_example_parses() {
    let text = file(
        "kind: card\ntitle: Capital of France\n",
        "What is the capital of France?\n???\nParis",
    );
    let Ok(Doc::Card(card)) = parse_doc(&text) else {
        panic!("not a card")
    };
    assert_eq!(card.front, "What is the capital of France?");
    assert_eq!(card.back, "Paris");
    assert_eq!(format_ulid(card.meta.uid), ID);
    assert!(!card.reversible);
}

#[test]
fn errors_are_typed_and_located() {
    let cases: [Case; 8] = [
        ("no front matter".into(), None, |k| {
            matches!(k, MissingFrontMatter)
        }),
        ("---\nid: x\n---\n".into(), None, |k| {
            matches!(k, MissingKey("kind"))
        }),
        (file("kind: widget\ntitle: t\n", ""), Some(3), |k| {
            matches!(k, UnknownKind(_))
        }),
        (file("kind: note\n", ""), None, |k| {
            matches!(k, MissingKey("title"))
        }),
        (
            "---\nid: nope\nkind: note\ntitle: t\n---\n".into(),
            Some(2),
            |k| matches!(k, InvalidValue { key, .. } if key == "id"),
        ),
        (
            file("kind: note\ntitle: t\ncreated: yesterday\n", ""),
            Some(5),
            |k| matches!(k, InvalidValue { key, .. } if key == "created"),
        ),
        (file("kind: card\ntitle: t\n", "front only"), None, |k| {
            matches!(k, MissingSeparator)
        }),
        (
            file("kind: stack\ntitle: t\nsrs:\n  bogus: 1\n", ""),
            None,
            |k| matches!(k, ParseErrorKind::InvalidYaml(_)),
        ),
    ];
    for (text, line, check) in cases {
        let err = parse_doc(&text).unwrap_err();
        assert!(check(&err.kind), "{err:?}");
        if line.is_some() {
            assert_eq!(err.line, line, "{err:?}");
        }
    }
}

#[test]
fn foreign_and_unknown_keys_survive_a_rewrite() {
    let front = "kind: note\ntitle: Hello\n# keep me\nreversible: maybe\ncolor: [red, blue]\n";
    let Ok(Doc::Note(note)) = parse_doc(&file(front, "body\n")) else {
        panic!("not a note")
    };
    let keys: Vec<_> = note.extra.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(keys, ["reversible", "color"]);
    let again = render_doc(&Doc::Note(note.clone())).unwrap();
    assert!(
        again.contains("reversible: maybe\ncolor: [red, blue]\n"),
        "{again}"
    );
    assert_eq!(parse_doc(&again), Ok(Doc::Note(note)));
}

#[test]
fn defaults_are_omitted_when_rendering() {
    let Ok(doc) = parse_doc(&file(
        "kind: question\ntitle: Q\nmin_score: 0.7\n",
        "Why?\n???\nBecause",
    )) else {
        panic!()
    };
    let text = render_doc(&doc).unwrap();
    assert!(
        !text.contains("min_score") && !text.contains("tags"),
        "{text}"
    );
}

#[test]
fn duplicate_ids_are_reported_for_every_file() {
    let entries = [
        ("a.md", Uid(1)),
        ("b.md", Uid(2)),
        ("c.md", Uid(1)),
        ("d.md", Uid(1)),
    ];
    let dups = find_duplicate_ids(&entries);
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0].0, Uid(1));
    assert_eq!(dups[0].1, [&"a.md", &"c.md", &"d.md"]);
}

mod nasty {
    use studyfrog_core::meta::Meta;
    use studyfrog_core::uid::Uid;
    use studyfrog_md::doc::{CardDoc, Doc, NoteDoc};
    use studyfrog_md::error::RenderError;
    use studyfrog_md::parse::parse_doc;
    use studyfrog_md::render::render_doc;

    fn meta(title: &str, tags: &[&str]) -> Meta {
        Meta {
            uid: Uid(7),
            title: title.to_owned(),
            tags: tags.iter().map(|t| (*t).to_owned()).collect(),
            aliases: Vec::new(),
            created: None,
            updated: None,
        }
    }

    #[test]
    fn scalars_that_look_like_yaml_syntax_survive() {
        let titles = [
            "yes",
            "null",
            "~",
            "true",
            "1e3",
            "0x1F",
            "123",
            "- dash",
            ": colon",
            "a: b",
            "#hash",
            "  padded  ",
            "'quoted'",
            "\"dq\"",
            "---",
            "...",
            "key: [x]",
            "{a}",
            "é\u{1F438}",
            "a # b",
            "!tag",
            "&anchor",
            "*alias",
            "|",
            ">",
            "@at",
            "%pct",
            "`tick`",
        ];
        for title in titles {
            let doc = Doc::Note(NoteDoc {
                meta: meta(title, &[title]),
                rehearse_title: false,
                body: String::new(),
                extra: Vec::new(),
            });
            let text = render_doc(&doc).unwrap();
            assert_eq!(parse_doc(&text), Ok(doc), "title {title:?}\n{text}");
        }
    }

    #[test]
    fn separator_inside_front_is_detected_not_corrupted() {
        for front in ["a\n???\nb", "```\nopen fence", "~~~~\nx\n~~~"] {
            let doc = Doc::Card(CardDoc {
                meta: meta("t", &[]),
                reversible: false,
                front: front.into(),
                back: "b".into(),
                extra: Vec::new(),
            });
            assert_eq!(
                render_doc(&doc),
                Err(RenderError::AmbiguousSeparator),
                "{front:?}"
            );
        }
        let fine = Doc::Card(CardDoc {
            meta: meta("t", &[]),
            reversible: true,
            front: "```\n???\n```".into(),
            back: "???".into(),
            extra: Vec::new(),
        });
        assert_eq!(parse_doc(&render_doc(&fine).unwrap()), Ok(fine));
    }
}
