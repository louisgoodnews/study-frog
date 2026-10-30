//! `repo-lint impls <root>`, `repo-lint issues <issues-dir>` and
//! `repo-lint board <issues-dir> <BOARD.md> [--check]`.
//! Prints `path:line: message` and exits 1 on any finding.

mod board;
mod files;
mod impls;
mod issues;
mod rules;

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let lines = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["impls", root] => run_impls(Path::new(root)),
        ["issues", dir] => run_issues(Path::new(dir)),
        ["board", dir, out] => run_board(Path::new(dir), Path::new(out), false),
        ["board", dir, out, "--check"] => run_board(Path::new(dir), Path::new(out), true),
        _ => return usage(),
    };
    report(&lines)
}

fn run_impls(root: &Path) -> Vec<String> {
    files::find_files(root, "rs")
        .iter()
        .flat_map(|path| {
            impls::check_file(path)
                .into_iter()
                .map(|f| format!("{}:{}: {}", path.display(), f.line, f.message))
        })
        .collect()
}

fn run_issues(dir: &Path) -> Vec<String> {
    let (loaded, mut errors) = issues::load_all(dir);
    errors.extend(rules::check_all(&loaded));
    errors
}

fn run_board(dir: &Path, out: &Path, check: bool) -> Vec<String> {
    let (loaded, mut errors) = issues::load_all(dir);
    errors.extend(board::wip_errors(&loaded));
    let text = board::render(&loaded);
    if check {
        errors.extend(stale_error(out, &text));
    } else if let Err(e) = std::fs::write(out, &text) {
        errors.push(format!("{}: {e}", out.display()));
    }
    errors
}

fn stale_error(out: &Path, expected: &str) -> Option<String> {
    let current = std::fs::read_to_string(out).unwrap_or_default();
    (current != expected).then(|| {
        format!(
            "{} is stale; run scripts/board.sh and commit it",
            out.display()
        )
    })
}

fn report(lines: &[String]) -> ExitCode {
    lines.iter().for_each(|l| println!("{l}"));
    if lines.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: repo-lint impls <root> | issues <dir> | board <dir> <out> [--check]");
    ExitCode::from(2)
}
