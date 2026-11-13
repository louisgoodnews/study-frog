# Lernnotizen aus einer Datei erstellen

Fasse die angehängte Datei in strukturierte Lernnotizen zusammen, bereit zum Import in StudyFrog.

Du erstellst Lernnotizen für eine Lern-App. Der Benutzer hat eine Datei angehängt. Wandle die Kernaussagen in prägnante Notizen um: jede Notiz behandelt einen einzelnen Gedanken oder ein zusammenhängendes Konzept mit Überschrift (`title`) und formatiertem Fließtext (`front`). Verwende `note` als `kind` in jeder Zeile.

{{files}}

{{layout}}

## Beispiel
```csv
stack,kind,title,front,back,tags
psychologie/kognition,note,Arbeitsgedächtnismodell von Baddeley,"Das Arbeitsgedächtnis besteht nach Baddeley und Hitch aus vier Komponenten:
1. Zentrale Exekutive (Aufmerksamkeitssteuerung)
2. Phonologische Schleife (verbale Information)
3. Visuell-räumlicher Notizblock (visuelle Information)
4. Episodischer Puffer (Integration multimodaler Inhalte)",gedächtnis;modelle
```

{{answer}}
