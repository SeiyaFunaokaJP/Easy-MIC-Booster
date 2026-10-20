---
title: Funktionen
layout: default
parent: Startseite
nav_order: 2
---

# Funktionen
{: .no_toc }

## Inhaltsverzeichnis
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Hauptfenster {#main-window}

| Bereich | Funktion |
|:--------|:---------|
| **Kopfzeile** | Status (Aktiv / Umgangen / Geräteprobleme, mit Abtastrate und Latenz), der Schalter **BOOST ON** und das Zahnrad für die Einstellungen |
| **Routing** | Ein- und Ausgabegeräte mit Pegelanzeigen sowie Geräteprofile |
| **Presets** | Ihre Presets und die integrierten |
| **Klang** | Verstärkung, Modus Einfach / Equalizer und das EQ-Diagramm mit Live-Spektrum |
| **Rauschunterdrückung** | Ein/Aus, Methode und Stärke |
| **Noise Gate** | Ein/Aus und der Schwellenwert |

**BOOST ON** schaltet Verstärkung und EQ ein oder aus. Solange es aus ist, wird das Mikrofon mit seinem ursprünglichen Pegel durchgeleitet; Rauschunterdrückung und Noise Gate arbeiten weiter wie eingestellt.

## Signalkette {#signal-chain}

Das Audiosignal durchläuft die Effekte in dieser Reihenfolge:

```
Eingang → Rauschunterdrückung → Equalizer → Verstärkung → Noise Gate → Limiter → Ausgang
```

Die Kette arbeitet intern mit 48 kHz mono – dem Format, das die Rauschunterdrücker erwarten –,
daher wird ein Eingang mit beliebiger Abtastrate oder Kanalzahl vorab umgewandelt, und der
Ausgang wird von Windows für das Zielgerät neu gemischt.

## Verstärkung {#gain}

Stellen Sie die Verstärkung mit dem Schieberegler, den Schaltflächen −1 / +1 oder durch Ziehen der Verstärkungslinie im Diagramm nach oben oder unten ein. Der Bereich reicht von −30 dB bis +50 dB; **Verstärkung über +50 dB zulassen** in den Einstellungen erweitert ihn auf +100 dB.

## Equalizer {#equalizer}

Die Karte **Klang** hat zwei Modi:

- **Einfach** -- es wird nur die Verstärkung angewendet. Ziehen Sie an beliebiger Stelle im Diagramm, um sie einzustellen.
- **Equalizer** -- fügt zusätzlich zur Verstärkung bis zu 16 EQ-Punkte hinzu.

Das Diagramm zeigt die Frequenzgangkurve über einem Live-Spektrum des Ausgangs.

### Bedienung des Diagramms {#graph-operation}

| Aktion | Bedienung |
|:-------|:----------|
| **Punkt hinzufügen** | Doppelklick auf das Diagramm oder Klick auf **Punkt hinzufügen** |
| **Punkt verschieben** | Ziehen – links/rechts ändert die Frequenz, oben/unten die Verstärkung |
| **Breite (Q) ändern** | Mausrad über dem Punkt drehen |
| **Typ ändern / löschen** | Rechtsklick auf den Punkt |
| **Exakte Werte bearbeiten** | Punkt anklicken und die Felder unter dem Diagramm verwenden (Typ, Frequenz, Verstärkung, Q) |
| **Alle Punkte entfernen** | **EQ zurücksetzen** |

Punkttypen: **Glocke**, **Low-Shelf**, **High-Shelf**, **Tiefensperre (Hochpass)** und **Höhensperre (Tiefpass)**.

## Rauschunterdrückung {#noise-suppression}

Entfernt Hintergrundgeräusche und erhält dabei die Stimme. Anders als ein Noise Gate
funktioniert dies **auch während Sie sprechen**. Drei Methoden stehen zur Verfügung:

| Methode | Funktionsweise | Stärken | CPU (ein Kern) | Latenz |
|:--------|:---------------|:--------|:---------------|:-------|
| **Leicht** | Spektrale Rauschminderung (Wiener-Filter, Grundrauschen pro Frequenz gelernt) | Gleichmäßige Geräusche: Lüfter, Rauschen, Klimaanlage | ~0.05 % | +10 ms |
| **Standard** | Neuronales Netz [RNNoise](https://github.com/xiph/rnnoise) | Gleichmäßige Geräusche sowie Tastenanschläge und Mausklicks | ~0.5 % | +10 ms |
| **Hohe Qualität** | Neuronales Netz [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet), Vollband | Die natürlichste Stimme, auch bei lauten oder wechselnden Geräuschen | ~2 % | +40 ms |

| Regler | Beschreibung |
|:-------|:-------------|
| **Schalter** | Schaltet die Rauschunterdrückung ein oder aus |
| **Methode** | Leicht / Standard / Hohe Qualität (für Details mit der Maus darüberfahren) |
| **Stärke** | Mischung zwischen bearbeitetem und ursprünglichem Signal |
| **Stimme** | Anzeige, wie sicher der Unterdrücker ist, dass Sie sprechen |

- Methode und Stärke werden in Presets gespeichert.
- Hohe Qualität lädt sein Modell bei der ersten Auswahl im Hintergrund
  (deutlich unter einer Sekunde); bis dahin wird das Signal unbearbeitet
  durchgeleitet. Im ARM64-Build für Windows ist diese Methode nicht verfügbar.
- Leicht benötigt nach dem Einschalten ein bis zwei Sekunden, um das Rauschen zu lernen.
- Alle Methoden sind in die ausführbare Datei integriert: Standard verwendet
  [nnnoiseless](https://github.com/jneem/nnnoiseless), eine Rust-Portierung von RNNoise
  (BSD-3-Clause); Hohe Qualität verwendet DeepFilterNets `libDF` auf der reinen Rust-Laufzeit
  [tract](https://github.com/sonos/tract) (MIT / Apache-2.0).

{: .note }
Das Noise Gate ist auch zusätzlich zur Rauschunterdrückung nützlich – die
Unterdrücker dämpfen Geräusche, erreichen zwischen Sätzen aber nicht immer völlige Stille.
Das Gate beseitigt das verbleibende Grundrauschen.

## Noise Gate {#noise-gate}

Schaltet das Mikrofon stumm, solange sein Pegel unter einem Schwellenwert bleibt, sodass die Pausen zwischen Sätzen völlig still sind.

- Schalten Sie es mit dem Schalter ein und **ziehen Sie dann die Markierung** auf der Anzeige knapp über Ihr Hintergrundrauschen (oder klicken Sie an die gewünschte Stelle).
- Die Anzeige zeigt den Pegel nach der Verstärkung; das Abzeichen zeigt **Durchlass**, während Sie zu hören sind, und **Stumm**, während das Gate geschlossen ist.

## Limiter {#limiter}

Immer aktiv am Ende der Kette. Er hält die Ausgangsspitzen unter der **Ausgangsobergrenze** (Einstellungen, Standard −1.0 dBFS), damit ein plötzliches lautes Geräusch nie übersteuert. Während er den Pegel absenkt, wird die Ausgangsanzeige rot und darunter erscheint *Limiter: −x dB*.

## Presets {#presets}

Ein Preset speichert die Verstärkung, den Modus und die EQ-Punkte, die Rauschunterdrückung (Ein/Aus, Methode, Stärke) und das Noise Gate.

| Aktion | Vorgehen |
|:-------|:---------|
| **Anwenden** | Ein Preset unter **Meine Presets** oder **Integriert** anklicken |
| **Als neu speichern** | **Als neu speichern**, dann einen Namen eingeben |
| **Änderungen speichern** | **Speichern** (erscheint, wenn das aktuelle Benutzer-Preset geändert wurde) |
| **Änderungen rückgängig machen** | **Zurücksetzen** |
| **Umbenennen / überschreiben / löschen** | Die Schaltfläche **⋯** an einem Preset oder Rechtsklick darauf |
| **Ordner öffnen** | Das Ordnersymbol öffnet den Ordner mit den `.json`-Dateien der Presets |

Wenn Sie von einem geänderten Preset wegwechseln, fragt Easy MIC Booster zuerst, ob es gespeichert werden soll. Integrierte Presets: *Original (0 dB)*, *Boost +12 dB*, *Boost +24 dB*, *Klare Stimme*, *Warme Stimme* und *Streaming*.

## Geräteprofile {#device-profiles}

Ein Geräteprofil merkt sich ein Eingangs-/Ausgangspaar. Verwenden Sie die Liste **Geräteprofil** in der Karte **Routing**:

- **Aktuelle Geräte als Profil speichern** -- speichert den aktuellen Ein- und Ausgang unter einem Namen.
- Wählen Sie ein Profil, um beide Geräte gleichzeitig zu wechseln; löschen Sie es über dieselbe Liste.

Profile lassen sich auch über das Menü im Infobereich wechseln.

## Menü im Infobereich {#tray-menu}

Ein Rechtsklick auf das Symbol im Infobereich bietet:

- **Boost** und **Rauschunterdrückung** ein/aus
- **Verstärkung** -- Schritte von ±3 dB, gängige Werte und *Equalizer verwenden*
- **Presets**, **Geräteprofile**, **Eingang** und **Ausgang**
- **Easy MIC Booster öffnen** und **Beenden**

## Dauerhafte Einstellungen {#persistent-settings}

Alles wird automatisch unter `%APPDATA%\EasyMICBooster` gespeichert (`settings.json` sowie die Ordner `Presets` und `DeviceProfiles`) und beim nächsten Start wiederhergestellt. Einstellungen und Presets aus v1.x werden automatisch übernommen.
