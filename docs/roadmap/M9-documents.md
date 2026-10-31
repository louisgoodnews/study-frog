# M9: Documents (PDF)

**Goal:** PDF files are first-class vault content: found, read, annotated, rearranged and mined for notes and cards.

**Entry criteria:** 0.1.0 released (M0-M8 exit met). ADR 0010 accepted before any other task starts.

## Epics, stories, tasks

- **EPIC-022** PDF documents (added at the owner's request on 2026-10-31)
  - **STORY-0054** PDF files in the vault to view, edit and extract
    - TASK-0109 Spike and ADR 0010 to choose the PDF library
    - TASK-0110 The studyfrog-pdf crate with documents, pages, text and pictures as data
    - TASK-0111 Index PDFs in the vault and make them searchable
    - TASK-0112 PDF viewer screen
    - TASK-0113 Safety limits for hostile or broken PDFs
    - TASK-0114 Annotations as highlights and notes in a sidecar file
    - TASK-0115 Page operations to rotate, delete, reorder, merge and split into a new PDF
    - TASK-0116 Export an annotated copy and edit document properties
    - TASK-0117 Extract text into Markdown notes with a link back to the page
    - TASK-0118 Detect scanned pages and say so
    - TASK-0119 Extract pictures and page snapshots into the vault
    - TASK-0120 Cards and questions from highlighted passages, with the assistant's help
    - TASK-0121 Add, link and organise PDFs
    - TASK-0122 Documentation, tests, accessibility and release notes for PDF support

**Deferred:** OCR for scanned PDFs (owner's decision, 2026-10-31); no tasks yet.

**Exit criteria:**
- A PDF added to a vault is listed, found by search and opened in the viewer
- Highlights and notes survive a restart and are never written into the original file
- Pages can be rotated, deleted, reordered, merged and split into a new file
- A passage becomes a note, a card or a question that links back to its page
- The malformed-file corpus neither crashes nor hangs the app
- The numbers in TASK-0112 and TASK-0122 are recorded in the benchmarking doc
