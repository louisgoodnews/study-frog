//! An example extension. It uses nothing but `studyfrog-ext`, which is the point: the core crates
//! are not touched. The application's one line of wiring is `studyfrog_ext_example::register`.
//!
//! - a **tab-separated importer** for plain-text exports (`front<TAB>back<TAB>tags`, the layout of
//!   Anki's "Notes in Plain Text" export);
//! - a **comma-separated importer** for CSV exports;
//! - a **cloze card type**: `The {{c1::mitochondria}} is the {{c2::powerhouse}}` becomes one card
//!   per gap.

pub mod cloze;
pub mod csv;
pub mod delimited;
pub mod tsv;

use studyfrog_ext::registry::{register_card_type, register_importer};
use studyfrog_ext::types::Registry;

/// Add this extension's importers and card type to the registry.
pub fn register(r: &mut Registry) -> Result<(), String> {
    register_importer(r, tsv::importer())?;
    register_importer(r, csv::importer())?;
    register_card_type(r, cloze::card_type())
}
