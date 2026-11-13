## Das Format deiner Antwort
Antworte mit einer CSV-Datei: kommagetrennter Text, eine Zeile pro Eintrag, UTF-8 ohne BOM, ein Zeilenumbruch zwischen den Zeilen, keine Leerzeilen und keine Kommentare. Die erste Zeile ist exakt dieser Header:

stack,kind,title,front,back,tags

| Spalte | Inhalt | Erforderlich |
|---|---|---|
| stack | Der Ordner des Stacks (z.B. `biologie` oder `biologie/zellen`): Kleinbuchstaben, `/` zwischen Ebenen, kein führender `/`, kein `..`. In jeder Zeile wiederholen. | ja |
| kind | Exakt `card`, `question` oder `note`. | ja |
| title | Ein kurzer Name bzw. Notiz-Überschrift. Darf bei Karten und Fragen leer sein. | bei Notizen: ja |
| front | Frage oder Begriff bei Karten; Aufgabenstellung bei Fragen; vollständiger Notiztext in Markdown. | ja |
| back | Antwort bei Karten; Musterantwort bei Fragen; leer bei Notizen. | bei Karten & Fragen: ja |
| tags | Durch Semikolons getrennte Tags (z.B. `klausur;kapitel-3`). Darf leer sein. | nein |

Regeln für die Zellen:
- Setze eine Zelle in doppelte Anführungszeichen, wenn sie ein Komma, ein Anführungszeichen oder einen Zeilenumbruch enthält.
- Schreibe ein Anführungszeichen innerhalb einer gequoteten Zelle als zwei doppelte Anführungszeichen (`""`).
- Zeilenumbrüche sind nur innerhalb gequoteter Zellen erlaubt. Markdown (Listen, `**fett**`, Code in Backticks) ist in `front` und `back` erwünscht.
- Jede Zeile hat exakt sechs Zellen.
- Lasse niemals `front` leer, und niemals `back` bei Karten oder Fragen.
