//! What can go wrong with a PDF, in words for the user.

/// Why a PDF could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfError {
    /// A limit was exceeded (size, pages, drawing size); the text names it.
    TooBig(String),
    /// The file is not a usable PDF.
    Invalid(String),
    /// The file is password protected.
    Encrypted,
    /// There is no such page.
    NoSuchPage(usize),
    /// The library did not answer in time.
    Timeout,
    /// The library failed in a way it should not have; the file is left alone.
    Crashed(String),
}

/// A sentence for the user.
pub fn describe_pdf_error(e: &PdfError) -> String {
    match e {
        PdfError::TooBig(what) => format!("This PDF is too big to open here: {what}."),
        PdfError::Invalid(why) => format!("This is not a PDF the app can read: {why}."),
        PdfError::Encrypted => {
            "This PDF is password protected, which is not supported yet.".to_owned()
        }
        PdfError::NoSuchPage(n) => format!("The PDF has no page {}.", n + 1),
        PdfError::Timeout => "Reading this PDF took too long, so it was stopped.".to_owned(),
        PdfError::Crashed(why) => {
            format!("Reading this PDF failed unexpectedly ({why}); the file was not changed.")
        }
    }
}
