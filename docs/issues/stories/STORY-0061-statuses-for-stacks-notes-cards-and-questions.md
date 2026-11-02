---
id: STORY-0061
type: story
title: Statuses for stacks, notes, cards and questions
status: backlog
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, core, md, ui]
acceptance_criteria:
  - "Stacks, notes, cards and questions have a status with a plain-language name, stored in the Markdown front-matter"
  - "A status can be changed on the item page and in lists, and the available statuses and their meaning are documented"
  - "A stack's status summarises its items"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Give every item a status like an issue in Jira, so the user can see and set where it stands: not started, being learned, being reviewed, complete.

## Acceptance Criteria
- [ ] Stacks, notes, cards and questions have a status with a plain-language name, stored in the Markdown front-matter
- [ ] A status can be changed on the item page and in lists, and the available statuses and their meaning are documented
- [ ] A stack's status summarises its items

## Design Notes
Proposed set, named for what it says: `new` shown as 'Not studied yet' (never rehearsed), `learning` as 'Learning' (rehearsed, still unfamiliar), `reviewing` as 'In review' (known, kept fresh by spaced repetition), `done` as 'Complete' (the user is finished with it; no longer scheduled), plus `paused` as 'On hold' (kept, not scheduled). 'Not studied yet' is derived while an item has no review; the first rehearsal moves it to 'Learning' on its own; the others are set by the user, with the familiarity score (STORY-0058) offering a suggestion to move on. Keep the stored values English and stable and translate only the labels. A stack's status is its own field with a roll-up shown beside it (how many items are in each status). Board columns (EPIC-011) stay free-form; whether a board column can be bound to a status is left open.

Tasks: TASK-0143, TASK-0144, TASK-0145.

## Dependencies
Familiarity-based suggestions need STORY-0058; everything else stands alone.

## Open Questions
1) Is 'Complete' set by hand only, or may it be proposed when familiarity stays high? 2) Should the board columns of an item board be tied to statuses? Answered by the owner on 2026-11-02: the five names are accepted.
