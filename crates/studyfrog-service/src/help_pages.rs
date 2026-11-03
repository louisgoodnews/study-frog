//! The table of supplied help pages, compiled in from `docs/help/`. The order is the order of the tree.

use crate::help::{HelpPage, Section};

macro_rules! page {
    ($id:literal, $section:ident) => {
        HelpPage {
            id: $id,
            section: Section::$section,
            text: include_str!(concat!("../../../docs/help/", $id, ".md")),
        }
    };
}

/// Every supplied page.
pub const PAGES: &[HelpPage] = &[
    page!("why-studyfrog", Start),
    page!("getting-started", Start),
    page!("privacy", Start),
    page!("home", Screens),
    page!("library", Screens),
    page!("stack-page", Screens),
    page!("item-page", Screens),
    page!("create-dialog", Screens),
    page!("rehearse", Screens),
    page!("calendar", Screens),
    page!("board", Screens),
    page!("reports", Screens),
    page!("achievements", Screens),
    page!("assistant", Screens),
    page!("graph", Screens),
    page!("pdf", Screens),
    page!("settings", Screens),
    page!("import-export", Screens),
    page!("import-formats", Screens),
    page!("notifications", Screens),
    page!("help", Screens),
    page!("faq", Reference),
    page!("glossary", Reference),
    page!("keyboard", Reference),
    page!("query-language", Reference),
    page!("familiarity", Reference),
    page!("statuses", Reference),
];
