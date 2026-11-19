//! The data of the import and export dialog.

use studyfrog_service::xfer_plan::{PlanLine, Policy};

use crate::state::VaultInfo;

/// Which half of the dialog is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Write a file.
    Export,
    /// Read a file.
    Import,
}

/// The file format of an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// A zip with `export.json` and the pictures and PDFs.
    Json,
    /// A TOML export file.
    Toml,
    /// A RON (Rusty Object Notation) export file.
    Ron,
    /// Cards and questions as CSV.
    Csv,
    /// The vault's Markdown files in a zip.
    Markdown,
}

/// What a dry run found, for the preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preview {
    /// The adapter that read the file.
    pub adapter: String,
    /// The first lines of the plan.
    pub lines: Vec<PlanLine>,
    /// Items in all.
    pub total: usize,
    /// To create, already there, refused.
    pub counts: (usize, usize, usize),
    /// Stacks that would be made.
    pub new_stacks: usize,
    /// What the reader said about the file.
    pub warnings: Vec<String>,
    /// Whether the file carries reviews.
    pub scheduling: bool,
    /// Pictures and PDFs in the file.
    pub files: usize,
    /// How a CSV or TSV file was read, with its problems.
    pub csv: Option<Box<CsvInfo>>,
}

/// How a CSV or TSV file was read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvInfo {
    /// Delimiter, encoding, header, columns and the first rows.
    pub sheet: studyfrog_service::xfer_csv_map::Sheet,
    /// Every problem, for all rows.
    pub problems: Vec<studyfrog_service::xfer_csv_map::Problem>,
}

/// What an import wrote.
#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    /// Items written as new.
    pub created: usize,
    /// Items overwritten.
    pub replaced: usize,
    /// Items left out.
    pub skipped: usize,
    /// Things worth telling.
    pub notes: Vec<String>,
    /// The vault afterwards.
    pub info: VaultInfo,
}

/// How the Universal Import Hub chooses where imported items are placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImportDestination {
    /// Import into an existing stack (or per-item stack when scope is `None`).
    #[default]
    ExistingStack,
    /// Create a new top-level stack and import into it.
    NewStack,
    /// Import directly as unstacked vault notes (Inbox).
    UnstackedNotes,
}

/// Extra Universal Import Hub state carried inside [`TransferState`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HubState {
    /// Which destination option is selected.
    pub dest_mode: ImportDestination,
    /// Title for the new stack when `dest_mode == ImportDestination::NewStack`.
    pub new_stack_name: String,
    /// Whether the chosen path was picked via the folder dialog.
    pub is_folder: bool,
}

/// Concrete target sent to the worker for Universal Import Hub jobs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HubTarget {
    /// Existing stack folder, or `None` ("As the file says").
    Existing(Option<String>),
    /// Create a new stack with this title.
    NewStack(String),
    /// Unstacked vault root (Inbox).
    UnstackedNotes,
}

/// The dialog's fields.
#[derive(Debug, Clone, PartialEq)]
pub struct TransferState {
    /// The half shown.
    pub tab: Tab,
    /// The stack exported or imported into; `None`: the whole library, or what the file names.
    pub scope: Option<String>,
    /// The export format.
    pub format: Format,
    /// The export mode question is open.
    pub choosing_mode: bool,
    /// Advanced mode: include scheduling.
    pub scheduling: bool,
    /// The folder an export is written to.
    pub destination: String,
    /// The file or folder to import.
    pub path: String,
    /// What to do with duplicates.
    pub policy: Policy,
    /// Import the review history of the file.
    pub with_schedule: bool,
    /// The dry run.
    pub preview: Option<Box<Preview>>,
    /// The last result or problem, in words.
    pub status: Option<String>,
    /// A job is running.
    pub busy: bool,
    /// What was found to import, once the search ran.
    pub found: Option<Vec<studyfrog_service::xfer_discover::Found>>,
    /// The search is running.
    pub looking: bool,
    /// What the user chose for reading a CSV file.
    pub csv: studyfrog_service::xfer_csv_map::CsvOptions,
    /// Universal Import Hub state (destination mode, new stack name, folder flag).
    pub hub: HubState,
}

impl Default for TransferState {
    fn default() -> Self {
        TransferState {
            tab: Tab::Export,
            scope: None,
            format: Format::Json,
            choosing_mode: false,
            scheduling: false,
            destination: String::new(),
            path: String::new(),
            policy: Policy::Skip,
            with_schedule: false,
            preview: None,
            status: None,
            busy: false,
            found: None,
            looking: false,
            csv: Default::default(),
            hub: Default::default(),
        }
    }
}

/// What the worker is asked to do.
#[derive(Debug, Clone, PartialEq)]
pub enum TransferJob {
    /// Write the reports as a CSV file into the vault's `exports` folder, or (true) into a folder
    /// chosen in the folder dialog.
    ReportCsv(bool),
    /// Ask for a file (or a folder).
    Browse(bool),
    /// Look for Obsidian vaults and Anki files to suggest.
    Discover,
    /// Read the file and say what an import would do.
    Preview(
        String,
        Option<String>,
        studyfrog_service::xfer_csv_map::CsvOptions,
    ),
    /// Read the file again and write it.
    Import {
        /// The file or folder.
        path: String,
        /// The stack to import into.
        target: Option<String>,
        /// What to do with duplicates.
        policy: Policy,
        /// Import the reviews.
        with_schedule: bool,
        /// How to read a CSV file.
        csv: studyfrog_service::xfer_csv_map::CsvOptions,
    },
    /// Write a sample file of an importer.
    SaveExample {
        /// The folder to write to (empty: Documents, or the home folder).
        folder: String,
        /// The file name.
        file: String,
        /// The text.
        text: String,
    },
    /// Write an export.
    Export {
        /// The folder to write to.
        folder: String,
        /// The stack, or the whole library.
        scope: Option<String>,
        /// The format.
        format: Format,
        /// Include scheduling.
        scheduling: bool,
    },
    /// Universal Import Hub jobs (custom target, PDF, extension card types).
    Hub(HubJob),
}

/// Worker jobs specific to the Universal Import Hub.
#[derive(Debug, Clone, PartialEq)]
pub enum HubJob {
    /// Dry-run preview against `(path, target, csv)`.
    PreviewHub(
        String,
        HubTarget,
        studyfrog_service::xfer_csv_map::CsvOptions,
    ),
    /// Commit import against `(path, target, policy, with_schedule, csv)`.
    ImportHub(
        String,
        HubTarget,
        Policy,
        bool,
        studyfrog_service::xfer_csv_map::CsvOptions,
    ),
    /// Add and index a PDF file into `(path, target)`.
    ImportPdf(String, HubTarget),
    /// Expand `(target, type_id, source)` and write cards into a stack.
    CreateCardType(HubTarget, String, String),
}
