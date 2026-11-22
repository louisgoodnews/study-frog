//! The plain-words layouts and sample files of the importers, shown in the import dialog, the formats
//! list and the error messages. Text only; each importer in `xfer_adapters` names its own.

/// The rules for an Obsidian vault folder (it has no file extension, so it is not an [`Adapter`]).
pub const OBSIDIAN_LAYOUT: &str = "A folder of Markdown files, or a zip of one. Each folder becomes a stack and each .md file a note, with its front matter, wikilinks, tags and pictures. Callouts, dataview blocks, comments, highlights, block references and note embeds stay as plain text.";

pub(crate) const JSON_LAYOUT: &str = "One JSON object with \"format\": \"studyfrog-export\" and \"version\": 1, a \"stacks\" list (folder, title, description) and an \"items\" list. Each item has \"kind\" (note, card or question), \"stack\" (a folder), \"title\", \"text\" (a card's front), \"answer\" (its back) and \"tags\" (a list). The review history is read only when \"scheduling\" is true.";

pub(crate) const ZIP_LAYOUT: &str = "A zip made by StudyFrog's JSON export (export.json and a files folder), or a zip of an Obsidian vault: Markdown files in folders.";

/// What a CSV or TSV file has to look like.
pub const CSV_LAYOUT: &str = "Text with one card, question or note per row. A first row can name the columns: front (or question, term) and back (or answer, definition) are needed; stack (or deck), kind (card, question or note; empty means card), title and tags (separated by semicolons) are optional. Without a header the first two columns are the front and the back. The delimiter (comma, semicolon, tab or |) and the encoding (UTF-8 or Latin-1) are detected; in the import dialog you can choose which column means what. Cells with the delimiter or line breaks go in double quotes.";

pub(crate) const PDF_LAYOUT: &str = "A PDF document. Each page with text becomes one card: the page's first line is the front, the lines below it the back, and the title is the page number with the file name. Pages without text (scans) are left out.";

pub(crate) const ANKI_LAYOUT: &str = "An Anki package. In Anki choose File, Export, format Anki Deck Package (.apkg), and tick Support older Anki versions; tick Include scheduling information to bring your reviews. Each deck becomes a stack; basic and cloze notes become cards.";

pub(crate) const JSON_EXAMPLE: &str = "{\n  \"format\": \"studyfrog-export\",\n  \"version\": 1,\n  \"exported\": \"2026-01-01T00:00:00Z\",\n  \"scheduling\": false,\n  \"stacks\": [\n    {\"folder\": \"french\", \"title\": \"French\", \"description\": \"\"}\n  ],\n  \"items\": [\n    {\"kind\": \"card\", \"title\": \"\", \"stack\": \"french\", \"text\": \"the cat\", \"answer\": \"le chat\", \"tags\": [\"animals\"]}\n  ]\n}\n";

pub(crate) const CSV_EXAMPLE: &str = "stack,kind,title,front,back,tags\nfrench,card,,the cat,le chat,animals;nouns\nfrench,question,Plural,\"What is the plural of \"\"cheval\"\"?\",chevaux,\n";
