---
title: Funciones
layout: default
parent: Inicio
nav_order: 2
---

# Funciones
{: .no_toc }

## Índice
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Ventana principal {#main-window}

| Área | Qué hace |
|:-----|:-------------|
| **Encabezado** | Indicador de estado (Activo / Omitido / problemas de dispositivo, con frecuencia de muestreo y latencia), el interruptor **BOOST ON** y el engranaje de Ajustes |
| **Enrutamiento** | Dispositivos de entrada y salida con medidores de nivel, y perfiles de dispositivos |
| **Preajustes** | Sus preajustes y los integrados |
| **Sonido** | Ganancia, modo Simple / Ecualizador y el gráfico de EQ con un espectro en tiempo real |
| **Supresión de ruido** | Activación/desactivación, método e intensidad |
| **Puerta de ruido** | Activación/desactivación y el umbral |

**BOOST ON** activa o desactiva la ganancia y el EQ. Mientras está desactivado, el micrófono pasa con su nivel original; la supresión de ruido y la puerta de ruido siguen funcionando como estén configuradas.

## Cadena de señal {#signal-chain}

El audio pasa por los efectos en este orden:

```
Entrada → Supresión de ruido → Ecualizador → Ganancia → Puerta de ruido → Limitador → Salida
```

La cadena funciona internamente a 48 kHz mono (el formato que esperan los supresores de ruido), por lo que una entrada
con cualquier frecuencia de muestreo o número de canales se convierte de antemano y Windows
vuelve a mezclar la salida para el dispositivo de destino.

## Ganancia {#gain}

Ajuste la ganancia con el control deslizante, los botones −1 / +1 o arrastrando hacia arriba o abajo la línea de ganancia del gráfico. El rango es de −30 dB a +50 dB; **Permitir ganancia superior a +50 dB** en Ajustes lo amplía hasta +100 dB.

## Ecualizador {#equalizer}

La tarjeta **Sonido** tiene dos modos:

- **Simple** -- solo se aplica la ganancia. Arrastre en cualquier punto del gráfico para ajustarla.
- **Ecualizador** -- añade hasta 16 puntos de EQ por encima de la ganancia.

El gráfico muestra la curva de respuesta sobre un espectro en tiempo real de la salida.

### Uso del gráfico {#graph-operation}

| Acción | Operación |
|:-------|:----------|
| **Añadir un punto** | Doble clic en el gráfico, o clic en **Añadir punto** |
| **Mover un punto** | Arrástrelo: izquierda/derecha cambia la frecuencia, arriba/abajo la ganancia |
| **Cambiar el ancho (Q)** | Gire la rueda del ratón sobre el punto |
| **Cambiar el tipo / eliminar** | Clic derecho sobre el punto |
| **Editar valores exactos** | Haga clic en un punto y use los campos bajo el gráfico (tipo, frecuencia, ganancia, Q) |
| **Borrar todos los puntos** | **Restablecer EQ** |

Tipos de punto: **Pico**, **Estante de graves**, **Estante de agudos**, **Corte de graves (paso alto)** y **Corte de agudos (paso bajo)**.

## Supresión de ruido {#noise-suppression}

Elimina el ruido de fondo conservando la voz. A diferencia de una puerta de ruido,
funciona **incluso mientras habla**. Hay tres métodos disponibles:

| Método | Cómo funciona | Bueno para | CPU (un núcleo) | Latencia |
|:-------|:-------------|:--------|:---------------|:--------|
| **Ligero** | Reducción espectral de ruido (filtro de Wiener, nivel de ruido aprendido por frecuencia) | Ruido constante: ventiladores, siseo, aire acondicionado | ~0.05 % | +10 ms |
| **Estándar** | Red neuronal [RNNoise](https://github.com/xiph/rnnoise) | Ruido constante más tecleo y clics del ratón | ~0.5 % | +10 ms |
| **Alta calidad** | Red neuronal [DeepFilterNet3](https://github.com/Rikorose/DeepFilterNet), banda completa | La voz más natural, incluso con ruido fuerte o cambiante | ~2 % | +40 ms |

| Control | Descripción |
|:--------|:------------|
| **Interruptor** | Activa o desactiva la supresión de ruido |
| **Método** | Ligero / Estándar / Alta calidad (pase el ratón por encima para ver detalles) |
| **Intensidad** | Mezcla entre la señal procesada y la original |
| **Voz** | Medidor de la confianza del supresor en que usted está hablando |

- El método y la intensidad se guardan en los preajustes.
- Alta calidad carga su modelo en segundo plano la primera vez que se
  selecciona (bastante menos de un segundo); hasta entonces la señal pasa
  sin procesar. No está disponible en la compilación de Windows para ARM64.
- Ligero necesita uno o dos segundos para aprender el ruido después de activarse.
- Todos los métodos están integrados en el ejecutable: Estándar usa
  [nnnoiseless](https://github.com/jneem/nnnoiseless), un port de RNNoise a Rust
  (BSD-3-Clause); Alta calidad usa `libDF` de DeepFilterNet sobre el runtime
  [tract](https://github.com/sonos/tract), escrito íntegramente en Rust (MIT / Apache-2.0).

{: .note }
La puerta de ruido sigue siendo útil además de la supresión de ruido: los
supresores atenúan el ruido, pero no siempre alcanzan el silencio absoluto entre frases.
La puerta elimina el ruido residual.

## Puerta de ruido {#noise-gate}

Silencia el micrófono mientras su nivel se mantiene por debajo de un umbral, de modo que las pausas entre frases quedan totalmente en silencio.

- Actívela con el interruptor y luego **arrastre el marcador** del medidor justo por encima de su ruido de fondo (o haga clic donde deba ir).
- El medidor muestra el nivel después de la ganancia; la etiqueta indica **Pasando** mientras se le oye y **Silenciado** mientras la puerta está cerrada.

## Limitador {#limiter}

Siempre activo al final de la cadena. Mantiene los picos de salida por debajo del **Techo de salida** (Ajustes, predeterminado −1.0 dBFS), para que un sonido fuerte repentino nunca sature. Mientras reduce el nivel, el medidor de salida se pone rojo y aparece *Limitador: −x dB* debajo.

## Preajustes {#presets}

Un preajuste guarda la ganancia, el modo y los puntos de EQ, la supresión de ruido (activación, método, intensidad) y la puerta de ruido.

| Acción | Cómo |
|:-------|:----|
| **Aplicar** | Haga clic en un preajuste en **Mis preajustes** o **Integrados** |
| **Guardar como nuevo** | **Guardar como nuevo** y luego escriba un nombre |
| **Guardar cambios** | **Guardar** (aparece cuando se ha modificado el preajuste de usuario actual) |
| **Deshacer cambios** | **Revertir** |
| **Cambiar nombre / sobrescribir / eliminar** | El botón **⋯** de un preajuste, o clic derecho sobre él |
| **Abrir carpeta** | El icono de carpeta abre la carpeta con los archivos `.json` de los preajustes |

Cuando cambia desde un preajuste modificado, Easy MIC Booster pregunta si desea guardarlo primero. Preajustes integrados: *Original (0 dB)*, *Boost +12 dB*, *Boost +24 dB*, *Voz clara*, *Voz cálida* y *Streaming*.

## Perfiles de dispositivos {#device-profiles}

Un perfil de dispositivos recuerda una pareja de entrada/salida. Use la lista **Perfil de dispositivos** de la tarjeta **Enrutamiento**:

- **Guardar los dispositivos actuales como perfil** -- guarda la entrada y la salida actuales con un nombre.
- Elija un perfil para cambiar ambos dispositivos a la vez; elimínelo desde la misma lista.

Los perfiles también se pueden cambiar desde el menú de la bandeja.

## Menú de la bandeja {#tray-menu}

Haga clic derecho en el icono de la bandeja para acceder a:

- Activar/desactivar **Boost** y **Supresión de ruido**
- **Ganancia** -- pasos de ±3 dB, valores habituales y *Usar ecualizador*
- **Preajustes**, **Perfiles de dispositivos**, **Entrada** y **Salida**
- **Abrir Easy MIC Booster** y **Salir**

## Ajustes persistentes {#persistent-settings}

Todo se guarda automáticamente en `%APPDATA%\EasyMICBooster` (`settings.json`, más las carpetas `Presets` y `DeviceProfiles`) y se restaura en el siguiente inicio. Los ajustes y preajustes de la v1.x se importan automáticamente.
