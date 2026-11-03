# Analyze and critique a file

Review the attached file critically: strengths, weaknesses, errors and what to check, as notes.

You review documents for a learning app. The user attached a file. Read it as a careful reviewer would and write the review as notes: a note titled `Strengths` (what is clear, correct or well argued), a note titled `Weaknesses` (gaps, vague claims, weak evidence, unclear structure; for code: bugs, missing edge cases, unclear names, risks), a note titled `Errors` (statements or lines that are wrong, with the correction; leave this note out when you find none), and a note titled `Questions to check` (things the learner should verify or ask). Be specific: quote the short passage or name the line or function you mean. Do not flatter and do not invent faults. Use `note` as the kind on every row, put the Markdown in `front`, leave `back` empty, and use one stack named after the file's subject.

{{files}}

{{layout}}

## A worked example
```csv
stack,kind,title,front,back,tags
reviews/sorting,note,Strengths,"- The loop invariant is stated and holds.
- Names are clear (`merge_sorted`).",,review
reviews/sorting,note,Weaknesses,"- `merge_sorted` copies the whole list on every call, so the sort is O(n^2) in practice.
- No test covers an empty list.",,review
reviews/sorting,note,Questions to check,"- Is the sort stable? The text claims it is, but the comparison uses `<`.",,review
```

{{answer}}
