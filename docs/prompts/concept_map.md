# Map the concepts of a file

Find what you must know first, the mental models and the concepts that are easily mixed up, as linked notes.

You analyze documents for a learning app. The user attached a file. Write the conceptual structure as notes: a note titled `Prerequisites` (the foundational ideas, maths or tools the reader must understand before the file makes sense, each with one line on why), a note titled `Core mental models` (the basic abstractions or metaphors that explain why the subject works this way), and a note titled `Contrast matrix` (a Markdown table that sets easily confused concepts side by side: the concept, the criterion that tells them apart and the typical misconception). Write every reference to a related term as a wikilink such as `[[Concept name]]`, so the notes join the vault graph. Tag them `prerequisite`, `mental-model` and `contrast`. Use `note` as the kind on every row, put the Markdown in `front`, leave `back` empty, and use one stack named after the file's subject.

{{files}}

{{layout}}

## A worked example
```csv
stack,kind,title,front,back,tags
concepts/networking,note,Prerequisites,"- [[Binary numbers]]: addresses are bit patterns.
- [[Client and server]]: every exchange has two sides.",,prerequisite
concepts/networking,note,Core mental models,"- **Layers**: each [[Protocol layer]] hides the one below it.
- **Packets**: data travels in small, independent pieces.",,mental-model
concepts/networking,note,Contrast matrix,"| Concept | Tells it apart | Typical misconception |
|---|---|---|
| [[TCP]] vs [[UDP]] | TCP guarantees delivery and order | UDP is always faster in practice |
| [[Router]] vs [[Switch]] | A router joins networks, a switch joins devices | They do the same job |",,contrast
```

{{answer}}
