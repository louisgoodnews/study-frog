//! What the PDF viewer remembers. Pure data; pictures of pages are kept as pixels and become
//! textures only in the view.

use std::sync::Arc;

use studyfrog_pdf::doc::PdfInfo;
use studyfrog_pdf::render::Pixels;
use studyfrog_pdf::text::PageText;

/// Most page pictures kept (thumbnails included); the oldest go first.
pub const MAX_CACHED: usize = 40;
/// Pictures drawn at or below this scale (thousandths) are thumbnails.
pub const THUMB_SCALE_MILLI: u32 = 300;
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
    /// A card or question being written.
    Draft(crate::pdf_draft_types::DraftMsg),
    /// Making notes from the text.
    Extract(crate::pdf_extract_types::ExtractMsg),
    /// Rearranging into a new file.
    Arrange(crate::pdf_arrange_types::ArrangeMsg),
}

/// Work the viewer asks the worker for.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfEffect {
    /// Open a PDF of the vault and read its structure.
    Open(String),
    /// Draw one page of a PDF.
    Render {
        /// Vault-relative path.
        path: String,
        /// Which page at what scale.
        key: PageKey,
    },
    /// Read the words of one page (0-based).
    Text {
        /// Vault-relative path.
        path: String,
        /// The page.
        page: usize,
    },
    /// Find the pages of a PDF that contain a text.
    Find {
        /// Vault-relative path.
        path: String,
        /// The text.
        query: String,
    },
    /// Find PDFs by title and text for the global search.
    Search(String),
    /// Ask for another PDF to merge in.
    PickMerge,
    /// Write a new PDF from a plan.
    SaveCopy(studyfrog_service::pdf_ops::SaveRequest),
    /// Write one file per page range.
    Split {
        /// Vault-relative path of the PDF.
        path: String,
        /// 0-based first and last page of each range.
        ranges: Vec<(usize, usize)>,
    },
    /// Make a note from pages or a selection.
    MakeNote {
        /// Vault-relative path of the PDF.
        path: String,
        /// First page (0-based).
        first: usize,
        /// Last page (0-based).
        last: usize,
        /// The selected text, if that is what to use.
        selection: Option<String>,
        /// The stack folder.
        folder: String,
        /// The title, or empty.
        title: String,
        /// Pictures to put at the end.
        pictures: Vec<String>,
    },
    /// Save the pictures of a page.
    SavePictures {
        /// Vault-relative path.
        path: String,
        /// The page (0-based).
        page: usize,
    },
    /// Save a rectangle of a page as a picture.
    SaveRegion {
        /// Vault-relative path.
        path: String,
        /// The page (0-based).
        page: usize,
        /// The rectangle.
        rect: studyfrog_pdf::render::PageRect,
    },
    /// Adding, listing and renaming PDFs.
    Library(crate::pdf_library_types::LibraryJob),
    /// Make a card or a question from a passage.
    MakeItem(crate::pdf_draft_types::ItemJob),
    /// List the items made from a PDF.
    LoadSourced(String),
    /// List the pages of a PDF that have no text.
    LoadScanned(String),
    /// Read the highlights and notes of a PDF.
    LoadMarks(String),
    /// Write the highlights and notes of a PDF.
    SaveMarks {
        /// Vault-relative path.
        path: String,
        /// All of its marks.
        marks: Vec<studyfrog_config::pdf_marks::PdfMark>,
    },
}
