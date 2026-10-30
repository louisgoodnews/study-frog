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

All external references should be re-checked when the UI milestone starts (M4).
