//! Words to commands: `nav home`, `item create card "Title" --text q --answer a`, `rehearse rate good`.

use studyfrog_core::event::Difficulty;
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;

use crate::control_cmd::{Command, ExportArgs, LogsQuery, RehearseCmd, SetupArgs};
use crate::control_words_items::{flag, item, need, split_flags, stack};
use crate::setup::Preset;
use crate::state::Screen;
use crate::transfer_types::Format;

/// The screen a word names, as the story and the sidebar call them.
pub fn screen_by_name(name: &str) -> Option<Screen> {
    Some(match name {
        "welcome" | "vaults" => Screen::Welcome,
        "home" => Screen::Home,
        "folders" => Screen::Folders,
        "items" | "library" | "stacks" => Screen::Library,
        "rehearse" | "rehearsal" => Screen::Rehearse,
        "calendar" => Screen::Calendar,
        "board" => Screen::Board,
        "reports" => Screen::Reports,
        "diary" => Screen::Diary,
        "achievements" => Screen::Achievements,
        "graph" => Screen::Graph,
        "assistant" | "agent" => Screen::Agent,
        "prompts" => Screen::Prompts,
        "settings" => Screen::Settings,
        "help" => Screen::Help,
        _ => return None,
    })
}

fn preset_named(word: &str) -> Result<Preset, String> {
    Ok(match word {
        "spaced" => Preset::Spaced,
        "all" | "in-order" => Preset::EverythingInOrder,
        "shuffled" => Preset::Shuffled,
        "hardest" => Preset::Hardest,
        "stalest" => Preset::Stalest,
        other => return Err(format!("unknown preset {other}")),
    })
}

fn setup(words: &[String]) -> Result<SetupArgs, String> {
    let (flags, _) = split_flags(words, &[])?;
    let mode = match flag(&flags, "mode") {
        None => None,
        Some("flip") => Some(RunMode::Flip),
        Some("answer") => Some(RunMode::Answer),
        Some(other) => return Err(format!("unknown mode {other} (flip, answer)")),
    };
    Ok(SetupArgs {
        preset: flag(&flags, "preset").map(preset_named).transpose()?,
        mode,
        limit: flag(&flags, "limit").map(str::to_owned),
        stacks: flags
            .iter()
            .filter(|(n, _)| *n == "stack")
            .map(|(_, v)| (*v).to_owned())
            .collect(),
    })
}

fn rating(word: &str) -> Result<Rating, String> {
    Ok(match word {
        "again" | "1" => Rating::Again,
        "hard" | "2" => Rating::Hard,
        "good" | "3" => Rating::Good,
        "easy" | "4" => Rating::Easy,
        other => return Err(format!("unknown rating {other} (again, hard, good, easy)")),
    })
}

fn mark(word: &str) -> Result<Difficulty, String> {
    Ok(match word {
        "easy" => Difficulty::Easy,
        "medium" => Difficulty::Medium,
        "difficult" | "hard" => Difficulty::Difficult,
        other => return Err(format!("unknown mark {other} (easy, medium, difficult)")),
    })
}

fn rehearse(words: &[String]) -> Result<RehearseCmd, String> {
    let arg = |what| {
        words
            .get(1)
            .map(String::as_str)
            .ok_or(format!("missing {what}"))
    };
    Ok(match words.first().map(String::as_str).unwrap_or("") {
        "setup" => RehearseCmd::Setup(setup(&words[1..])?),
        "start" => RehearseCmd::Start,
        "flip" | "reveal" => RehearseCmd::Flip,
        "answer" => RehearseCmd::Answer(words[1..].join(" ")),
        "rate" => RehearseCmd::Rate(rating(arg("rating")?)?),
        "mark" => RehearseCmd::Mark(mark(arg("mark")?)?),
        "skip" => RehearseCmd::Skip,
        "pause" => RehearseCmd::Pause,
        "resume" => RehearseCmd::Resume,
        "stop" | "abort" => RehearseCmd::Stop,
        "leave" => RehearseCmd::Leave,
        other => return Err(format!("unknown rehearse step {other:?}")),
    })
}

fn format_named(word: &str) -> Result<Format, String> {
    Ok(match word {
        "json" => Format::Json,
        "toml" => Format::Toml,
        "ron" => Format::Ron,
        "csv" => Format::Csv,
        "markdown" | "md" => Format::Markdown,
        other => return Err(format!("unknown format {other}")),
    })
}

/// Read the `logs` options.
pub fn logs_query(words: &[String]) -> Result<LogsQuery, String> {
    let (flags, _) = split_flags(words, &["follow", "json"])?;
    let tail = match flag(&flags, "tail") {
        Some(n) => n.parse().map_err(|_| format!("--tail {n}: not a number"))?,
        None => 50,
    };
    let level = flag(&flags, "level").map(str::to_owned);
    if let Some(l) = level
        .as_deref()
        .filter(|l| !matches!(*l, "info" | "warn" | "error"))
    {
        return Err(format!("unknown level {l} (info, warn, error)"));
    }
    let follow = flag(&flags, "follow").is_some();
    Ok(LogsQuery {
        tail,
        follow,
        level,
    })
}

fn two_words(words: &[String]) -> Result<Command, String> {
    let rest = &words[1..];
    let arg = |i: usize, what: &str| rest.get(i).cloned().ok_or(format!("missing {what}"));
    Ok(match words[0].as_str() {
        "nav" | "go" => match arg(0, "screen")?.as_str() {
            "back" => Command::Back,
            "forward" => Command::Forward,
            s => Command::Nav(screen_by_name(s).ok_or(format!("unknown screen {s}"))?),
        },
        "vault" => match (arg(0, "open or close")?.as_str(), rest.get(1)) {
            ("open", Some(path)) => Command::VaultOpen(path.clone()),
            ("close", _) => Command::VaultClose,
            _ => return Err("vault open <folder> | vault close".into()),
        },
        "item" => Command::Item(item(rest)?),
        "stack" => Command::Stack(stack(rest)?),
        "rehearse" => Command::Rehearse(rehearse(rest)?),
        "screenshot" => Command::Screenshot(arg(0, "PNG path")?.into()),
        "logs" => Command::Logs(logs_query(rest)?),
        "export" => {
            let (flags, pos) = split_flags(rest, &["scheduling"])?;
            Command::Export(ExportArgs {
                format: format_named(need(&pos, 0, "format")?)?,
                dest: need(&pos, 1, "destination folder")?.to_owned(),
                stack: flag(&flags, "stack").map(str::to_owned),
                scheduling: flag(&flags, "scheduling").is_some(),
            })
        }
        "import" => {
            let (flags, pos) = split_flags(rest, &[])?;
            let path = need(&pos, 0, "file to import")?.to_owned();
            Command::Import {
                path,
                stack: flag(&flags, "stack").map(str::to_owned),
            }
        }
        other => return Err(format!("unknown command {other} (try help)")),
    })
}

/// The command `words` stand for.
pub fn command_from_words(words: &[String]) -> Result<Command, String> {
    let first = words.first().map(String::as_str).ok_or("empty command")?;
    Ok(match first {
        "state" => Command::State,
        "help" | "commands" => Command::Help,
        "back" => Command::Back,
        "forward" => Command::Forward,
        "notices" | "notifications" => Command::Notices,
        "dismiss" => Command::Dismiss,
        "close" => Command::CloseModal,
        "settle" | "wait" => Command::Settle,
        _ => return two_words(words),
    })
}
