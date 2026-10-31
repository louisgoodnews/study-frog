//! Following the links of a PDF page.

use studyfrog_pdf::links::LinkTarget;

/// What the user did with a link.
#[derive(Debug, Clone, PartialEq)]
pub enum LinkMsg {
    /// A link was clicked.
    Click(LinkTarget),
    /// The user said yes to opening the waiting address.
    Approve,
    /// The user said no.
    Dismiss,
}
