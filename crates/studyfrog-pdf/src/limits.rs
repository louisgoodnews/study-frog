//! Limits and isolation for untrusted files (TASK-0113). A PDF comes from anywhere, so every call runs on
//! its own thread under a time limit and with panics caught; sizes are checked before the work starts.

use std::sync::mpsc;
use std::time::Duration;

use crate::PdfBytes;
use crate::error::PdfError;

/// Largest file opened, in bytes.
pub const MAX_FILE_BYTES: usize = 200 * 1024 * 1024;
/// Most pages a file may have.
pub const MAX_PAGES: usize = 5_000;
/// Most pixels one drawing may have.
pub const MAX_RENDER_PIXELS: u64 = 36_000_000;
/// Longest side of a page, in points (PDF's own limit is 14 400).
pub const MAX_PAGE_POINTS: f32 = 14_400.0;
/// Seconds a call on one page may take.
pub const TIME_LIMIT_SECS: u64 = 30;
/// Seconds a call on a whole file (indexing) may take.
pub const FILE_TIME_LIMIT_SECS: u64 = 180;

/// Refuse a file that is over the size limit.
pub fn check_size(bytes: &PdfBytes) -> Result<(), PdfError> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(PdfError::TooBig(format!(
            "the file is larger than {} MB",
            MAX_FILE_BYTES / (1024 * 1024)
        )));
    }
    if !bytes.starts_with(b"%PDF")
        && !bytes[..bytes.len().min(1024)]
            .windows(4)
            .any(|w| w == b"%PDF")
    {
        return Err(PdfError::Invalid("it does not start like a PDF".to_owned()));
    }
    Ok(())
}

/// Run `work` on its own thread. A panic becomes [`PdfError::Crashed`] and running over the time limit
/// becomes [`PdfError::Timeout`] (the thread is left to finish by itself; it holds nothing the caller needs).
pub fn guarded<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, PdfError> + Send + 'static,
) -> Result<T, PdfError> {
    guarded_for(TIME_LIMIT_SECS, work)
}

/// Like [`guarded`] with a different time limit, for work over a whole file.
pub fn guarded_for<T: Send + 'static>(
    seconds: u64,
    work: impl FnOnce() -> Result<T, PdfError> + Send + 'static,
) -> Result<T, PdfError> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).unwrap_or_else(|p| {
                let why = p
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_owned())
                    .or_else(|| p.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown".to_owned());
                Err(PdfError::Crashed(why))
            });
        let _ = tx.send(result);
    });
    rx.recv_timeout(Duration::from_secs(seconds))
        .map_err(|_| PdfError::Timeout)?
}
