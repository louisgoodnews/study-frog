## The layout of your answer
Answer with a CSV file: comma-separated text, one row per item, UTF-8 without a byte order mark, one line break between rows, no blank lines and no comments. The first row is exactly this header:

stack,kind,title,front,back,tags

| Column | What it holds | Required |
|---|---|---|
| stack | The folder of the stack the item goes into, such as `biology` or `biology/cells`: lower case words, `/` between levels, no leading `/`, no `..`. Repeat it on every row. | yes |
| kind | Exactly `card`, `question` or `note`. | yes |
| title | A short name. A note's heading. May be empty for cards and questions. | notes: yes |
| front | A card's question or term. A question's prompt. A note's whole text, in Markdown. | yes |
| back | A card's answer. A question's model answer. Empty for a note. | cards and questions: yes |
| tags | Tags separated by semicolons, such as `exam;chapter-3`. May be empty. | no |

How to write the cells:
- Put a cell in double quotes when it contains a comma, a double quote or a line break. A cell without those needs no quotes.
- Write a double quote inside a quoted cell as two double quotes (`""`).
- A line break is allowed only inside a quoted cell. Markdown (lists, `**bold**`, code in backticks) is fine in `front` and `back`.
- Every row has exactly six cells, so a row with an empty `title` and `tags` still has its commas.
- Never leave `front` empty, and never leave `back` empty for a card or a question.
