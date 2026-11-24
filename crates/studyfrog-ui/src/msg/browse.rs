//! Stack browser and item list messages.

pub use crate::state::DraftField;
use crate::state::{ItemRow, Selection, VaultInfo};
use studyfrog_core::uid::Uid;

/// Stack browser and item list messages.
#[derive(Debug, Clone, PartialEq)]
pub enum MsgBrowse {
    /// The items that link to an open item.
    BacklinksLoaded(
        studyfrog_core::uid::Uid,
        Result<Vec<studyfrog_store::overview::ItemRow>, String>,
    ),
    /// How familiar the user is with an open item, over time.
    FamCurveLoaded(
        studyfrog_core::uid::Uid,
        Result<Box<studyfrog_service::fam_curve::ItemCurve>, String>,
    ),
    /// A stack (or the inbox) was chosen in the browser.
    Select(Selection),
    /// The title filter changed.
    SearchEdited(String),
    /// The items of the selection were loaded.
    ItemsLoaded(Result<Vec<ItemRow>, String>),
    /// The status counts of the selected stack were loaded.
    StatusCountsLoaded(Result<Vec<(studyfrog_core::status::Status, u32)>, String>),
    /// A field of the new-card form changed.
    DraftEdited(DraftField, String),
    /// "Add card" was activated.
    AddCardClicked,
    /// A stack or card was created (or not); carries the refreshed vault.
    Created(Result<VaultInfo, String>),
    /// An item was chosen for editing.
    OpenItem(Uid),
}
