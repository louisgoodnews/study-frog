//! Reading and writing learn modes for the editor (STORY-0198): the stored modes, one mode, and a save.
//! Each answers with the editor's own message.

use studyfrog_service::learn_mode_store::{from_toml, list, read_text, save};

use crate::exec::Worker;
use crate::learn_editor::{LearnEffect, LearnMsg};
use crate::msg::Msg;

/// Perform a learn-mode effect and answer with the editor's message.
pub fn run(w: &mut Worker, effect: LearnEffect) -> Msg {
    let Some(session) = &w.session else {
        return Msg::Learn(LearnMsg::Listed(Err("no vault is open".to_owned())));
    };
    let root = session.root.as_path();
    match effect {
        LearnEffect::List => Msg::Learn(LearnMsg::Listed(Ok(list(root)))),
        LearnEffect::Load(file) => {
            let loaded = read_text(root, &file)
                .and_then(|text| from_toml(&text))
                .map(|graph| (file, graph));
            Msg::Learn(LearnMsg::Loaded(loaded))
        }
        LearnEffect::Save(graph) => {
            let saved = save(root, &graph).map(|path| {
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            });
            Msg::Learn(LearnMsg::Saved(saved))
        }
    }
}
