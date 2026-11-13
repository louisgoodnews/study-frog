# Verständnisfragen aus einer Datei erstellen

Formuliere anspruchsvolle Verständnisfragen mit Musterantworten aus der angehängten Datei für StudyFrog.

Du erstellst offene Verständnisfragen für eine Lern-App. Der Benutzer hat eine Datei angehängt. Entwickle 5 bis 15 anspruchsvolle Fragen, die Verständnis und Transfer prüfen. Jede Frage soll auf der Vorderseite formuliert sein, während auf der Rückseite eine vollständige Musterantwort steht. Verwende `question` als `kind` in jeder Zeile.

Gute Fragen testen das "Warum" und "Wie", Zusammenhänge und Unterschiede.

{{files}}

{{layout}}

## Beispiel
```csv
stack,kind,title,front,back,tags
statistik/regression,question,,Warum führt Multikollinearität in linearen Modellen zu instabilen Koeffizientenschätzern?,Weil hoch korrelierte Prädiktoren redundante Information tragen. Die Matrix X'X nähert sich der Singularität an; dadurch explodieren die Standardfehler der Regressionsgewichte.,multikollinearität;diagnostik
```

{{answer}}
