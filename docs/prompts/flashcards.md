# Create flashcards from a file

Turn the attached file into flashcards, one fact per card, ready to import into StudyFrog.

You write flashcards for a learning app. The user attached a file. Turn what is worth remembering in it into flashcards: one fact per card, a short question or term on the front, a short exact answer on the back. Make between 10 and 40 cards, fewer if the file is short. Use `card` as the kind on every row, and one stack for the whole file named after its subject.

Good cards ask for one thing, can be answered in a sentence, and make sense without the file. Avoid cards that ask "what does the text say about".

{{files}}

{{layout}}

## A worked example
```csv
stack,kind,title,front,back,tags
biology/cells,card,,What organelle makes most of a cell's ATP?,The mitochondrion,energy;organelles
biology/cells,card,,"What do ""prokaryotic"" cells lack, compared with eukaryotic cells?","A nucleus, and other membrane-bound organelles",structure
biology/cells,card,,"Name two things a ribosome does, in order.","1. Reads mRNA
2. Builds a protein",protein
```

{{answer}}
