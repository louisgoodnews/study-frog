#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The Console screen (STORY-0133): the transcript and history, the screen in the egui harness, and
//! its link to a headless instance (answers, served logs and lifecycle events come back as entries).

mod common;
mod control_common;

use common::views::{harness, open_model};
use control_common::*;
use egui_kittest::kittest::Queryable;
use studyfrog_ui::console_link::{link_new, poll, submit};
use studyfrog_ui::console_ui::{ConsoleMsg, ConsoleState, EntryKind, apply_console, entry_of};
use studyfrog_ui::control_events::bus_new;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

#[test]
fn lines_are_sent_kept_in_the_history_and_walked_with_up_and_down() {
    let mut c = ConsoleState::default();
    for line in ["state", "  ", "query type = card", "query type = card"] {
        apply_console(&mut c, ConsoleMsg::Edit(line.into()));
        apply_console(&mut c, ConsoleMsg::Submit);
    }
    assert_eq!(
        c.outbox,
        ["state", "query type = card", "query type = card"]
    );
    assert_eq!(
        c.history,
        ["state", "query type = card"],
        "blank and repeated lines are not kept twice"
    );
    apply_console(&mut c, ConsoleMsg::Older);
    assert_eq!(c.input, "query type = card");
    apply_console(&mut c, ConsoleMsg::Older);
    apply_console(&mut c, ConsoleMsg::Older);
    assert_eq!(c.input, "state");
    apply_console(&mut c, ConsoleMsg::Newer);
    assert_eq!(c.input, "query type = card");
    apply_console(&mut c, ConsoleMsg::Newer);
    assert_eq!(c.input, "", "past the newest is an empty line");
    apply_console(&mut c, ConsoleMsg::Clear);
    assert!(c.entries.is_empty());
}

#[test]
fn answers_errors_events_and_script_output_become_entries() {
    let ok = entry_of(r#"{"id":1,"ok":true,"result":{"screen":"home"}}"#);
    assert_eq!(ok.kind, EntryKind::Answer);
    assert!(ok.text.contains("\"screen\": \"home\""));
    let err = entry_of(r#"{"id":1,"ok":false,"error":"unknown command x"}"#);
    assert_eq!(
        (err.kind, err.text.as_str()),
        (EntryKind::Error, "unknown command x")
    );
    let ev = entry_of(
        r#"{"event":"lifecycle","data":{"kind":"item_created","at_ms":1,"data":{"title":"T"}}}"#,
    );
    assert_eq!(ev.kind, EntryKind::Event);
    assert!(ev.text.starts_with("item_created {"), "{}", ev.text);
    let script = entry_of(
        r#"{"event":"lifecycle","data":{"kind":"script_finished","data":{"job":"health","ok":false,"code":2,"stdout":"checked 3","stderr":"boom"}}}"#,
    );
    assert_eq!(script.kind, EntryKind::Script);
    assert_eq!(
        script.text,
        "script health failed (exit code 2)\nchecked 3\nboom"
    );
    assert_eq!(entry_of("started health").text, "started health");
}

#[test]
fn the_screen_shows_the_transcript_and_sends_what_is_typed() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Console));
    apply(
        &mut m,
        Msg::Console(ConsoleMsg::Line(
            r#"{"id":1,"ok":false,"error":"refused here"}"#.into(),
        )),
    );
    let mut h = harness(m);
    h.run_steps(4);
    assert!(
        h.query_all_by_label("Console").count() >= 2,
        "the heading and the sidebar entry"
    );
    assert!(h.query_by_label_contains("refused here").is_some());
    h.get_by_label("Console command").focus();
    h.get_by_label("Console command").type_text("nav reports");
    h.run();
    h.key_press(eframe::egui::Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().0.state.console.outbox, ["nav reports"]);
    assert_eq!(
        h.state().0.state.console.entries.last().unwrap().text,
        "nav reports"
    );
}

#[test]
fn the_link_brings_back_answers_logs_and_events_from_the_instance() {
    let (vault, logs) = (sample_vault("console-link"), temp_dir("console-link-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    let bus = bus_new();
    e.events = Some(bus.clone());
    let (inbox, jobs) = channel();
    let mut ctx = conn_ctx(inbox, &logs.path, || {});
    ctx.events = Some(bus);
    let link = link_new(Some(ctx), None);
    let serve = |e: &mut Engine| {
        if let Job::Run(req, reply) = jobs.recv().unwrap() {
            reply.send(handle(e, &req)).unwrap();
        }
    };
    submit(&link, "item create note Hello --stack french");
    serve(&mut e);
    submit(&link, "bogus words");
    submit(&link, "logs --tail 1 --level warn");
    let mut lines = Vec::new();
    for _ in 0..200 {
        lines.extend(poll(&link));
        if lines.len() >= 4 {
            break;
        }
        let _ = channel::<()>()
            .1
            .recv_timeout(std::time::Duration::from_millis(10));
    }
    let entries: Vec<_> = lines
        .into_iter()
        .map(|m| match m {
            Msg::Console(ConsoleMsg::Line(l)) => entry_of(&l),
            other => panic!("{other:?}"),
        })
        .collect();
    let kinds: Vec<EntryKind> = entries.iter().map(|x| x.kind).collect();
    assert!(kinds.contains(&EntryKind::Event), "{entries:?}");
    assert!(kinds.contains(&EntryKind::Answer), "{entries:?}");
    assert!(
        entries.iter().any(|x| x.text.contains("\"entries\"")),
        "logs served: {entries:?}"
    );
    assert!(
        entries
            .iter()
            .any(|x| x.kind == EntryKind::Error && x.text.contains("unknown command bogus")),
        "{entries:?}"
    );
}

#[cfg(unix)]
#[test]
fn script_run_and_script_list_go_to_the_runner_and_its_output_comes_back() {
    use studyfrog_ui::script_runner::{Runner, child_pause};
    let (home, logs) = (temp_dir("console-script"), temp_dir("console-script-logs"));
    let paths = studyfrog_config::paths::app_paths(&home.path);
    std::fs::create_dir_all(&paths.scripts).unwrap();
    let script = paths.scripts.join("hello.sh");
    std::fs::write(&script, "#!/bin/sh\necho hello from $STUDYFROG_JOB\n").unwrap();
    std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    let bus = bus_new();
    let (inbox, _jobs) = channel();
    let mut ctx = conn_ctx(inbox, &logs.path, || {});
    ctx.events = Some(bus.clone());
    let runner = Runner {
        paths,
        ctx: ctx.clone(),
        bus,
        running: Arc::default(),
        pause: child_pause,
    };
    let link = link_new(Some(ctx), Some(runner));
    submit(&link, "script run hello.sh");
    submit(&link, "script run nothing.sh");
    submit(&link, "script list");
    let mut texts = Vec::new();
    for _ in 0..300 {
        texts.extend(poll(&link).into_iter().map(|m| match m {
            Msg::Console(ConsoleMsg::Line(l)) => entry_of(&l).text,
            other => panic!("{other:?}"),
        }));
        if texts.iter().any(|t| t.contains("hello from hello.sh")) {
            break;
        }
        child_pause();
    }
    let all = texts.join("\n");
    assert!(all.contains("started hello.sh"), "{all}");
    assert!(all.contains("no job or script nothing.sh"), "{all}");
    assert!(
        all.contains("scripts.toml"),
        "the list names the file: {all}"
    );
    assert!(
        all.contains("script hello.sh finished\nhello from hello.sh"),
        "{all}"
    );
}
