# Extending StudyFrog

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

## Limits

- Extensions are compiled in. There is no loading of code at run time, no sandbox and no permissions: an
  extension is as trusted as the application. A dynamic plugin ABI would need `unsafe`, which this workspace
  forbids, or a separate process; neither is built.
- Only importers and card types exist. Other extension points (exporters, scorers, report panels) would be
  new record types in `studyfrog-ext` following the same pattern.
- A card type produces cards from text once, at creation time. It does not keep the source: editing one of the
  generated cards does not regenerate its siblings.
