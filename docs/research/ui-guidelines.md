# UI guidelines

| Decision | Rationale | Source |
|---|---|---|
| Keyboard-first rehearsal (space flips, 1-4 rates) | Reduces motor time; latency measurement stays about memory, not mouse travel. | Fitts (1954), *J. Experimental Psychology* 47(6); Hick (1952). |
| Few, consistent controls per screen | Fewer choices, faster decisions. | Hick's law; Nielsen's usability heuristics (1994), nngroup.com. |
| Visible system status (queue progress, timer) | Heuristic 1 of Nielsen's ten. | Nielsen (1994). |
| Undo for destructive actions (delete card, move task) | Heuristic 3/5: user control, error prevention. | Nielsen (1994). |
| Contrast ratio at least 4.5:1 body text; targets at least 24x24 px | Accessibility. | W3C WCAG 2.2, SC 1.4.3 and 2.5.8. |
| Respect system light/dark and font scaling | Platform conformity. | GNOME Human Interface Guidelines (developer.gnome.org/hig). |
| Avoid layout shift and per-frame allocation in render loop | Perceived performance; ADR 0002. | Project constraint P7. |
| Distraction-free review screen | Retrieval needs focus; show only prompt and timer. | Project judgement; validate in user tests. |

Verification (2026-10-30): WCAG 2.2 SC 1.4.3 (AA, 4.5:1 normal text, 3:1 large text) and SC 2.5.8
(AA, 24x24 CSS px, with exceptions) confirmed on w3.org. Fitts (1954), Hick (1952), Nielsen (1994)
and the GNOME HIG are from memory and unverified; re-check when the UI milestone starts (M4).
Note: 24 px is the WCAG *minimum*; prefer larger targets for primary rehearsal controls.
