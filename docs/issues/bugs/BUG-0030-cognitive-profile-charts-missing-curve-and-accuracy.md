---
id: BUG-0030
type: bug
title: "The Cognitive profile shows a velocity number, not a curve, and no semantic accuracy trend"
status: backlog
estimate: 2
labels: [m14, ui, report]
epic: EPIC-060
parents: ['EPIC-060']
acceptance_criteria:
  - "The Cognitive profile shows learning velocity as a curve of the rate over time, not only as one number"
  - "The Cognitive profile shows a semantic accuracy trend over time, or the story is changed to say it is not shown until semantic scores exist"
  - "The latency chart shows a distribution of answer times, not only the scatter against word count"
---

## Context
Found while auditing STORY-0178 / EPIC-060 in the running app (2026-11-22, display :92). The Cognitive profile tab (`/tmp/claude-0/audit/g2/02-cognitive.png`, `/tmp/claude-0/audit/g2/04-cognitive-bottom.png`) shows: learning velocity as one line of text (61.15 passing answers per minute), a latency scatter against word count, a cumulative recall share by week, and days between passing reviews by week. The story's acceptance criteria name a learning velocity curve, a latency distribution and a semantic accuracy trend. The implementation notes say semantic scores are not produced yet.

## Description
Show the velocity as a curve over the review history, add a latency distribution, and show a semantic accuracy trend once scores exist (see BUG-0031). Until then, state on the tab which of these are not shown.

## Acceptance Criteria
- [ ] The Cognitive profile shows learning velocity as a curve of the rate over time, not only as one number
- [ ] The Cognitive profile shows a semantic accuracy trend over time, or the story is changed to say it is not shown until semantic scores exist
- [ ] The latency chart shows a distribution of answer times, not only the scatter against word count

## Design Notes
- None.

## Dependencies
- STORY-0178 (criterion 2), EPIC-060

## Verification (2026-11-22)
Reproduced in the running app (`/tmp/claude-0/verifyB`, 1280x800), vault with 7 answered reviews. Reports > Cognitive profile:
- Velocity: one line of text, "543.44 passing answers per minute of answering" (`/tmp/claude-0/verifyB/r02-cognitive.png`). No curve. Criterion 1 not met.
- No semantic accuracy trend, and no line on the tab saying one is not shown until scores exist (`/tmp/claude-0/verifyB/r02-cognitive.png`, `/tmp/claude-0/verifyB/r03-cognitive-bottom.png`; `view/reports_cognitive.rs` has no semantic text). Criterion 2 not met.
- Latency: a scatter of answer time against word count, no distribution (`/tmp/claude-0/verifyB/r02-cognitive.png`). Criterion 3 not met.
Status stays backlog.

Review (2026-11-22): NOT FIXED. Criterion 1: velocity is still one line of text (`/tmp/claude-0/review-c/shots/02-cognitive.png`). Criterion 2: no semantic accuracy trend and no note that one is not shown (`/tmp/claude-0/review-c/shots/03-cognitive-bottom.png`). Criterion 3: the latency view is still a scatter against word count (`02-cognitive.png`). Stays backlog.

## Open Questions
- None.
