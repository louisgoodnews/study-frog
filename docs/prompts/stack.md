# Create a whole stack from a file

Turn the attached file into a stack: an overview note, sub-stacks by topic, and cards and questions in each.

You build a study stack for a learning app. The user attached a file. Organize its subject into a stack with sub-stacks, one per main topic (between 2 and 8), using folder names such as `chemistry`, `chemistry/bonds`, `chemistry/reactions`. Give the stack:
- one `note` as an overview in `chemistry` (kind `note`, a title, the overview as Markdown in `front`),
- for each sub-stack: one `note` that summarizes the topic, and between 5 and 15 `card` rows and a few `question` rows.

{{files}}

{{layout}}

## A worked example
```csv
stack,kind,title,front,back,tags
chemistry,note,Overview,"# Chemistry basics

- **Bonds** hold atoms together.
- **Reactions** rearrange them.",,overview
chemistry/bonds,note,Bonds,"Atoms bond by sharing (covalent) or transferring (ionic) electrons.",,bonds
chemistry/bonds,card,,What is a covalent bond?,A bond made by two atoms sharing electrons,bonds
chemistry/bonds,question,,Why is table salt brittle?,"Its ions sit in a rigid lattice; a shift lines up like charges, which repel and split the crystal.",bonds;ionic
chemistry/reactions,card,,What is conserved in every chemical reaction?,"Mass, because atoms are only rearranged",reactions
```

{{answer}}
