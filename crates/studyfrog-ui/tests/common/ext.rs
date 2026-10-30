#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Two small extensions defined in the tests, so the UI tests do not depend on the example crate.

use studyfrog_ext::registry::{register_card_type, register_importer, registry_new};
use studyfrog_ext::types::{CardType, Faces, ImportedCard, Importer, Registry};
use studyfrog_ui::host::extensions_of;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Selection;

use super::views::open_model;

pub fn comma_lines(text: &str) -> Result<Vec<ImportedCard>, String> {
    text.lines()
        .enumerate()
        .map(|(n, l)| {
            l.split_once(',')
                .map(|(f, b)| ImportedCard {
                    front: f.into(),
                    back: b.into(),
                    ..ImportedCard::default()
                })
                .ok_or(format!("line {}: no comma", n + 1))
        })
        .collect()
}

pub fn twice(source: &str) -> Result<Vec<Faces>, String> {
    if source.contains('!') {
        return Err("no exclamation marks".into());
    }
    Ok(vec![
        Faces {
            front: format!("{source}?"),
            back: "first".into(),
        },
        Faces {
            front: format!("{source}??"),
            back: "second".into(),
        },
    ])
}

pub fn registry() -> Registry {
    let mut r = registry_new();
    register_importer(
        &mut r,
        Importer {
            id: "commas",
            name: "Comma lines",
            extensions: &["csv"],
            import: comma_lines,
        },
    )
    .unwrap();
    register_card_type(
        &mut r,
        CardType {
            id: "twice",
            name: "Two cards",
            hint: "Write a word.",
            expand: twice,
        },
    )
    .unwrap();
    r
}

pub fn model_in_stack() -> studyfrog_ui::model::Model {
    let mut m = open_model();
    m.state.extensions = extensions_of(&registry());
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    m
}
