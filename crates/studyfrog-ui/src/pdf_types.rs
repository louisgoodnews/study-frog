//! What the PDF viewer remembers. Pure data; pictures of pages are kept as pixels and become
//! textures only in the view.

use std::sync::Arc;

pub use crate::pdf_effect::PdfEffect;
use studyfrog_pdf::doc::PdfInfo;
use studyfrog_pdf::render::Pixels;
use studyfrog_pdf::text::PageText;

/// Most full-size page pictures kept; they are megabytes each, so the oldest go early.
pub const MAX_PAGE_PICTURES: usize = 6;
/// Most thumbnails kept; they are small, and a long sidebar scrolls through many of them.
pub const MAX_THUMB_PICTURES: usize = 60;
/// Most pictures kept in all. Pages and thumbnails are trimmed apart so that scrolling the
/// thumbnails never pushes out the page being read.
pub const MAX_CACHED: usize = MAX_PAGE_PICTURES + MAX_THUMB_PICTURES;
/// Pictures drawn at or below this scale (thousandths) are thumbnails; the sidebar and the
/// arrange grid draw theirs at exactly this scale.
pub const THUMB_SCALE_MILLI: u32 = 250;
/// The zoom steps of the + and - buttons, in percent.
pub const ZOOM_STEPS: [u32; 9] = [25, 50, 75, 100, 125, 150, 200, 300, 400];

/// One drawn page: which page and at what scale (thousandths, so it can be compared).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageKey {
    /// 0-based page.
    pub page: usize,
    /// Pixels per point times 1000.
    pub scale_milli: u32,
}

/// Whether `key` names a thumbnail rather than a full-size page.
pub fn is_thumb(key: PageKey) -> bool {
    key.scale_milli <= THUMB_SCALE_MILLI
}

/// How big the page is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Zoom {
    /// As wide as the window allows.
    #[default]
    FitWidth,
    /// The whole page in the window.
    FitPage,
    /// A fixed percentage.
    Percent(u32),
}

/// What the side panel lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Side {
    /// The document's outline.
    #[default]
    Outline,
    /// Thumbnails of every page.
    Pages,
    /// The highlights and notes.
    Notes,
    /// Making notes from the text.
    Extract,
    /// Folded away.
    Closed,
}

/// An open PDF.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PdfView {
    /// Vault-relative path.
    pub path: String,
    /// Title shown in the header.
    pub title: String,
    /// Structure, once read.
    pub info: Option<PdfInfo>,
    /// The page shown (0-based).
    pub page: usize,
    /// How big it is shown.
    pub zoom: Zoom,
    /// Words of the page shown.
    pub text: Option<Arc<PageText>>,
    /// Page pictures, oldest first.
    pub pictures: Vec<(PageKey, Arc<Pixels>)>,
    /// Pictures asked for and not yet delivered.
    pub requested: Vec<PageKey>,
    /// Pictures that could not be drawn (not asked for again).
    pub failed: Vec<PageKey>,
    /// What the search box holds.
    pub query: String,
    /// Pages (0-based) that are only pictures, without text.
    pub scanned: Vec<usize>,
    /// Pages (0-based) the query was found on.
    pub found: Vec<usize>,
    /// Indexes of the selected words on the shown page.
    pub selection: Vec<usize>,
    /// The side panel.
    pub side: Side,
    /// Why the file could not be shown.
    pub error: Option<String>,
    /// Highlights and notes.
    pub marks: Vec<studyfrog_config::pdf_marks::PdfMark>,
    /// The page each mark is on now, in the order of `marks`.
    pub marks_now: Vec<Option<usize>>,
    /// The colour new marks get.
    pub color: String,
    /// The next click places a note.
    pub placing_note: bool,
    /// The card or question being written.
    pub draft: Option<crate::pdf_draft_types::Draft>,
    /// A web address waiting for the user's yes.
    pub pending_link: Option<String>,
    /// The "make a note" form.
    pub extract: crate::pdf_extract_types::ExtractForm,
    /// Items made from this PDF.
    pub sourced: Vec<studyfrog_store::pdf_notes::SourcedItem>,
    /// The new file being planned, if any.
    pub arrange: Option<crate::pdf_arrange_types::Arrange>,
}

/// Changes to the viewer.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfMsg {
    /// Open the PDF at this vault-relative path, at a page (0-based).
    Open(String, usize),
    /// Opened: its structure, or why not.
    Opened(String, Result<Box<PdfInfo>, String>),
    /// Leave the viewer.
    Close,
    /// Go to a page (0-based).
    GoTo(usize),
    /// The next page.
    Next,
    /// The previous page.
    Previous,
    /// Zoom in one step.
    ZoomIn,
    /// Zoom out one step.
    ZoomOut,
    /// Set the zoom.
    SetZoom(Zoom),
    /// The view needs this picture.
    Need(PageKey),
    /// A picture was drawn, or could not be.
    Rendered(String, PageKey, Result<Arc<Pixels>, String>),
    /// The words of a page arrived.
    TextLoaded(String, usize, Result<Arc<PageText>, String>),
    /// The search box changed.
    Query(String),
    /// The pages a query was found on.
    Found(String, String, Result<Vec<usize>, String>),
    /// The next page with a hit, wrapping round.
    NextHit,
    /// The previous page with a hit, wrapping round.
    PreviousHit,
    /// Select these words of the shown page.
    Select(Vec<usize>),
    /// Show another side panel.
    ShowSide(Side),
    /// The pages without text arrived.
    Scanned(String, Result<Vec<usize>, String>),
    /// Highlights and notes.
    Marks(crate::pdf_mark_types::MarkMsg),
    /// The PDFs in the library list.
    Library(crate::pdf_library_types::LibraryMsg),
    /// A link on the page was clicked.
    Link(crate::pdf_link_types::LinkMsg),
    /// A card or question being written.
    Draft(crate::pdf_draft_types::DraftMsg),
    /// Making notes from the text.
    Extract(crate::pdf_extract_types::ExtractMsg),
    /// Rearranging into a new file.
    Arrange(crate::pdf_arrange_types::ArrangeMsg),
    /// Exporting notes or stacks to PDF.
    Export(crate::pdf_export_types::PdfExportMsg),
}
