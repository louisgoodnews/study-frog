//! Reading and writing learn modes for the editor (STORY-0198): the stored modes, one mode, and a save.
//! Each answers with the editor's own message.

use studyfrog_core::plan::plan_spaced;
use studyfrog_core::time::UnixMs;
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::learn_mode_store::{from_toml, list, read_text, save};
use studyfrog_service::learn_run;
use studyfrog_service::timezone::local_day_window;

use crate::exec::Worker;
use crate::learn_editor::{LearnEffect, LearnMsg};
use crate::msg::Msg;

/// Perform a learn-mode effect and answer with the editor's message.
pub fn run(w: &mut Worker, effect: LearnEffect, now_ns: i64) -> Msg {
    let Some(session) = &w.session else {
        return match effect {
            LearnEffect::Start(_) => {
                Msg::Learn(LearnMsg::Started(Err("no vault is open".to_owned())))
            }
            _ => Msg::Learn(LearnMsg::Listed(Err("no vault is open".to_owned()))),
        };
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
        LearnEffect::Start(file) => {
            let now = UnixMs(now_ns / 1_000_000);
            let started = fresh_id_source(u64::try_from(now.0).unwrap_or(0))
                .map_err(|e| e.to_string())
                .and_then(|ids| {
                    learn_run::start(
                        session,
                        &file,
                        plan_spaced(),
                        now,
                        local_day_window(now),
                        ids,
                    )
                    .map_err(|e| e.to_string())
                })
                .and_then(|run| {
                    let graph = read_text(root, &file).and_then(|text| from_toml(&text))?;
                    Ok((run, graph))
                });
            Msg::Learn(LearnMsg::Started(started))
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
