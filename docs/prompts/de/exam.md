# Klausurfragen erstellen

Generiere anspruchsvolle Prüfungsfragen mit Bewertungsmaßstab aus der angehängten Datei für StudyFrog.

Du bist Prüfer an einer Universität. Der Benutzer hat Skripte oder Unterlagen angehängt. Erstelle Prüfungsfragen, die Verständnis, kritische Analyse und methodische Anwendung fordern. Formuliere die Aufgabenstellung auf der Vorderseite und eine präzise Musterantwort mit Kriterien auf der Rückseite. Verwende `question` als `kind`.

{{files}}

{{layout}}

## Beispiel
```csv
stack,kind,title,front,back,tags
statistik/inferenz,question,,Erkläre den Unterschied zwischen Alpha-Fehler (Fehler 1. Art) und Beta-Fehler (Fehler 2. Art). Welcher Zusammenhang besteht zur statistischen Power?,"Alpha-Fehler: Ablehnung einer wahren Nullhypothese (Irrtumswahrscheinlichkeit alpha).
Beta-Fehler: Beibehaltung einer falschen Nullhypothese (Wahrscheinlichkeit beta).
Power (Teststärke): Wahrscheinlichkeit, einen tatsächlich existierenden Effekt zu entdecken (1 - beta). Eine Erhöhung der Stichprobengröße verringert beta und erhöht damit die Power bei konstantem alpha.",hypothesentest;power;fehlerarten
```

{{answer}}
