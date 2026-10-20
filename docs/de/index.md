---
title: Startseite
layout: default
nav_order: 15
has_children: true
has_toc: false
---

# Easy MIC Booster

Mikrofonverstärkung & hochwertige Audioverarbeitung für Windows.
{: .fs-6 .fw-300 }

Nimmt Ihr Mikrofon auf, bereinigt und verstärkt es – Rauschunterdrückung, Equalizer, Verstärkung, Noise Gate und Limiter – und leitet das Ergebnis an ein virtuelles Audiogerät wie VB-CABLE weiter, sodass jede App die bearbeitete Stimme hört.

[Loslegen](user-guide){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[Download](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest){: .btn .fs-5 .mb-4 .mb-md-0 .mr-2 }
[GitHub](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster){: .btn .fs-5 .mb-4 .mb-md-0 }

![Hauptfenster von Easy MIC Booster](../images/app_top.png)

---

## Hauptfunktionen {#key-features}

- **Software-Verstärker** -- verstärkt ein leises Mikrofon über die Windows-Grenze hinaus (bis +50 dB, freigeschaltet bis +100 dB)
- **Rauschunterdrückung** -- entfernt Lüfter, Tastenanschläge und Raumgeräusche auch während Sie sprechen; wählen Sie Leicht, Standard (RNNoise) oder Hohe Qualität (DeepFilterNet3)
- **Parametrischer Equalizer** -- formen Sie Ihren Klang auf einem Diagramm mit Live-Spektrum
- **Noise Gate** -- schaltet das Mikrofon zwischen Sätzen stumm, für völlig stille Pausen
- **Limiter** -- hält Spitzen unter einer Obergrenze, damit plötzliche laute Geräusche nie übersteuern
- **Presets & Geräteprofile** -- wechseln Sie Klänge und Mikrofon-/Ausgangspaare mit einem Klick
- **Steuerung über den Infobereich** -- läuft im Infobereich weiter; Boost, Presets und Geräte sind nur einen Rechtsklick entfernt
- **8 Sprachen** -- English, 日本語, 简体中文, 繁體中文, 한국어, Deutsch, Français, Español

## Funktionsweise {#how-it-works}

```
Mikrofon
    │
    ▼
Aufnahme (WASAPI unter Windows)
    │
    ▼
Rauschunterdrückung → Equalizer → Verstärkung → Noise Gate → Limiter
    │
    ▼
Virtuelles Audiogerät (z. B. CABLE Input)
    │
    ▼
Discord / Zoom / OBS / Teams / Spiele …
```

## Schnellstart {#quick-start}

1. Installieren Sie [VB-CABLE](https://vb-audio.com/Cable/).
2. Laden Sie die neueste Version herunter (Installationsprogramm `...-setup.exe` oder das portable Zip) und starten Sie Easy MIC Booster.
3. Wählen Sie unter **Routing** Ihr Mikrofon als **Eingang** und `CABLE Input (VB-Audio Virtual Cable)` als **Ausgang**.
4. Stellen Sie sicher, dass oben rechts **BOOST ON** aktiviert ist, und stellen Sie die **Verstärkung** ein.
5. Wählen Sie in Discord, Zoom, OBS usw. `CABLE Output (VB-Audio Virtual Cable)` als Mikrofon.

Die vollständige Anleitung finden Sie im [Benutzerhandbuch](user-guide).
