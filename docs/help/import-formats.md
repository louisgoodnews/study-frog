# Import formats

What the importers in the [[import-export|Import and export]] dialog read. The dialog shows the same list under
**Which files can I import?**, and **Save example** there writes the sample files below. Your AI assistant needs the
same information: the prompts in StudyFrog use these layouts.

## CSV or TSV cards
Text with one card, question or note per row. A first row can name the columns: front (or question, term) and back (or answer, definition) are needed; stack (or deck), kind (card, question or note; empty means card), title and tags (separated by semicolons) are optional. Without a header the first two columns are the front and the back. The delimiter (comma, semicolon, tab or |) and the encoding (UTF-8 or Latin-1) are detected; in the import dialog you can choose which column means what. Cells with the delimiter or line breaks go in double quotes.

```csv
stack,kind,title,front,back,tags
french,card,,the cat,le chat,animals;nouns
french,question,Plural,"What is the plural of ""cheval""?",chevaux,
```

After **Preview**, the dialog shows the first rows and lets you choose the delimiter, the encoding, whether the first row is a header and which column is the front, the back, the title, the tags, the stack and the kind. Every problem is listed at once with its row and column, and **Copy the problems for the AI** puts a ready message on the clipboard that asks an AI to fix exactly those rows.

## StudyFrog export (JSON)
One JSON object with "format": "studyfrog-export" and "version": 1, a "stacks" list (folder, title, description) and an "items" list. Each item has "kind" (note, card or question), "stack" (a folder), "title", "text" (a card's front), "answer" (its back) and "tags" (a list). The review history is read only when "scheduling" is true.

```json
{
  "format": "studyfrog-export",
  "version": 1,
  "exported": "2026-01-01T00:00:00Z",
  "scheduling": false,
  "stacks": [
    {"folder": "french", "title": "French", "description": ""}
  ],
  "items": [
    {"kind": "card", "title": "", "stack": "french", "text": "the cat", "answer": "le chat", "tags": ["animals"]}
  ]
}
```

## Zip
A zip made by StudyFrog's JSON export (export.json and a files folder), or a zip of an Obsidian vault: Markdown files in folders.

## Obsidian vault
A folder of Markdown files, or a zip of one. Each folder becomes a stack and each .md file a note, with its front matter, wikilinks, tags and pictures. Callouts, dataview blocks, comments, highlights, block references and note embeds stay as plain text.

## Anki package
An Anki package. In Anki choose File, Export, format Anki Deck Package (.apkg), and tick Support older Anki versions; tick Include scheduling information to bring your reviews. Each deck becomes a stack; basic and cloze notes become cards.

StudyFrog does not read Anki's own database, only exported packages.
