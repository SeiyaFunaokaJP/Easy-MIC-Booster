---
title: Fonctionnalités
layout: default
parent: Accueil
nav_order: 2
---

# Fonctionnalités
{: .no_toc }

## Table des matières
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Fenêtre principale {#main-window}

| Zone | Rôle |
|:-----|:-----|
| **En-tête** | Indicateur d'état (Actif / Contourné / problèmes de périphérique, avec fréquence d'échantillonnage et latence), l'interrupteur **BOOST ON** et l'engrenage des Paramètres |
| **Routage** | Périphériques d'entrée et de sortie avec vumètres, et profils de périphériques |
| **Préréglages** | Vos préréglages et les préréglages intégrés |
| **Son** | Gain, mode Simple / Égaliseur et graphe d'EQ avec spectre en direct |
| **Suppression de bruit** | Activation, méthode et intensité |
| **Noise gate** | Activation et seuil |

**BOOST ON** active ou désactive le gain et l'EQ. Lorsqu'il est désactivé, le microphone passe à son niveau d'origine ; la suppression de bruit et le noise gate continuent de fonctionner selon leurs réglages.

## Chaîne de traitement {#signal-chain}

L'audio traverse les effets dans cet ordre :

```
Entrée → Suppression de bruit → Égaliseur → Gain → Noise gate → Limiteur → Sortie
```

La chaîne fonctionne en interne à 48 kHz mono (le format attendu par les suppresseurs de bruit) :
une entrée de n'importe quelle fréquence d'échantillonnage ou nombre de canaux est donc convertie en amont,
et la sortie est remixée par Windows pour le périphérique de destination.

## Gain {#gain}

Réglez le gain avec le curseur, les boutons −1 / +1, ou en faisant glisser la ligne de gain du graphe vers le haut ou le bas. La plage va de −30 dB à +50 dB ; l'option **Autoriser un gain supérieur à +50 dB** des Paramètres l'étend jusqu'à +100 dB.

## Égaliseur {#equalizer}

La carte **Son** propose deux modes :

- **Simple** -- seul le gain est appliqué. Faites glisser n'importe où sur le graphe pour le régler.
- **Égaliseur** -- ajoute jusqu'à 16 points d'EQ en plus du gain.

Le graphe affiche la courbe de réponse par-dessus le spectre de la sortie en direct.

### Utilisation du graphe {#graph-operation}

| Action | Manipulation |
|:-------|:-------------|
| **Ajouter un point** | Double-cliquez sur le graphe, ou cliquez sur **Ajouter un point** |
| **Déplacer un point** | Faites-le glisser : gauche/droite modifie la fréquence, haut/bas le gain |
| **Modifier la largeur (Q)** | Tournez la molette de la souris au-dessus du point |
| **Changer le type / supprimer** | Clic droit sur le point |
| **Saisir des valeurs exactes** | Cliquez sur un point et utilisez les champs sous le graphe (type, fréquence, gain, Q) |
| **Effacer tous les points** | **Réinitialiser l'EQ** |

Types de points : **Cloche**, **Plateau grave**, **Plateau aigu**, **Coupe-bas (passe-haut)** et **Coupe-haut (passe-bas)**.

## Suppression de bruit {#noise-suppression}

Supprime le bruit de fond tout en préservant la voix. Contrairement à un noise gate, elle
agit **même pendant que vous parlez**. Trois méthodes sont disponibles :

| Méthode | Principe | Points forts | CPU (un cœur) | Latence |
|:--------|:---------|:-------------|:--------------|:--------|
| **Léger** | Réduction spectrale du bruit (filtre de Wiener, bruit de fond appris par fréquence) | Bruits constants : ventilateurs, souffle, climatisation | ~0.05 % | +10 ms |
| **Standard** | Réseau de neurones [RNNoise](https://github.com/xiph/rnnoise) | Bruits constants, plus frappes au clavier et clics de souris | ~0.5 % | +10 ms |
| **Haute qualité** | Réseau de neurones [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet), pleine bande | La voix la plus naturelle, même avec un bruit fort ou changeant | ~2 % | +40 ms |

| Réglage | Description |
|:--------|:------------|
| **Interrupteur** | Active ou désactive la suppression de bruit |
| **Méthode** | Léger / Standard / Haute qualité (survolez pour plus de détails) |
| **Intensité** | Dosage entre le signal traité et le signal d'origine |
| **Voix** | Vumètre indiquant à quel point le suppresseur est sûr que vous parlez |

- La méthode et l'intensité sont enregistrées dans les préréglages.
- Haute qualité charge son modèle en arrière-plan la première fois qu'elle est
  sélectionnée (bien moins d'une seconde) ; en attendant, le signal passe
  sans traitement. Elle n'est pas disponible dans la version Windows ARM64.
- Léger a besoin d'une seconde ou deux pour apprendre le bruit après son activation.
- Toutes les méthodes sont intégrées à l'exécutable : Standard utilise
  [nnnoiseless](https://github.com/jneem/nnnoiseless), un portage en Rust de RNNoise
  (BSD-3-Clause) ; Haute qualité utilise `libDF` de DeepFilterNet sur le moteur d'exécution
  100 % Rust [tract](https://github.com/sonos/tract) (MIT / Apache-2.0).

{: .note }
Le noise gate reste utile en complément de la suppression de bruit : les
suppresseurs atténuent le bruit, mais n'atteignent pas toujours le silence absolu entre les phrases.
Le gate élimine le bruit résiduel.

## Noise gate {#noise-gate}

Coupe le microphone tant que son niveau reste sous un seuil, afin que les pauses entre les phrases soient parfaitement silencieuses.

- Activez-le avec l'interrupteur, puis **faites glisser le repère** sur le vumètre juste au-dessus de votre bruit de fond (ou cliquez à l'endroit voulu).
- Le vumètre affiche le niveau après gain ; le badge indique **Passe** lorsque l'on vous entend et **Coupé** lorsque le gate est fermé.

## Limiteur {#limiter}

Toujours actif en fin de chaîne. Il maintient les crêtes de sortie sous le **Plafond de sortie** (Paramètres, par défaut −1.0 dBFS), afin qu'un son fort et soudain ne sature jamais. Pendant qu'il réduit le niveau, le vumètre de sortie passe au rouge et *Limiteur : −x dB* s'affiche en dessous.

## Préréglages {#presets}

Un préréglage enregistre le gain, le mode et les points d'EQ, la suppression de bruit (activation, méthode, intensité) et le noise gate.

| Action | Comment |
|:-------|:--------|
| **Appliquer** | Cliquez sur un préréglage dans **Mes préréglages** ou **Intégrés** |
| **Enregistrer comme nouveau** | **Enregistrer comme nouveau**, puis saisissez un nom |
| **Enregistrer les modifications** | **Enregistrer** (apparaît lorsque le préréglage utilisateur actuel est modifié) |
| **Annuler les modifications** | **Rétablir** |
| **Renommer / remplacer / supprimer** | Le bouton **⋯** d'un préréglage, ou un clic droit dessus |
| **Ouvrir le dossier** | L'icône de dossier ouvre le dossier contenant les fichiers `.json` des préréglages |

Lorsque vous quittez un préréglage modifié, Easy MIC Booster vous demande d'abord s'il faut l'enregistrer. Préréglages intégrés : *Original (0 dB)*, *Boost +12 dB*, *Boost +24 dB*, *Voix claire*, *Voix chaude* et *Streaming*.

## Profils de périphériques {#device-profiles}

Un profil de périphériques mémorise un couple entrée/sortie. Utilisez la liste **Profil de périphériques** dans la carte **Routage** :

- **Enregistrer les périphériques actuels comme profil** -- enregistre l'entrée et la sortie actuelles sous un nom.
- Choisissez un profil pour changer les deux périphériques d'un coup ; supprimez-le depuis la même liste.

Les profils peuvent aussi être changés depuis le menu de la zone de notification.

## Menu de la zone de notification {#tray-menu}

Un clic droit sur l'icône de la zone de notification donne accès à :

- l'activation de **Boost** et de **Suppression de bruit**
- **Gain** -- pas de ±3 dB, valeurs courantes et *Utiliser l'égaliseur*
- **Préréglages**, **Profils de périphériques**, **Entrée** et **Sortie**
- **Ouvrir Easy MIC Booster** et **Quitter**

## Paramètres persistants {#persistent-settings}

Tout est enregistré automatiquement dans `%APPDATA%\EasyMICBooster` (`settings.json`, ainsi que les dossiers `Presets` et `DeviceProfiles`) et restauré au lancement suivant. Les paramètres et préréglages de la v1.x sont importés automatiquement.
