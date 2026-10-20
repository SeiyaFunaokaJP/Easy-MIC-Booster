---
title: Benutzerhandbuch
layout: default
parent: Startseite
nav_order: 1
---

# Benutzerhandbuch
{: .no_toc }

## Inhaltsverzeichnis
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Systemanforderungen {#system-requirements}

- **Betriebssystem**: Windows 10 / 11 (x64, x86 oder ARM64)
- **Virtueller Audiotreiber**: [VB-CABLE](https://vb-audio.com/Cable/) oder ein gleichwertiges virtuelles Kabel

Builds für Linux und macOS werden ebenfalls veröffentlicht, sind aber experimentell; wie Sie dort Audio weiterleiten, steht im [Entwicklerhandbuch](developer-guide#supported-platforms).

## Installation {#installation}

### 1. VB-CABLE installieren {#1-install-vb-cable}

1. Laden Sie es von [vb-audio.com/Cable](https://vb-audio.com/Cable/) herunter.
2. Entpacken Sie es und führen Sie `VBCABLE_Setup_x64.exe` **als Administrator** aus.
3. Starten Sie Ihren PC neu.

### 2. Easy MIC Booster installieren {#2-install-easy-mic-booster}

Laden Sie die neueste Version von der Seite [Releases](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest) herunter, entweder:

- **Installationsprogramm** (`EasyMICBooster-x64-vX.Y.Z-setup.exe`): installiert für den aktuellen Benutzer ohne Administratorrechte und legt einen Eintrag im Startmenü an. Deinstallieren Sie es über Windows-Einstellungen > Apps; Ihre Einstellungen bleiben erhalten.
- **Portabel** (`EasyMICBooster-x64-vX.Y.Z-portable.zip`): in einen beliebigen Ordner entpacken und `EasyMICBooster.exe` ausführen.

Verwenden Sie die `arm64`-Dateien unter Windows on ARM und die `x86`-Dateien nur unter 32-Bit-Windows. Der ARM64-Build enthält die Rauschunterdrückungsmethode *Hohe Qualität* nicht.

## Ersteinrichtung {#first-time-setup}

### Geräte auswählen {#choose-the-devices}

In der Karte **Routing** auf der linken Seite:

| Feld | Auswahl |
|:-----|:--------|
| **Eingang (Mikrofon)** | Ihr physisches Mikrofon oder *Standardgerät*, um Windows zu folgen |
| **Ausgang (virtuelles Kabel)** | `CABLE Input (VB-Audio Virtual Cable)` |

Die Pegelanzeige unter jeder Liste zeigt den Pegel vor und nach der Verarbeitung. Die Zeile dazwischen fasst zusammen, was aktiv ist (zum Beispiel `+12.0 dB · NS · Gate`).

{: .warning }
Wenn der Ausgang Ihre Lautsprecher oder Kopfhörer sind (das Standard-Wiedergabegerät), hören Sie sich selbst. Wählen Sie als Ausgang immer das virtuelle Kabel.

### Verstärkung einstellen {#set-the-gain}

1. Stellen Sie sicher, dass oben rechts **BOOST ON** aktiviert ist.
2. Sprechen Sie normal und erhöhen Sie die **Verstärkung** in der Karte **Klang**, bis die Ausgangsanzeige im grünen bis gelben Bereich ausschlägt. Die Schaltflächen −1 / +1 ändern den Wert in 1-dB-Schritten.
3. Wenn die Anzeige rot wird und *Limiter* erscheint, hält der Limiter die Spitzen zurück; verringern Sie die Verstärkung etwas.

Der Status neben dem Titel zeigt **Aktiv** mit Abtastrate und Latenz an, wenn Audio fließt.

### Empfangende App konfigurieren {#configure-the-receiving-app}

Öffnen Sie in Discord, Zoom, OBS, Teams usw. die Audioeinstellungen und wählen Sie `CABLE Output (VB-Audio Virtual Cable)` als Mikrofon (Eingabegerät).

{: .warning }
Legen Sie `CABLE Output` in den Windows-Soundeinstellungen **nicht** als **Standard-Wiedergabegerät** fest. Das erzeugt eine Rückkopplungsschleife (Pfeifen).

## Täglicher Gebrauch {#daily-use}

- **Umgehen** -- schalten Sie **BOOST ON** aus, um das Mikrofon ohne Verstärkung oder EQ durchzuleiten (Rauschunterdrückung und Gate bleiben wie eingestellt).
- **Infobereich** -- beim Schließen des Fensters läuft die App im Infobereich weiter. Klicken Sie mit der rechten Maustaste auf das Symbol, um Boost oder Rauschunterdrückung umzuschalten, die Verstärkung zu ändern, Presets, Geräteprofile und Geräte zu wechseln oder die App zu beenden. Ein Klick darauf öffnet das Fenster.
- **Presets** -- speichern Sie Ihren Klang mit **Als neu speichern** und wechseln Sie zwischen Presets in der Karte **Presets** oder über den Infobereich. Siehe [Funktionen](features#presets).
- **Geräteprofile** -- speichern Sie ein Mikrofon-/Ausgangspaar und wechseln Sie zwischen solchen Paaren (zum Beispiel ein Headset am Schreibtisch und ein USB-Mikrofon zum Streamen). Siehe [Funktionen](features#device-profiles).
- **Getrennte Geräte** -- verschwindet das Mikrofon oder der Ausgang, wird der Status rot; das Routing wird automatisch fortgesetzt, sobald das Gerät wieder verbunden ist.

## Einstellungen {#settings}

Öffnen Sie die **Einstellungen** über das Zahnradsymbol oben rechts.

| Einstellung | Beschreibung |
|:------------|:-------------|
| **Sprache** | Sprache der Oberfläche (folgt beim ersten Start Windows) |
| **Bei der Anmeldung starten** | Startet automatisch bei der Anmeldung an Windows (im Infobereich) |
| **Beim Schließen im Infobereich weiterlaufen** | Aus: Schließen des Fensters beendet die App |
| **Verstärkung über +50 dB zulassen** | Hebt die Verstärkungsgrenze auf +100 dB an |
| **Ausgangsobergrenze** | Spitzenpegel, unter dem der Limiter den Ausgang hält (Standard −1.0 dBFS) |
| **Nach Updates suchen** | Prüft beim Start auf GitHub, ob eine neuere Version vorliegt |

Die Schaltflächen darunter öffnen den Einstellungsordner, die GitHub-Seite und die Sponsor-Seite oder beenden die App.

Einstellungen, Presets und Geräteprofile werden automatisch unter `%APPDATA%\EasyMICBooster` gespeichert und beim nächsten Start wiederhergestellt. Einstellungen aus v1.x werden beim ersten Start übernommen.

Alle Audio-Regler im Detail finden Sie unter [Funktionen](features).
