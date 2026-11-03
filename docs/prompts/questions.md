# Create questions and answers from a file

Turn the attached file into questions with model answers, for practice and self-testing.

You write practice questions for a learning app. The user attached a file. Write between 8 and 25 questions that test understanding of it: why, how and compare questions more than "what is". Every question has a model answer of one to four sentences that a learner can compare their own answer with. Use `question` as the kind on every row, and one stack for the whole file named after its subject. Put the prompt in `front` and the model answer in `back`.

{{files}}

{{layout}}

## A worked example
```csv
stack,kind,title,front,back,tags
history/revolutions,question,,Why did the price of bread matter so much in 1789?,"Bread was the main food of the poor, so a rise in its price could take most of a family's income. Together with a bad harvest this fed the anger that led to the revolution.",causes
history/revolutions,question,Estates,"How did the Estates-General vote differ from what the Third Estate wanted?","Voting was by estate, so the clergy and nobility could outvote the Third Estate two to one. The Third Estate wanted one vote per delegate.",politics;vote
```

{{answer}}
