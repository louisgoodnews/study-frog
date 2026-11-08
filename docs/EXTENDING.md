# Extending StudyFrog

There are two ways to extend StudyFrog:

| | Compiled-in extensions (this page, ADR 0008) | Scripts (ADR 0013, [scripting spec](specs/scripting.md)) |
|---|---|---|
| Written in | Rust, a crate built into the binary | any language; Python has a client (`python/studyfrog`) |
| Installed by | rebuilding StudyFrog | dropping a file into `<app home>/scripts/` |
| Adds | importers and card types in the window's dialogs; format adapters | importers run from the command line or the Console, scheduled jobs, event hooks, automated notes and exports |
| Can reach | text in, cards out; no I/O | the control protocol, under the script's access (read, write, destructive) |
| Runs | inside the application | as a child process with a timeout |

An extension adds an **importer** or a **card type** without changing any core crate. Extensions are
plain data (records of function pointers) gathered in a registry; see
[ADR 0008](adr/0008-extensions-as-plain-data.md) for why.

## The two extension points (`studyfrog-ext`)

```rust
pub struct Importer {
    pub id: &'static str,                 // lowercase letters, digits and `-`; unique
    pub name: &'static str,               // shown to the user
    pub extensions: &'static [&'static str], // file extensions it reads, lowercase, no dot
    pub import: fn(&str) -> Result<Vec<ImportedCard>, String>,
}

pub struct CardType {
    pub id: &'static str,
    pub name: &'static str,
    pub hint: &'static str,               // one line: what to type
    pub expand: fn(&str) -> Result<Vec<Faces>, String>,
}
```

- An **importer** gets the *text* of a file (the application reads it) and returns the cards it found
  (front, back, optional title, tags). The user picks a file in a stack's "Import from a file" panel; the
  importer is chosen by the file extension (case-insensitive, first registered wins).
- A **card type** gets one source text and returns one or more cards. When any card type is registered the
  new-card form offers it next to "Basic"; the text field becomes "Source text" and shows the hint.

Both functions must be pure: no I/O, no panics, no global state. Return `Err` with a message that says what
is wrong with the input (an importer should include a line number). The application shows the message.

What the application does with the result is not up to the extension: every card is validated like a card
typed by hand (a front and a back are required, the title must be one line), written as a Markdown file with
its own id, and the vault is reindexed once. Cards that cannot be written are skipped and reported by number;
an import is limited to 5,000 cards.

## The registry

```rust
let mut registry = registry_new();
register_importer(&mut registry, my_importer)?;   // refused: bad or repeated id, no extensions
register_card_type(&mut registry, my_card_type)?;
```

A `Registry` is two `Vec`s; look things up with `importer_for_file`, `find_card_type` and
`importable_extensions`.

## Writing an extension

1. Make a crate that depends on `studyfrog-ext` and nothing else from this workspace
   (`scripts/check-deps.sh` enforces it for everything under `examples/`).
2. Export `pub fn register(r: &mut Registry) -> Result<(), String>` that registers your records.
3. Add one line to `crates/studyfrog-app/src/lib.rs`: `studyfrog_my_extension::register(&mut registry)`.

`examples/studyfrog-ext-example` is a complete one: a **tab-separated importer** (the layout of Anki's
"Notes in Plain Text" export: `front<TAB>back<TAB>tags`, `#` header lines skipped, `<br>` becomes a line
break) and a **cloze card type** (`The {{c1::mitochondria}} is the {{c2::powerhouse::energy}}` becomes one
card per gap number; the front hides the gap or shows its hint, the back shows the answer in bold).

## Writing a format adapter (import and export of whole libraries)

The importers above add cards to a stack. Whole files in other formats (an Anki package, an Obsidian vault, a JSON export) are read
by **format adapters**, which build a *bundle* of stacks, items and files; the same dry run, duplicate handling and writing then
serve every format ([interchange](specs/interchange.md)). A new format needs no change to the core.

An adapter is one record in `ADAPTERS` in `crates/studyfrog-service/src/xfer_adapters.rs`:

```rust
Adapter { name: "My format", extensions: &["myf"], read: my_format }
```

`read` is `fn(&[u8]) -> Result<Bundle, String>`. It must never panic and must bound its work: use `xfer_zip::read_zip` for zips (it
limits the number and size of files), refuse input over the limits in `xfer_plan::MAX_ITEMS`, and return an error string for anything it
cannot read. The bundle (`xfer_model.rs`) holds `XStack { folder, title, description }`, `XItem { kind, title, text, answer, tags, stack,
status, priority, difficulty, reviews, extra }` and `files` (vault-relative path and bytes). Folders must be relative with no `..`;
`xfer_model::safe_folder` checks it and the import refuses what fails. Put things the format has and StudyFrog does not into
`bundle.warnings` ("3 callouts kept as text") so the preview can say so.

**Worked example**, a format of one line per card, `front|back|tag1,tag2`:

```rust
fn pipes(bytes: &[u8]) -> Result<Bundle, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "not UTF-8 text".to_owned())?;
    let mut b = Bundle::default();
    for (n, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        let [front, back] = [parts.first(), parts.get(1)].map(|p| p.copied().unwrap_or_default());
        if back.is_empty() {
            b.warnings.push(format!("line {}: no back, left out", n + 1));
            continue;
        }
        let mut item = xfer_model::item(ItemKind::Card, "", front, back);
        item.tags = parts.get(2).map_or_else(Vec::new, |t| t.split(',').map(|s| s.trim().to_owned()).collect());
        b.items.push(item);
    }
    Ok(b)
}
```

Add `Adapter { name: "Pipe-separated cards", extensions: &["pipes"], read: pipes }` to `ADAPTERS`; the import dialog then offers it,
shows its preview and writes its cards with the user's choice for duplicates. Test it like the others in
`crates/studyfrog-service/tests/xfer_*.rs`: a good file, and a corpus of broken ones that must give an error, not a crash.

## Script plugins (out of process)

What a script can extend today, with the examples in `examples/scripts/`:

- **Importers** for any format: a script reads the file and creates items through the API
  (`import_glossary.py`: `term — definition` lines become cards, already present terms are skipped). Run it with
  `studyfrog script run import_glossary.py -- file.txt --stack french` or from the Console. A script can also convert a
  file into StudyFrog's JSON interchange format and call `import_file`, which then goes through the usual dry run
  and duplicate handling.
- **Custom cards**: a script can generate cards of any shape from a source (a table, a word list, a PDF's text) and
  create them as cards, questions or cloze notes.
- **Automated views**: a scheduled or event-bound script that writes a note (`health_check.py --note STACK`) or an
  export (`export_vault.py`), or tags new items (`tag_new_cards.py`).

What scripts cannot do: add an entry to the window's import dialog or the new-card form, draw in the window, add a
screen, or reach calendar, board, diary and settings actions (the control protocol does not have them yet). For those
a compiled-in extension (below) or a new control command is needed.

## Limits

- Extensions are compiled in. There is no loading of code at run time, no sandbox and no permissions: an
  extension is as trusted as the application. A dynamic plugin ABI would need `unsafe`, which this workspace
  forbids; the separate-process route is the scripts above, which talk to the application but do not plug into
  its dialogs.
- Only importers and card types exist. Other extension points (exporters, scorers, report panels) would be
  new record types in `studyfrog-ext` following the same pattern.
- A card type produces cards from text once, at creation time. It does not keep the source: editing one of the
  generated cards does not regenerate its siblings.
