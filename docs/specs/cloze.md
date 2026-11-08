# Cloze deletions (STORY-0108)

A card's front or a note's body can hide words as gaps: `{{c1::answer}}` or `{{c1::answer::hint}}`.
The file keeps the syntax as written (plain Markdown, lossless round trip); everything else is done
when a run shows the item. Parser and transforms: `studyfrog_md::cloze` (pure).

## Syntax
- `{{c<N>::<answer>}}` or `{{c<N>::<answer>::<hint>}}`, `N` from 1 to 99, on one line, answer not blank.
  Gaps with the same number form a group. Anything else that looks similar (`{{c0::..}}`, `{{c100::..}}`,
  no closing braces on the line, an empty answer) stays ordinary text.
- Gaps inside fenced code are recognised too (programming syntax); they are shown without code marks.

## Rehearsal
- One group is asked at a time. A card or note with gaps asks the group at position
  `reviews % groups` (its earlier reviews), so the groups take turns from review to review and share one
  schedule. A card with `cloze_group: N` in its front matter always asks group `N` (to give a group its
  own card and schedule, copy the card and give each copy its number).
- Before the answer: the gaps of the asked group show as `[...]`, or `[hint]` when the gap has a hint,
  set off as code; the other gaps show their words.
- After the answer (flip, typed answer or exam result): the text with the asked group's words in bold
  green, followed by the card's back (extra information) when it has one.
- A typed answer (Type answers, Exam) is compared with the asked group's words joined by `, `; the
  window passes the words it showed, so a re-shown card is compared with what was on screen.
- The note shown in a run with gaps: its body is the prompt (masked) and the answer (revealed).

## Authoring
- The Markdown toolbar has a `[...]` button; Ctrl+Shift+C wraps the selection (or a placeholder
  `answer`) in a gap with the next number, Ctrl+Shift+Alt+C with the number used last.
- The editor's preview shows every gap's words in green.
- A new card whose front has gaps needs no back; its default title shows every gap as `[...]`.

## Imports
- CSV and other text imports keep `{{c1::...}}` as written, so the cards rehearse as clozes.
- Anki cloze notes keep becoming one card per gap number with the gap already hidden (the review log of
  each Anki card maps onto its own item), as before.
