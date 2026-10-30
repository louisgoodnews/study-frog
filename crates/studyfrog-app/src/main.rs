//! StudyFrog binary entry point.
//!
//! `studyfrog --smoke-frames N` draws N frames and exits (for CI under a virtual display).

use std::process::ExitCode;

/// The value after `--smoke-frames`, if given and numeric.
fn smoke_frames(args: &[String]) -> Option<u32> {
    let at = args.iter().position(|a| a == "--smoke-frames")?;
    args.get(at + 1)?.parse().ok()
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match studyfrog_ui::run(smoke_frames(&args)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("studyfrog: {e}");
            ExitCode::FAILURE
        }
    }
}
