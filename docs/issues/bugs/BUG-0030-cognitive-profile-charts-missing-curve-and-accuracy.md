---
id: BUG-0030
type: bug
title: "The Cognitive profile shows a velocity number, not a curve, and no semantic accuracy trend"
status: done
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
- [x] The Cognitive profile shows learning velocity as a curve of the rate over time, not only as one number
- [x] The Cognitive profile shows a semantic accuracy trend over time, or the story is changed to say it is not shown until semantic scores exist
- [x] The latency chart shows a distribution of answer times, not only the scatter against word count

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

Fix (2026-11-23): the Cognitive profile shows (1) the learning velocity as a curve: passing answers per minute of answering after each timed answer, against cumulative answering minutes (the x axis is study time, so a short vault still draws a line; the figure is the curve's last point); (2) the answer times as a distribution in 2-second bins; (3) a semantic accuracy trend: the running mean of the judged answers' scores by answer count, or the words "Not shown yet: no answer has a semantic score" when there are none. Code: `studyfrog-service::cognitive_curves` (pure), `studyfrog-service::cognitive_report::profile` (takes the telemetry too), `studyfrog-ui::view::reports_cognitive` and `reports_cognitive_charts::{curve, bars}`. Tests: `studyfrog-service/tests/cognitive_curves.rs`, `studyfrog-ui/tests/reports_cognitive_ui.rs`. Semantic source: the built-in word matcher (no model installed here). Verified in the running app (display :92; 10 cards; 5 spaced flip answers and 5 spaced typed answers): at 1280x800 `/tmp/claude-0/cog/05-cognitive-1280-top.png` (velocity curve), `/tmp/claude-0/cog/06-cognitive-1280-mid.png` (answer-time bars, semantic trend), `/tmp/claude-0/cog/07-cognitive-1280-low.png`; at 960x640 `/tmp/claude-0/cog/11-cognitive-960-top.png`, `/tmp/claude-0/cog/09-cognitive-960-velocity.png`, `/tmp/claude-0/cog/10-cognitive-960-semantic.png`. Before: `/tmp/claude-0/review-c/shots/02-cognitive.png` (one number, no curve, no trend). The semantic trend reads the rows of `review_telemetry` written by BUG-0031's fix.

## Open Questions
- None.
