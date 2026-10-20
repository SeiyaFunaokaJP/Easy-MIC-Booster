---
title: Guía del usuario
layout: default
parent: Inicio
nav_order: 1
---

# Guía del usuario
{: .no_toc }

## Índice
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Requisitos del sistema {#system-requirements}

- **SO**: Windows 10 / 11 (x64, x86 o ARM64)
- **Controlador de audio virtual**: [VB-CABLE](https://vb-audio.com/Cable/) o un cable virtual equivalente

También se publican compilaciones para Linux y macOS, pero son experimentales; consulte la [Guía para desarrolladores](developer-guide#supported-platforms) para saber cómo enrutar el audio en esos sistemas.

## Instalación {#installation}

### 1. Instalar VB-CABLE {#1-install-vb-cable}

1. Descárguelo de [vb-audio.com/Cable](https://vb-audio.com/Cable/).
2. Descomprímalo y ejecute `VBCABLE_Setup_x64.exe` **como administrador**.
3. Reinicie el PC.

### 2. Instalar Easy MIC Booster {#2-install-easy-mic-booster}

Descargue la última versión desde la página de [Releases](https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest), en una de estas formas:

- **Instalador** (`EasyMICBooster-x64-vX.Y.Z-setup.exe`): se instala para el usuario actual sin derechos de administrador y añade una entrada al menú Inicio. Desinstálelo desde Configuración de Windows > Aplicaciones; sus ajustes se conservan.
- **Portable** (`EasyMICBooster-x64-vX.Y.Z-portable.zip`): descomprímalo en cualquier carpeta y ejecute `EasyMICBooster.exe`.

Use los archivos `arm64` en Windows on ARM y los archivos `x86` solo en Windows de 32 bits. La compilación ARM64 no incluye el método de supresión de ruido *Alta calidad*.

## Configuración inicial {#first-time-setup}

### Elegir los dispositivos {#choose-the-devices}

En la tarjeta **Enrutamiento**, a la izquierda:

| Campo | Elija |
|:------|:-------|
| **Entrada (micrófono)** | Su micrófono físico, o *Dispositivo predeterminado* para seguir a Windows |
| **Salida (cable virtual)** | `CABLE Input (VB-Audio Virtual Cable)` |

El medidor bajo cada lista muestra el nivel antes y después del procesamiento. La línea entre ambos resume lo que está activo (por ejemplo, `+12.0 dB · NS · Gate`).

{: .warning }
Si la salida son sus altavoces o auriculares (el dispositivo de reproducción predeterminado), se escuchará a sí mismo. Elija siempre el cable virtual como salida.

### Ajustar la ganancia {#set-the-gain}

1. Asegúrese de que **BOOST ON** esté activado en la esquina superior derecha.
2. Hable con normalidad y suba la **Ganancia** en la tarjeta **Sonido** hasta que los picos del medidor de salida queden en la zona entre verde y amarillo. Los botones −1 / +1 cambian el valor en pasos de 1 dB.
3. Si el medidor se pone rojo y aparece *Limitador*, el limitador está conteniendo los picos; baje un poco la ganancia.

El indicador de estado junto al título muestra **Activo** con la frecuencia de muestreo y la latencia cuando el audio está fluyendo.

### Configurar la aplicación receptora {#configure-the-receiving-app}

En Discord, Zoom, OBS, Teams, etc., abra los ajustes de audio y seleccione `CABLE Output (VB-Audio Virtual Cable)` como micrófono (dispositivo de entrada).

{: .warning }
**No** configure `CABLE Output` como **dispositivo de reproducción predeterminado** en la configuración de sonido de Windows. Eso crea un bucle de realimentación (acoplamiento).

## Uso diario {#daily-use}

- **Omitir** -- desactive **BOOST ON** para dejar pasar el micrófono sin ganancia ni EQ (la supresión de ruido y la puerta se mantienen como estén).
- **Bandeja** -- al cerrar la ventana, la aplicación sigue funcionando en el área de notificación. Haga clic derecho en el icono de la bandeja para activar o desactivar el boost o la supresión de ruido, cambiar la ganancia, cambiar de preajuste, de perfil de dispositivos y de dispositivos, o salir. Haga clic en él para abrir la ventana.
- **Preajustes** -- guarde su sonido con **Guardar como nuevo** y cambie entre preajustes en la tarjeta **Preajustes** o desde la bandeja. Consulte [Funciones](features#presets).
- **Perfiles de dispositivos** -- guarde una pareja micrófono/salida y cambie entre ellas (por ejemplo, unos auriculares con micrófono en el escritorio y un micrófono USB para hacer streaming). Consulte [Funciones](features#device-profiles).
- **Dispositivos desconectados** -- si el micrófono o la salida desaparecen, el estado se vuelve rojo; el enrutamiento se reanuda automáticamente cuando se vuelven a conectar.

## Ajustes {#settings}

Abra **Ajustes** con el icono del engranaje en la esquina superior derecha.

| Ajuste | Descripción |
|:--------|:------------|
| **Idioma** | Idioma de la interfaz (sigue a Windows en el primer inicio) |
| **Iniciar al iniciar sesión** | Se inicia automáticamente al iniciar sesión en Windows (arranca en la bandeja) |
| **Seguir en la bandeja del sistema al cerrar** | Desactivado: al cerrar la ventana se sale de la aplicación |
| **Permitir ganancia superior a +50 dB** | Eleva el límite de ganancia a +100 dB |
| **Techo de salida** | Nivel de pico por debajo del cual el limitador mantiene la salida (predeterminado −1.0 dBFS) |
| **Buscar actualizaciones** | Comprueba en GitHub si hay una versión más reciente al iniciar |

Los botones de abajo abren la carpeta de ajustes, la página de GitHub y la página de patrocinio, o cierran la aplicación.

Los ajustes, los preajustes y los perfiles de dispositivos se guardan automáticamente en `%APPDATA%\EasyMICBooster` y se restauran en el siguiente inicio. Los ajustes de la v1.x se importan en el primer inicio.

Consulte [Funciones](features) para ver cada control de audio en detalle.
