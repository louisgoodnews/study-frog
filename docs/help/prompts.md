# Prompts for an AI

Prompts are ready-made instructions for any AI chat (ChatGPT, Gemini, Claude, a local model). They tell the AI exactly
which file to produce, so that StudyFrog can import the answer the first time. You find them on the **Prompts** tab of
the [[assistant|Assistant]] screen, and in the import dialog under **Which files can I import?** (see
[[import-formats]]). Copying a prompt needs no connection to any assistant.

## The six prompts
- **Create flashcards from a file**: one fact per card.
- **Create questions and answers from a file**: practice questions with model answers.
- **Create a whole stack from a file**: an overview note, sub-stacks by topic, with cards and questions.
- **Create notes from a file**: well-organized Markdown notes.
- **Analyze and summarize a file**: a summary, the structure and the key terms.
- **Analyze and critique a file**: strengths, weaknesses, errors and what to check.

## How to use one
1. Open **Assistant**, then **Prompts**, and press **Copy to clipboard** (or **Save as .md** to keep the prompt as a
   file; **Save all prompts to a folder…** saves all six).
2. In your AI chat, paste the prompt and attach your file. Notes, text copied from a PDF, source code (python, r, rust,
   php, java, cpp and so on) and data files (toml, json, html, csv, tsv) all work.
3. The AI answers with one block of CSV. Copy it into a file whose name ends in `.csv`.
4. In StudyFrog open **Import** and choose that file. The preview shows what would be made, and any problem with its
   row and column; **Copy the problems for the AI** gives you a message to send back so the AI fixes exactly those rows.

## Running a prompt with your own assistant
If you connected a model on the **Connections** tab, the lower part of the **Prompts** tab runs a prompt for you:
choose a prompt and a file (a text, source or data file; for a PDF copy its text into a .txt file first), press
**Show what will be sent**, read the text, and press **Send**. Nothing is sent before that, and not to a provider you
did not allow. When the answer arrives, **Import this answer…** opens the import preview. If the preview lists
problems, **Ask the assistant to fix these** prepares a message with exactly those rows; you read it and press Send
again.

If the AI says `PROBLEM:`, the attached file was not enough; attach more or something else.
