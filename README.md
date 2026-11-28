<p align="center"><img src="assets/icon.svg" width="96" height="96" alt="Icono de PNGTuber Lite"></p>

# PNGTuber Lite

[![Build](https://github.com/roldyoran/pngtuber-lite/actions/workflows/build.yml/badge.svg)](https://github.com/roldyoran/pngtuber-lite/actions/workflows/build.yml)

Overlay de escritorio ultraligero para un avatar PNGTuber: una ventana
transparente, sin bordes y siempre visible encima del escritorio, que abre y
cierra la boca con tu voz. Sin IA, sin reconocimiento de voz y sin modelos: solo
RMS, histéresis y dos imágenes.

- Rust + GPUI (sin Electron, sin webview).
- Dos PNG (boca cerrada / boca abierta) con transparencia real por píxel.
- VAD por umbral en dBFS con tiempos de ataque/cierre para no parpadear.
- Selección de micrófono por identificador estable.
- Ajustes con guardado automático y **previsualización lateral**.
- El overlay no se crea hasta pulsar **Mostrar**: el primer arranque no tapa
  nada.
- Arrastrable con el botón izquierdo y redimensionable por los bordes; clic
  derecho reabre los ajustes.

## Requisitos

- Windows 10 1903 o superior (x86_64).
- Un micrófono que exponga entrada por WASAPI.
- Dos PNG con transparencia. No se incluyen avatares de ejemplo.

## Instalación

Este repositorio **no se compila en local**: el único verificador es GitHub
Actions.

1. Haz push a `main` (o lanza el workflow `Build` a mano).
2. Abre la pestaña **Actions → Build → último run → Artifacts**.
3. Descarga `pngtuber-lite-windows-x86_64` y descomprime
   `pngtuber-lite.exe`.

> `cargo build --release` ya produce el binario final. El `.exe` usa el
> subsistema de Windows: al abrirlo **no** aparece una ventana de consola.

## Primer arranque

La app abre solo la ventana de ajustes, con una **previsualización** del avatar
en el panel derecho. Todavía no hay ninguna ventana transparente encima del
escritorio.

1. En **Avatar**, elige el PNG de boca cerrada y el de boca abierta. La
   previsualización cambia al ritmo de tu voz.
2. En **Micrófono**, elige el dispositivo (o deja el predeterminado del
   sistema) y comprueba que el estado diga *Escuchando*.
3. En **Nivel de audio**, habla y observa el medidor: la línea ámbar es el
   umbral. Ajusta **Sensibilidad** hasta que la etiqueta cambie entre
   `SILENCIO` y `HABLANDO` con tu voz normal, sin abrirse con ruido de fondo.
4. Pulsa **Mostrar overlay** (se habilita al importar al menos un PNG). Ahí
   nace la ventana transparente real, en la posición y el tamaño guardados.

Los cambios se guardan solos en `%APPDATA%\pngtuber-lite\settings.json`, junto
a una copia de los PNG en `avatars\`.

## Uso diario

- **Arrastrar:** botón izquierdo sobre el avatar.
- **Redimensionar:** arrastra cualquiera de los cuatro lados o esquinas.
- **Abrir ajustes:** clic derecho sobre el avatar.
- **Ajustes:** la ventana no se puede cerrar; usa **Minimizar** o
  **Maximizar** de Windows. Sigue en la barra de tareas.
- **Ocultar de la pantalla sin cerrar:** en **Ventana** o en la preview, botón
  **Ocultar**. El avatar se aparca fuera de los monitores y **sigue
  renderizando para OBS**. **Mostrar** lo devuelve a su última posición y lo
  sube por encima de la ventana de ajustes.
- **Salir del todo:** botón **Salir de la app** en el pie de ajustes.

> Minimizar con la barra de Windows congela la captura de OBS. Para OBS usa
> **Ocultar**: la ventana se mueve fuera del escritorio pero sigue dibujando
> frames.

## Captura en OBS

El overlay está pensado para `Captura de ventana` con transparencia real:

1. En OBS, añade una fuente **Captura de ventana**.
2. Como método elige **Windows 10 (1903 y posteriores)** (WGC). El método
   automático usa BitBlt para esta clase de ventana y sale en negro.
3. En ventana elige **PNGTuber Lite · avatar**. La ventana de ajustes aparece
   como **PNGTuber Lite · ajustes**.
4. Si la fuente ya existía de antes (cuando el título estaba vacío), **borra
   la fuente y créala de nuevo**: la selección guardada apuntaba al título
   viejo.

Notas: el overlay no muestra botón en la barra de tareas, pero puede aparecer
en Alt+Tab (limitación conocida de quitar `WS_EX_TOOLWINDOW`). Con **Ocultar**
el avatar se aparca fuera de los monitores y OBS lo sigue capturando.

> Si con WGC quedara una línea de 1 px en el borde, activá **Client Area**
> en las propiedades de la fuente de OBS: la app ya quita el borde DWM y las
> esquinas redondeadas en Windows 11.

## Presupuesto

El objetivo del proyecto es medido, no supuesto. Límites del MVP en
`docs/benchmarks.md`: RAM privada ≤ 40 MB, working set ≤ 70 MB, CPU en reposo
≤ 1,5 % de un núcleo, CPU hablando ≤ 3 %, arranque < 400 ms y binario ≤ 16 MB.
El piso de GPUI en Windows (hilo de VSync) es conocido y se declara.

## Estructura

```text
src/
├── main.rs          arranque: solo ajustes, sin overlay
├── app.rs           AppState + bomba de eventos de audio
├── window.rs        opciones, apertura de ajustes y overlay perezoso
├── win32.rs         shim aislado: arrastre, redimensionado, siempre visible y captura para OBS
├── shared/ui.rs     tokens y controles propios
└── features/
    ├── avatar/      carga, copia y decodificación de PNG
    ├── audio/       cpal, RMS y VAD puro con tests
    └── settings/    esquema v1, escritura atómica, UI y preview
```

## Documentación

- `docs/architecture.md`: módulos, hilos y fronteras.
- `docs/design-system.md`: tokens y reglas visuales.
- `docs/benchmarks.md`: cómo se mide y qué se midió.
- `docs/development.md`: CI, lockfile y convenciones.
- `odd/tasks/2026-11-25-pngtuber-lite-mvp.md`: plan ODD y evidencia.

## Licencia

MIT — ver [LICENSE](LICENSE). El binario no empaqueta fuentes ni iconos de terceros.
