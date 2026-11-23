---
id: STORY-0192
type: story
title: "Search and tooltips in the control protocol"
status: done
epic: EPIC-064
parents: ['EPIC-064']
estimate: 2
labels: [m14, tooling, ui]
acceptance_criteria:
  - 'search focus focuses the search field of the screen in front and the answer names the focused field'
  - 'help tooltip with a control name answers with the explanation and the shortcut view::tips gives it'
  - 'the state snapshot names the focused field'
---

## Context
Ctrl+F and the slash key (STORY-0188) and the tooltip texts (STORY-0169) are not reachable. An agent cannot tell what a control does or whether a field has the keyboard.

## Description
The focused field is the egui memory's focused id; the protocol reads it on the UI thread.

## Acceptance Criteria
- [x] search focus focuses the search field of the screen in front and the answer names the focused field
- [x] help tooltip with a control name answers with the explanation and the shortcut view::tips gives it
- [x] the state snapshot names the focused field

## Design Notes
None.

## Implementation
- `search --focus [TEXT]` sets `chrome.search_focus`; the library's search fields and the PDF find field take the keyboard on their next frame (`view::search_focus::focus_field` with the request) and report it with `ChromeMsg::SearchFocusTaken(name)`. The answer is `focus: {pending, focused}`; `state` carries `focused_field` and `search_focus_pending`.
- `help tooltip NAME` answers the explanation and text of a control from `view::tips` (`explain_of`); an unknown name lists the names it knows.
- Limit: the focus is confirmed by a frame that draws the field; the protocol tests run without a frame, so `pending` stays true there. Not yet checked in the running app.

Review (2026-11-22): criterion 1 MET on the library query view: `ctl query status=new`, then `ctl search --focus`, answered `focus: {focused: "Query text", pending: false}`, and `state` reports `focused_field: "Query text"`. Limit: on Home and on a stack Summary (no search field of their own) the request stays `pending: true` and the top-bar "Search items" field is not focused (BUG-0049). Criterion 2 MET: `ctl "help tooltip \"Back (Alt+Left)\""` answers the explanation ("Goes back to the screen you were on before.") and the shortcut in the name; "Zen mode (Ctrl+Shift+F)" likewise. Criterion 3 NOT MET, unticked: after `nav home`, `state` still reports `focused_field: "Query text"` with no library screen open (BUG-0050).

## Closed (2026-11-23)
Set to done with EPIC-064, whose criterion 1 is now met (see EPIC-064's Verification); its children are all done.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.

## Verification (2026-11-23)

Checked in the running app (640 by 420 and 1280 by 800, display :101), not headless:
- Criterion 1: on Home, `search --focus` answered `focus: {focused: "Search items", pending: false}` (BUG-0049 fixed); on the library query view it answered `{focused: "Query text", pending: false}`.
- Criterion 2: `help tooltip "Back (Alt+Left)"` answered its explanation and the shortcut in the name; `help tooltip Calendar` (a navigation rail name, added 2026-11-23) answered "Sessions, reminders and the timetable, by day, week or month."; an unknown name lists the names it knows.
- Criterion 3: `state` named `focused_field: "Search items"` on Home and `"Query text"` on the library; after `nav home` it is `null` (BUG-0050 fixed).
- Nothing was missing in the protocol itself; the navigation rail names were added to the tooltip table so `help tooltip` answers them.
- Status stays review, although all three criteria are ticked. The issue rule (`tools/repo-lint`, `check_status`) makes a parent done exactly when all its children are; this is the last child of EPIC-064, whose criterion 1 (every shipped feature drivable through the protocol) is open and not verified here. Story and epic should move to done together once the owner decides EPIC-064's criterion 1.
