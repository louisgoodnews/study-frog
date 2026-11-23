//! The words of `achievements`, `graph`, `assistant`, `settings`, `match` and `folders`.

use crate::control_cmd::Command;
use crate::control_cmd::screens::{
    AchFilter, AchievementsCmd, AssistantCmd, FoldersCmd, MatchCmd, ScreenCmd, SettingsCmd,
};

fn achievements(rest: &[String]) -> Result<AchievementsCmd, String> {
    match rest.first().map(String::as_str) {
        None | Some("list") => Ok(AchievementsCmd::List),
        Some("filter") => Ok(AchievementsCmd::Filter(
            match rest.get(1).map(String::as_str) {
                Some("all") => AchFilter::All,
                Some("global") => AchFilter::Global,
                Some("stack") => AchFilter::Stack(stack_folder(rest, 2)?),
                _ => return Err("achievements filter all|global|stack FOLDER".into()),
            },
        )),
        Some(other) => Err(format!("achievements has list and filter (not {other})")),
    }
}

fn stack_folder(rest: &[String], at: usize) -> Result<String, String> {
    rest.get(at)
        .cloned()
        .ok_or_else(|| "missing stack folder".to_owned())
}

fn assistant(rest: &[String]) -> Result<AssistantCmd, String> {
    match rest.first().map(String::as_str) {
        None | Some("state") => Ok(AssistantCmd::State),
        Some("send") => {
            let text = rest[1..].join(" ");
            if text.trim().is_empty() {
                return Err("assistant send needs the prompt".into());
            }
            Ok(AssistantCmd::Send(text))
        }
        Some(other) => Err(format!("assistant has state and send TEXT (not {other})")),
    }
}

fn settings(rest: &[String]) -> Result<SettingsCmd, String> {
    match (
        rest.first().map(String::as_str),
        rest.get(1).map(String::as_str),
    ) {
        (None, _) => Ok(SettingsCmd::Read),
        (Some("slow-answer"), Some("on")) => Ok(SettingsCmd::SlowAnswer(true)),
        (Some("slow-answer"), Some("off")) => Ok(SettingsCmd::SlowAnswer(false)),
        _ => Err("settings reads; settings slow-answer on|off sets one value".into()),
    }
}

fn matching(rest: &[String]) -> Result<MatchCmd, String> {
    match rest.first().map(String::as_str) {
        None | Some("state") => Ok(MatchCmd::State),
        Some("start") => Ok(MatchCmd::Start),
        Some(other) => Err(format!("match has state and start (not {other})")),
    }
}

fn folders(rest: &[String]) -> Result<FoldersCmd, String> {
    match rest.first().map(String::as_str) {
        None | Some("list") => Ok(FoldersCmd::List),
        Some("open") => Ok(FoldersCmd::Open(stack_folder(rest, 1)?)),
        Some(other) => Err(format!("folders has list and open PATH (not {other})")),
    }
}

/// The command for a word that names one of the screens (`words[0]`).
pub fn parse(words: &[String]) -> Result<Command, String> {
    let rest = &words[1..];
    let cmd = match words[0].as_str() {
        "achievements" => ScreenCmd::Achievements(achievements(rest)?),
        "graph" if rest.is_empty() => ScreenCmd::Graph,
        "graph" => return Err("graph takes no words".into()),
        "assistant" | "agent" => ScreenCmd::Assistant(assistant(rest)?),
        "settings" => ScreenCmd::Settings(settings(rest)?),
        "match" => ScreenCmd::Match(matching(rest)?),
        "folders" => ScreenCmd::Folders(folders(rest)?),
        other => return Err(format!("unknown screen command {other}")),
    };
    Ok(Command::Screens(cmd))
}
