---
id: 01HZY000000000000000000001
kind: note
title: Complete PDF Export Showcase
tags: [demo, pdf, export]
---

# Complete PDF Export Showcase

A comprehensive demonstration of every formatting element supported by StudyFrog's Markdown-to-PDF export engine.

[TOC]

## Typography and Text Styling

StudyFrog supports standard Markdown formatting alongside selective rich-text extensions:

- **Bold typography** for key terminology and strong emphasis
- *Italic styling* for definitions and foreign expressions
- ***Bold and italic combined*** for primary assertions
- Inline `code spans` with dedicated background tinting
- <u>Underlined text passages</u> using HTML underline tags
- Strikethrough text with ~~double tildes~~ for superseded notes

### The Six Print-Safe Palette Hues

The six curated palette colors ensure clear contrast in both light and dark reading environments:

- <span style="color:red">Red text</span> denotes critical warnings, failure modes, and security notices.
- <span style="color:orange">Orange text</span> highlights cautionary remarks and impending deadlines.
- <span style="color:green">Green text</span> marks verified observations and positive review results.
- <span style="color:blue">Blue text</span> identifies structural references and informational notes.
- <span style="color:purple">Purple text</span> emphasizes conceptual theories and synthetic insights.
- <span style="color:grey">Grey text</span> formats peripheral annotations and metadata comments.

## Headings and Document Structure

Headings from level 1 down to level 4 are structured hierarchically in the text flow and emitted into PDF document outlines (bookmarks) for easy sidebar navigation.

### Major Subsection

Subsection content is neatly separated with typographic rhythm.

#### Minor Technical Detail

Fine-grained sectioning for deep reference documentation and technical outlines.

## Lists and Task Tracking

### Unordered Bullet Points
- Fundamental concept of memory consolidation
- Dual-process theory of cognition
  - System 1 fast heuristics
  - System 2 deliberate analytical thought

### Sequential Numbered Steps
1. Ingest Markdown source documents into vault
2. Schedule spaced rehearsal intervals with FSRS-6
3. Generate publication-ready PDF summaries

### Interactive Task Checklists
- [x] Establish foundational vault taxonomy
- [x] Validate cross-reference link graphs
- [ ] Conduct comprehensive end-of-term knowledge review

## Blockquotes and Citations

> "Active recall and spaced repetition represent the single most robust finding in modern cognitive psychology for long-term retention of structured knowledge."
> — Principles of Human Memory

## Tables with Column Alignment

Pipe tables support headers, cell typography, and column alignments:

| Domain | Status | Retention | Priority |
| :--- | :---: | ---: | :--- |
| Molecular Genetics | Active | 92.4% | High |
| Discrete Structures | In Review | 84.1% | Urgent |
| Quantum Mechanics | In Progress | 78.5% | Normal |
| Cognitive Science | Mastered | 98.0% | Low |

## Syntax-Highlighted Code Blocks

Code blocks feature token classification with distinct color highlighting for keywords, strings, comments, and numbers:

```rust
// Compute memory stability update
fn compute_stability(difficulty: f64, retrievability: f64) -> f64 {
    let factor = 1.0 + (10.0 - difficulty) * 0.1;
    (factor / (1.0 - retrievability)).clamp(0.1, 100.0)
}
```

```python
def spaced_intervals(stability: float, desired_retention: float = 0.9) -> int:
    """Calculate target rehearsal interval in days."""
    import math
    factor = 9.0 * (1.0 / desired_retention - 1.0)
    return max(1, round(stability * math.log(desired_retention) / math.log(0.9)))
```

## Links and Cross References

Clickable hyperlinks are converted into native PDF link annotations:

- External web link: [StudyFrog Source Repository](https://github.com/studyfrog/studyfrog)
- Internal wikilink reference: [[Cognitive Architecture Overview]]

## Image Attachments

Embedded images and figures from the vault are rendered with visual borders and captions:

![Cellular Biology Overview Diagram](assets/cellular_biology_overview.png)

---

End of Complete PDF Export Showcase Note.
