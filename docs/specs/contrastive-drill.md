# Contrastive drill (STORY-0111)

Two to four items that get confused, studied side by side. Service: `studyfrog_service::contrast`
(`load_contrast`, `save_contrast`); window: `crates/studyfrog-ui/src/contrast.rs`, `view/contrast.rs`.

## Starting it
- Reports > Leeches (and a stack's Reports tab): every pair of items that get confused has
  "Compare A and B side by side" next to "Drill A and B together" (the practice run of STORY-0104).
- The Library: with 2 to 4 rows selected, the bulk bar offers "Contrastive drill".
- Fewer than 2 or more than 4 items are refused with a notice.

## The drill
1. **Match.** The prompts stand side by side, one column per item; under them the answers in a shuffled
   order labelled A to D, never one under its own prompt. The learner picks each prompt's answer and
   presses "Check the matches"; each column says Right or Not this one. Picks lock once checked.
2. **Compare.** Each column then shows the item's title, prompt and answer, and "Only here:" the words of
   its answer (longer than three letters) that no other answer has.
3. **Keep the distinction.** "What is the key difference that distinguishes them?" and a choice:
   - **Add it as a hint to each item**: the answer (card back, question reference or note body) ends
     with `> Distinction (vs [[Other]], ...): <text on one line>`. A later save replaces the hint (it is
     always the last thing in the answer). Typed answers are compared without it.
   - **Create or update a comparison note**: a note "Comparison: A vs B" in the first item's folder,
     with `compares: "<ULIDs sorted>"` in its front matter, the distinction, and a section per item
     (`## [[Title]]`, what it asks and answers). Saving the same items again (in any order) rewrites
     that note instead of making another. The links make the items list the note as a backlink.
   Either save can be undone from the Library ("Undo last change").
- The drill never changes a schedule; it records no reviews.
