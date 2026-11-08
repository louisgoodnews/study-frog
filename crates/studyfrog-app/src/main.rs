//! StudyFrog binary entry point.
//!
//! `studyfrog --smoke-frames N` draws N frames and exits (for CI under a virtual display).
//! `studyfrog --screenshot FILE [--screen NAME] [--size WxH] [--theme dark|light|system]` saves a PNG of a screen and exits (`home`, `library`,
//! `rehearse`, `settings`, `help`, `calendar`, `board`, `reports`, `agent`, or `run`, or `stack:<folder>`).
//! `studyfrog --print-paths` lists where the application keeps its files, and
//! `studyfrog logs [--tail N] [--follow] [--json]` shows its log, and
//! `studyfrog --mcp [--vault PATH] [--mcp-allow-write] [--mcp-allow-destructive]` serves the vault to AI
//! clients over the Model Context Protocol on standard input and output.
//! `studyfrog --control-stdio [--vault PATH]` runs headless, driven by control requests on standard input, and
//! `studyfrog ctl <command>` sends one to a running instance (`studyfrog ctl help` lists them). The window serves
//! `run/control.sock` unless `--no-control` is given; `--control-read-only` and `--control-allow-destructive`
//! set what requests may change.

use std::process::ExitCode;

use studyfrog_ui::RunOptions;
use studyfrog_ui::app_shot::Shot;

/// The value after `flag`, if given.
fn value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let at = args.iter().position(|a| a == flag)?;
    args.get(at + 1).map(String::as_str)
}

fn size(args: &[String]) -> Option<(f32, f32)> {
    let (w, h) = value(args, "--size")?.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

fn options(args: &[String]) -> RunOptions {
    RunOptions {
        smoke_frames: value(args, "--smoke-frames").and_then(|v| v.parse().ok()),
        shot: value(args, "--screenshot").map(|path| Shot {
            path: path.into(),
            screen: value(args, "--screen").map(str::to_owned),
            frames: 0,
            theme: value(args, "--theme").and_then(studyfrog_ui::app_shot::theme_arg),
        }),
        size: size(args),
        control: control_policy(args),
    }
}

/// The control protocol's policy for the window, or `None` when it is not served (also for the
/// smoke and screenshot runs, which must not take the socket of a running instance).
fn control_policy(args: &[String]) -> Option<studyfrog_ui::control_cmd::ControlPolicy> {
    let has = |flag: &str| args.iter().any(|a| a == flag);
    if has("--no-control") || has("--smoke-frames") || has("--screenshot") {
        return None;
    }
    let mut policy = studyfrog_ui::control_cmd::policy_default();
    policy.allow_write = !has("--control-read-only");
    policy.allow_destructive = has("--control-allow-destructive");
    Some(policy)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().skip(1).any(|a| a == "--control-stdio") {
        return ExitCode::from(studyfrog_ui::control_stdio::run(
            &args,
            studyfrog_app::host(),
        ));
    }
    if let Some(code) = studyfrog_ui::cli::run(&args) {
        return ExitCode::from(code);
    }
    match studyfrog_ui::run_with(options(&args), studyfrog_app::host()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("studyfrog: {e}");
            ExitCode::FAILURE
        }
    }
}
