//! `repo-lint impls <root>`, `repo-lint issues <issues-dir>` and
//! `repo-lint board <issues-dir> <BOARD.md> [--check]`, and `repo-lint bench-compare <base.ns> <new.ns>`.
//! Prints `path:line: message` and exits 1 on any finding.

mod bench_compare;
mod board;
mod files;
mod impls;
mod issues;
mod rules;

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("bench-compare") {
        return run_bench_compare(&args[1..]);
    }
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

/// Exit 0 for no change or an improvement, 1 for a regression, 2 for bad input.
fn run_bench_compare(args: &[String]) -> ExitCode {
    let [base, new] = args else {
        return usage();
    };
    match bench_compare::run(Path::new(base), Path::new(new)) {
        Ok(c) => {
            println!("{}", bench_compare::format_line(&c));
            if c.verdict == bench_compare::Verdict::Regression {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("bench-compare: {e}");
            ExitCode::from(2)
        }
    }
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: repo-lint impls <root> | issues <dir> | board <dir> <out> [--check] | bench-compare <base.ns> <new.ns>"
    );
    ExitCode::from(2)
}
