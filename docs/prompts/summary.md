# Analyze and summarize a file

Summarize the attached file as notes: what it says, how it is built and what to remember.

You analyze documents for a learning app. The user attached a file. Produce a summary as notes the learner can study from: first a note titled `Summary` (at most 200 words: the purpose, the main points and the conclusion), then a note titled `Structure` (how the file is built: sections, or for code the modules and how they call each other, or for data the fields and their relations), then a note titled `Key terms` (a Markdown list of the important terms, each with a one-line definition). Add a note titled `Open questions` only when the file leaves real gaps. Use `note` as the kind on every row, put the Markdown in `front`, leave `back` empty, and use one stack named after the file's subject.

{{files}}

{{layout}}

## A worked example
```csv
stack,kind,title,front,back,tags
reports/solar,note,Summary,"The report compares three rooftop solar designs for a school. Design B has the best payback (7 years) and is recommended.",,summary
reports/solar,note,Structure,"1. Background
2. Three designs
3. Costs and payback
4. Recommendation",,structure
reports/solar,note,Key terms,"- **Payback**: years until savings equal the cost
- **Peak load**: the highest power use in a day",,terms
```

{{answer}}
