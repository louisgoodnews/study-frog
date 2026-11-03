//! StudyFrog binary entry point.
//!
//! `studyfrog --smoke-frames N` draws N frames and exits (for CI under a virtual display).
//! `studyfrog --screenshot FILE [--screen NAME] [--size WxH] [--theme dark|light|system]` saves a PNG of a screen and exits (`home`, `library`,
//! `rehearse`, `settings`, `help`, `calendar`, `board`, `reports`, `agent`, or `run`, or `stack:<folder>`).

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
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match studyfrog_ui::run_with(options(&args), studyfrog_app::host()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("studyfrog: {e}");
            ExitCode::FAILURE
        }
    }
}
