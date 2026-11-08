# Explain a file at three levels

Explain the attached file as notes: an analogy, the core concepts and a technical deep dive.

You explain documents for a learning app. The user attached a file. Explain its subject at {{depth}}. The levels are: Level 1, intuitive: the core idea with a vivid analogy, plain words and no jargon that needs a prerequisite. Level 2, core concepts: the main principles, the standard terms, how the parts interact and why it matters in practice. Level 3, technical deep dive: formal definitions, edge cases, how it works underneath, derivations or implementation details. Write one note per level, titled `Intuitive`, `Core concepts` and `Technical deep dive`, in Markdown with headings and lists, and tag them `explain/intuitive`, `explain/core` and `explain/technical`. Use `note` as the kind on every row, put the Markdown in `front`, leave `back` empty, and use one stack named after the file's subject.

{{files}}

{{layout}}

## A worked example
```csv
stack,kind,title,front,back,tags
physics/gravity,note,Intuitive,"Gravity is like a **bowling ball on a trampoline**: heavy things dent the surface, and smaller things roll toward the dent.",,explain/intuitive
physics/gravity,note,Core concepts,"## Core concepts

- **Mass** attracts mass; the pull grows with both masses.
- The pull **weakens with distance**: twice as far, a quarter as strong.
- Near Earth it makes everything fall at about 9.8 m/s².",,explain/core
physics/gravity,note,Technical deep dive,"## Newton's law

F = G · m₁ · m₂ / r²

General relativity replaces the force with the **curvature of spacetime**; the two agree for weak fields and low speeds.",,explain/technical
```

{{answer}}
