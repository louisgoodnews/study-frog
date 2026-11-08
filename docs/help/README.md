# Help content

The pages of the in-app help (STORY-0066). Each page here is compiled into the binary, shown read-only by the help
screen and searched with it. The user's own pages live in the vault's `wiki/` folder and are listed after these.

## Conventions
- One file per page: `docs/help/<id>.md`. The id is the file name without `.md`; it never changes once released,
  because user pages and the help buttons link to it.
- The first line is `# Title`; nothing else is needed (no front matter).
- Links between pages are wikilinks to the id: `[[library]]` or `[[library|the library]]`. `scripts/check.sh` fails
  when a link points to a page that does not exist (the help tests in `studyfrog-service`).
- Plain English for users: short sentences, the names the screens use, no internal crate or file names unless the
  user edits that file (`studyfrog.toml`, `_stack.toml`).
- No pictures for now (they would have to be compiled in too).

## Pages

Audience: **new** (first hour with StudyFrog), **all** (every user), **power** (users who edit settings files or write
queries).

### Start
| Id | Title | Audience | Source |
|---|---|---|---|
| `why-studyfrog` | Why StudyFrog | new | `docs/VISION.md`, rewritten for users |
| `getting-started` | Getting started | new | new: vault, stack, card, rehearsal, calendar |
| `privacy` | Privacy and your data | all | `docs/VISION.md` principles, `docs/specs/agent.md`, backups |

### Screens (one how-to per screen)
| Id | Title | Audience | Source |
|---|---|---|---|
| `home` | Home | all | CHANGELOG (STORY-0038, TASK-0139) |
| `library` | Library | all | `docs/specs/library.md` |
| `stack-page` | Stack page | all | `docs/specs/stack-view.md` |
| `item-page` | Item page and editor | all | CHANGELOG (EPIC-021, TASK-0187) |
| `create-dialog` | Creating items | all | CHANGELOG (TASK-0185 to TASK-0187) |
| `rehearse` | Rehearsing | all | `docs/specs/rehearsal-run.md`, `docs/specs/scoring.md` |
| `exam-mode` | Exam mode | all | `docs/specs/exam-mode.md` |
| `calendar` | Calendar | all | CHANGELOG (TASK-0146 to TASK-0148) |
| `board` | Board | all | CHANGELOG |
| `reports` | Reports | all | `docs/specs/reports.md` |
| `achievements` | Achievements | all | CHANGELOG |
| `assistant` | Assistant | all | `docs/specs/agent.md` |
| `graph` | Graph | all | CHANGELOG |
| `pdf` | PDFs | all | `docs/specs/pdf.md` |
| `pdf-export` | Export to PDF | all | `docs/specs/pdf-export.md` |
| `settings` | Settings | all | `docs/specs/config-files.md`, CHANGELOG |
| `import-export` | Import and export | all | `docs/specs/interchange.md` |
| `prompts` | Prompts for an AI | all | `docs/prompts/` |
| `import-formats` | Import formats | all | the importer table in `xfer_adapters.rs` |
| `notifications` | Notifications | all | CHANGELOG (TASK-0126, TASK-0127) |
| `help` | Using help | all | this story: browsing, searching, own pages, tutorials |

### Reference
| Id | Title | Audience | Source |
|---|---|---|---|
| `faq` | Frequently asked questions | all | STATUS.md known limits, CHANGELOG |
| `glossary` | Glossary | all | `docs/GLOSSARY.md`, without the development terms |
| `keyboard` | Keyboard shortcuts | all | the shortcuts of the old help menu, the run keys |
| `query-language` | The query language | power | `docs/specs/query.md` |
| `familiarity` | Familiarity and scheduling | power | `docs/specs/familiarity.md`, `docs/specs/srs-algorithm.md` |
| `statuses` | Item statuses | all | `docs/specs/status.md` |

## Tutorials
Tutorials are step lists in the app, not pages: each step opens the screen it describes and is done when the user has
done it (TASK-0166). The help page `help` describes how to start one again.

| Id | Title | Steps |
|---|---|---|
| `first-steps` | First steps | create a stack, create a card, rehearse it, plan a session in the calendar |

The first-run tutorial (`first-steps`) is offered once after the first vault opens and can be skipped; it can be
repeated from the help screen.

## Help buttons
The `?` in the top bar opens the page of the screen that is showing (`F1` too):
Home `home`, Library `library` (or `stack-page` while a stack is open), Rehearse `rehearse`, Settings `settings`,
Editor `item-page`, Graph `graph`, Calendar `calendar`, Board `board`, Achievements `achievements`, Reports `reports`,
Assistant `assistant`, PDF `pdf`, the vault chooser `getting-started`.
