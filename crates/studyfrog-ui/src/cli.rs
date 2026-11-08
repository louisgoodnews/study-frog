//! Command-line access to the application's own files: `--print-paths` lists where they are and
//! `logs [--tail N] [--follow] [--json]` shows the application log.

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;

use studyfrog_config::paths::{AppPaths, LOG_JSON_FILE, LOG_TEXT_FILE};
use studyfrog_service::logging::tail_lines;

use crate::runtime::paths;

/// Lines shown by `logs` when `--tail` is not given.
const DEFAULT_TAIL: usize = 50;

/// What `logs` was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogsArgs {
    /// How many of the last lines to show first.
    pub tail: usize,
    /// Keep running and show lines as they are written.
    pub follow: bool,
    /// Show the JSON Lines log instead of the text log.
    pub json: bool,
}

/// Read the arguments after `logs`.
pub fn parse_logs_args(args: &[String]) -> Result<LogsArgs, String> {
    let mut out = LogsArgs {
        tail: DEFAULT_TAIL,
        follow: false,
        json: false,
    };
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--follow" => out.follow = true,
            "--json" => out.json = true,
            "--tail" => {
                let n = rest.next().ok_or("--tail needs a number")?;
                out.tail = n.parse().map_err(|_| format!("--tail {n}: not a number"))?;
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(out)
}

/// The application's directories and log files, one per line.
pub fn paths_report(p: &AppPaths) -> String {
    let rows = [
        ("home", p.root.clone()),
        ("config", p.config.clone()),
        ("profiles", p.profiles.clone()),
        ("models", p.models.clone()),
        ("run", p.run.clone()),
        ("logs", p.logs.clone()),
        ("text log", p.logs.join(LOG_TEXT_FILE)),
        ("json log", p.logs.join(LOG_JSON_FILE)),
    ];
    rows.iter()
        .map(|(name, path)| format!("{name:<9} {}\n", path.display()))
        .collect()
}

/// What was written to `path` after the first `seen` bytes, and the size read up to. A log that
/// became smaller was rotated, so it is read again from its start.
fn read_new(path: &Path, seen: u64) -> io::Result<(Vec<u8>, u64)> {
    let len = fs::metadata(path).map_or(0, |m| m.len());
    let from = if len < seen { 0 } else { seen };
    if len == from {
        return Ok((Vec::new(), len));
    }
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(from))?;
    let mut added = Vec::new();
    file.read_to_end(&mut added)?;
    let end = from + added.len() as u64;
    Ok((added, end))
}

/// Wait before looking at the log again. The one place this command blocks: it is a command-line
/// edge, not domain code, so it is passed to `follow` rather than called inside it.
#[allow(clippy::disallowed_methods)]
fn pause() {
    std::thread::sleep(Duration::from_millis(500));
}

/// Print what is written to `path` after the first `seen` bytes, forever.
fn follow(path: &Path, mut seen: u64, wait: fn()) -> io::Result<()> {
    loop {
        wait();
        let (added, end) = read_new(path, seen)?;
        print!("{}", String::from_utf8_lossy(&added));
        seen = end;
    }
}

fn show_logs(p: &AppPaths, args: LogsArgs) -> io::Result<()> {
    let name = if args.json {
        LOG_JSON_FILE
    } else {
        LOG_TEXT_FILE
    };
    let path = p.logs.join(name);
    for line in tail_lines(&path, args.tail)? {
        println!("{line}");
    }
    if args.follow {
        follow(&path, fs::metadata(&path).map_or(0, |m| m.len()), pause)?;
    }
    Ok(())
}

fn logs(args: &[String]) -> u8 {
    let result = parse_logs_args(args)
        .map_err(|e| e.to_string())
        .and_then(|a| {
            let p = paths().ok_or("no home directory is known")?;
            show_logs(&p, a).map_err(|e| e.to_string())
        });
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("studyfrog logs: {e}");
            1
        }
    }
}

fn print_paths() -> u8 {
    match paths() {
        Some(p) => {
            print!("{}", paths_report(&p));
            0
        }
        None => {
            eprintln!("studyfrog: no home directory is known");
            1
        }
    }
}

/// Run the command in `args` (the program name first) if it is one of ours; `None` means open the
/// window as usual.
pub fn run(args: &[String]) -> Option<u8> {
    if args.iter().skip(1).any(|a| a == "--mcp") {
        return Some(crate::mcp_stdio::run(args));
    }
    match args.get(1).map(String::as_str) {
        Some("--print-paths") => Some(print_paths()),
        Some("logs") => Some(logs(&args[2..])),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use studyfrog_config::paths::app_paths;

    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn logs_options_have_defaults_and_can_be_combined() {
        let default = parse_logs_args(&[]).unwrap();
        assert_eq!(
            (default.tail, default.follow, default.json),
            (50, false, false)
        );
        let all = parse_logs_args(&args(&["--tail", "7", "--follow", "--json"])).unwrap();
        assert_eq!((all.tail, all.follow, all.json), (7, true, true));
    }

    #[test]
    fn bad_logs_options_say_what_is_wrong() {
        assert!(
            parse_logs_args(&args(&["--tail"]))
                .unwrap_err()
                .contains("number")
        );
        assert!(
            parse_logs_args(&args(&["--tail", "x"]))
                .unwrap_err()
                .contains("not a number")
        );
        assert!(
            parse_logs_args(&args(&["--loud"]))
                .unwrap_err()
                .contains("--loud")
        );
    }

    #[test]
    fn only_what_was_added_is_read_and_a_rotated_log_is_read_again() {
        let dir = std::env::temp_dir().join(format!("studyfrog-cli-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let log = dir.join("studyfrog.log");
        fs::write(&log, "one\n").unwrap();
        let (first, seen) = read_new(&log, 0).unwrap();
        assert_eq!((first.as_slice(), seen), (b"one\n".as_slice(), 4));
        assert_eq!(read_new(&log, seen).unwrap(), (Vec::new(), 4));
        fs::write(&log, "one\ntwo\n").unwrap();
        assert_eq!(read_new(&log, seen).unwrap(), (b"two\n".to_vec(), 8));
        fs::write(&log, "new\n").unwrap();
        assert_eq!(read_new(&log, 8).unwrap(), (b"new\n".to_vec(), 4));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_paths_report_names_every_directory_and_both_logs() {
        let report = paths_report(&app_paths(Path::new("/s")));
        assert!(report.contains("config    /s/config\n"));
        assert!(report.contains("json log  /s/logs/console.jsonl\n"));
        assert_eq!(report.lines().count(), 8);
    }
}
