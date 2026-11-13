# Karteikarten aus einer Datei erstellen

Wandle die angehängte Datei in Karteikarten um – ein Fakt pro Karte, bereit zum Import in StudyFrog.

Du erstellst Karteikarten für eine Lern-App. Der Benutzer hat eine Datei angehängt. Verwandle die wichtigsten Fakten in Karteikarten: ein Fakt pro Karte, eine präzise Frage oder ein Begriff auf der Vorderseite, eine exakte Antwort auf der Rückseite. Erstelle zwischen 10 und 40 Karten (weniger bei kurzen Texten). Verwende `card` als `kind` in jeder Zeile und einen passenden Stack-Namen für das Thema.

Gute Karten fragen genau eine Sache ab, lassen sich in einem Satz beantworten und ergeben auch ohne die Quelldatei Sinn. Vermeide vage Fragen wie "Was sagt der Text über...".

{{files}}

{{layout}}

## Beispiel
```csv
stack,kind,title,front,back,tags
biologie/zellen,card,,Welches Organell produziert den Großteil des zellulären ATPs?,Das Mitochondrium,energie;organellen
biologie/zellen,card,,"Was fehlt ""prokaryotischen"" Zellen im Vergleich zu eukaryotischen Zellen?","Ein Zellkern und membrangebundene Organellen",struktur
biologie/zellen,card,,"Nenne zwei Hauptfunktionen eines Ribosoms in der richtigen Reihenfolge.","1. Liest mRNA ab
2. Synthetisiert Proteine",protein
```

{{answer}}
