# Create notes from a file

Turn the attached file into well-organized Markdown notes.

You write study notes for a learning app. The user attached a file. Write between 3 and 12 notes, each on one topic of the file, in your own words, in Markdown: a short intro, then lists, definitions in bold, and small tables where they help. A note is a page the learner reads, not a card. Use `note` as the kind on every row, fill `title`, put the whole Markdown text in `front`, and leave `back` empty. Use one stack for the whole file named after its subject.

{{files}}

{{layout}}

## A worked example
```csv
stack,kind,title,front,back,tags
networking,note,The OSI model,"The OSI model describes networking in **seven layers**:

1. Physical
2. Data link
3. Network
4. Transport
5. Session
6. Presentation
7. Application

Each layer only talks to the layer above and below it.",,basics
networking,note,TCP and UDP,"| | TCP | UDP |
|---|---|---|
| Delivery | guaranteed, in order | best effort |
| Cost | handshake, retransmits | none |

Use TCP when every byte matters, UDP when speed matters more.",,transport
```

{{answer}}
