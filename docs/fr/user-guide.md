---
title: Guide de l'utilisateur
layout: default
parent: Accueil
nav_order: 1
---

# Guide de l'utilisateur
{: .no_toc }

## Table des matières
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Configuration requise {#system-requirements}

- **Système d'exploitation** : Windows 10 / 11 (x64, x86 ou ARM64)
- **Pilote audio virtuel** : [VB-CABLE](https://vb-audio.com/Cable/) ou un câble virtuel équivalent

Des versions Linux et macOS sont également publiées, mais elles sont expérimentales ; consultez le [Guide du développeur](developer-guide#supported-platforms) pour savoir comment y router l'audio.

## Installation {#installation}

### 1. Installer VB-CABLE {#1-install-vb-cable}

1. Téléchargez-le depuis [vb-audio.com/Cable](https://vb-audio.com/Cable/).
2. Décompressez l'archive et exécutez `VBCABLE_Setup_x64.exe` **en tant qu'administrateur**.
3. Redémarrez votre PC.

### 2. Installer Easy MIC Booster {#2-install-easy-mic-booster}

Téléchargez la dernière version depuis la page [Releases](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest), au choix :

- **Programme d'installation** (`EasyMICBooster-x64-vX.Y.Z-setup.exe`) : installe l'application pour l'utilisateur actuel sans droits d'administrateur et ajoute une entrée au menu Démarrer. Désinstallez-la depuis Paramètres Windows > Applications ; vos réglages sont conservés.
- **Version portable** (`EasyMICBooster-x64-vX.Y.Z-portable.zip`) : décompressez l'archive dans le dossier de votre choix et lancez `EasyMICBooster.exe`.

Utilisez les fichiers `arm64` sous Windows on ARM et les fichiers `x86` uniquement sous Windows 32 bits. La version ARM64 n'inclut pas la méthode de suppression de bruit *Haute qualité*.

## Première configuration {#first-time-setup}

### Choisir les périphériques {#choose-the-devices}

Dans la carte **Routage**, à gauche :

| Champ | Choix |
|:------|:------|
| **Entrée (micro)** | Votre microphone physique, ou *Périphérique par défaut* pour suivre Windows |
| **Sortie (câble virtuel)** | `CABLE Input (VB-Audio Virtual Cable)` |

Le vumètre sous chaque liste affiche le niveau avant et après traitement. La ligne qui les sépare résume ce qui est actif (par exemple `+12.0 dB · NS · Gate`).

{: .warning }
Si la sortie est réglée sur vos haut-parleurs ou votre casque (le périphérique de lecture par défaut), vous vous entendrez. Choisissez toujours le câble virtuel comme sortie.

### Régler le gain {#set-the-gain}

1. Vérifiez que **BOOST ON** est activé en haut à droite.
2. Parlez normalement et augmentez le **Gain** dans la carte **Son** jusqu'à ce que les crêtes du vumètre de sortie atteignent la zone verte à jaune. Les boutons −1 / +1 ajustent par pas de 1 dB.
3. Si le vumètre passe au rouge et que *Limiteur* apparaît, le limiteur est en train de contenir les crêtes ; baissez légèrement le gain.

Lorsque l'audio circule, l'indicateur d'état à côté du titre affiche **Actif** avec la fréquence d'échantillonnage et la latence.

### Configurer l'application de destination {#configure-the-receiving-app}

Dans Discord, Zoom, OBS, Teams, etc., ouvrez les paramètres audio et sélectionnez `CABLE Output (VB-Audio Virtual Cable)` comme microphone (périphérique d'entrée).

{: .warning }
Ne définissez **pas** `CABLE Output` comme **périphérique de lecture par défaut** dans les paramètres son de Windows. Cela crée une boucle de rétroaction (larsen).

## Utilisation au quotidien {#daily-use}

- **Contournement** -- désactivez **BOOST ON** pour laisser passer le microphone sans gain ni EQ (la suppression de bruit et le gate restent tels quels).
- **Zone de notification** -- fermer la fenêtre laisse l'application tourner dans la zone de notification. Faites un clic droit sur l'icône pour activer ou désactiver le boost ou la suppression de bruit, modifier le gain, changer de préréglage, de profil de périphériques ou de périphérique, ou quitter. Cliquez dessus pour ouvrir la fenêtre.
- **Préréglages** -- enregistrez votre son avec **Enregistrer comme nouveau** et passez d'un préréglage à l'autre dans la carte **Préréglages** ou depuis la zone de notification. Voir [Fonctionnalités](features#presets).
- **Profils de périphériques** -- enregistrez un couple micro/sortie et passez de l'un à l'autre (par exemple un casque au bureau et un micro USB pour le streaming). Voir [Fonctionnalités](features#device-profiles).
- **Périphériques débranchés** -- si le microphone ou la sortie disparaît, l'état passe au rouge ; le routage reprend automatiquement dès que le périphérique est reconnecté.

## Paramètres {#settings}

Ouvrez les **Paramètres** avec l'icône d'engrenage en haut à droite.

| Paramètre | Description |
|:----------|:------------|
| **Langue** | Langue de l'interface (suit Windows au premier lancement) |
| **Lancer à l'ouverture de session** | Démarre automatiquement à l'ouverture de votre session Windows (dans la zone de notification) |
| **Rester dans la zone de notification à la fermeture** | Désactivé : fermer la fenêtre quitte l'application |
| **Autoriser un gain supérieur à +50 dB** | Relève la limite de gain à +100 dB |
| **Plafond de sortie** | Niveau de crête sous lequel le limiteur maintient la sortie (par défaut −1.0 dBFS) |
| **Rechercher des mises à jour** | Recherche une version plus récente sur GitHub au démarrage |

Les boutons situés en dessous ouvrent le dossier des paramètres, la page GitHub et la page Sponsor, ou quittent l'application.

Les paramètres, préréglages et profils de périphériques sont enregistrés automatiquement dans `%APPDATA%\EasyMICBooster` et restaurés au lancement suivant. Les paramètres de la v1.x sont importés au premier démarrage.

Consultez [Fonctionnalités](features) pour le détail de chaque réglage audio.
