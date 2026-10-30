# Glossary

| Term | Meaning |
|---|---|
| **Vault** | The root folder holding all Markdown content. |
| **Stack** | A logical container of Items (a deck/notebook). Has its own settings and achievements. |
| **Item** | Anything inside a stack that can be studied: a Note, Card or Question. |
| **Note** | Free-form Markdown document; may link to other notes with `[[wikilinks]]`. Its title can be rehearsed. |
| **Card** | Flashcard with a front and back. Reviewed by flipping. |
| **Question** | Prompt with a reference answer, answered in free text and scored by the SLM. |
| **Front-matter** | YAML block at the top of a file holding metadata. See [markdown-schema](specs/markdown-schema.md). |
| **Review** | One recorded recall attempt of an Item with a rating; input to scheduling. |
| **Rating** | Again / Hard / Good / Easy (SRS input). |
| **Run** | A timed rehearsal session over a queue of Items. |
| **Run event** | Timestamped record inside a Run (shown, flipped, skipped, answered). |
| **Latency** | Time-to-skip, time-to-flip or time-to-answer for an Item in a Run. |
| **SRS** | Spaced-repetition scheduling; computes the next due time. |
| **Stability / Difficulty / Retrievability** | FSRS memory-model variables. |
| **SLM** | Local small language model used to compare user input to reference text. |
| **Score** | Similarity result in [0, 1] from the SLM comparison. See [scoring](specs/scoring.md). |
| **Session** | A planned calendar block for learning, linked to a Stack. |
| **Board** | Kanban/scrum board of Tasks with columns and WIP limits. |
| **Achievement** | Unlockable milestone, stack-scoped or global. |
| **Epic / Story / Task / Subtask / Bug** | Work-item hierarchy, see [issues](issues/README.md). |
| **Phase (M0..M8)** | Roadmap milestone with explicit exit criteria. |
