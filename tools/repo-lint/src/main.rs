//! `repo-lint impls <root>` and `repo-lint issues <issues-dir>`.
//! Prints `path:line: message` and exits 1 on any finding.

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

fn report(lines: &[String]) -> ExitCode {
    lines.iter().for_each(|l| println!("{l}"));
    if lines.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn usage() -> ExitCode {
    eprintln!("usage: repo-lint impls <root> | repo-lint issues <issues-dir>");
    ExitCode::from(2)
}
