---
title: Accueil
layout: default
nav_order: 16
has_children: true
has_toc: false
---

# Easy MIC Booster

Amplification du gain du microphone et traitement audio de haute qualité pour Windows.
{: .fs-6 .fw-300 }

Easy MIC Booster capte votre microphone, le nettoie et l'amplifie (suppression de bruit, égaliseur, gain, noise gate et limiteur), puis envoie le résultat vers un périphérique audio virtuel comme VB-CABLE : toutes vos applications entendent ainsi la voix traitée.

[Commencer](user-guide){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[Télécharger](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest){: .btn .fs-5 .mb-4 .mb-md-0 .mr-2 }
[GitHub](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster){: .btn .fs-5 .mb-4 .mb-md-0 }

![Fenêtre principale d'Easy MIC Booster](../images/app_top.png)

---

## Fonctionnalités principales {#key-features}

- **Amplificateur logiciel** -- amplifie un microphone trop faible au-delà de la limite de gain de Windows (jusqu'à +50 dB, ou +100 dB une fois la limite levée)
- **Suppression de bruit** -- supprime ventilateurs, frappes au clavier et bruits de la pièce, même pendant que vous parlez ; au choix Léger, Standard (RNNoise) ou Haute qualité (DeepFilterNet3)
- **Égaliseur paramétrique** -- façonnez votre timbre sur un graphe affichant le spectre en direct
- **Noise gate** -- coupe le micro entre les phrases pour des silences parfaitement nets
- **Limiteur** -- maintient les crêtes sous un plafond pour qu'un son fort et soudain ne sature jamais
- **Préréglages et profils de périphériques** -- changez de son et de couple micro/sortie en un clic
- **Contrôle depuis la zone de notification** -- l'application reste active dans la zone de notification ; boost, préréglages et périphériques sont à un clic droit
- **8 langues** -- English, 日本語, 简体中文, 繁體中文, 한국어, Deutsch, Français, Español

## Fonctionnement {#how-it-works}

```
Microphone
    │
    ▼
Capture (WASAPI sous Windows)
    │
    ▼
Suppression de bruit → Égaliseur → Gain → Noise gate → Limiteur
    │
    ▼
Périphérique audio virtuel (ex. CABLE Input)
    │
    ▼
Discord / Zoom / OBS / Teams / jeux …
```

## Démarrage rapide {#quick-start}

1. Installez [VB-CABLE](https://vb-audio.com/Cable/).
2. Téléchargez la dernière version (programme d'installation `...-setup.exe` ou zip portable) et lancez Easy MIC Booster.
3. Dans **Routage**, choisissez votre microphone comme **Entrée** et `CABLE Input (VB-Audio Virtual Cable)` comme **Sortie**.
4. Vérifiez que **BOOST ON** est activé en haut à droite et réglez le **Gain**.
5. Dans Discord, Zoom, OBS, etc., sélectionnez `CABLE Output (VB-Audio Virtual Cable)` comme microphone.

Consultez le [Guide de l'utilisateur](user-guide) pour la procédure complète.
