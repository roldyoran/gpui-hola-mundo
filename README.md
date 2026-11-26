# PNGTuber Lite

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
- Arrastrable con el botón izquierdo, redimensionable por los bordes y con
  pantalla completa; clic derecho reabre los ajustes.

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
4. Pulsa **Mostrar overlay**. Ahí nace la ventana transparente real, en la
   posición y el tamaño guardados.

Los cambios se guardan solos en `%APPDATA%\pngtuber-lite\settings.json`, junto
a una copia de los PNG en `avatars\`.

## Uso diario

- **Arrastrar:** botón izquierdo sobre el avatar.
- **Redimensionar:** arrastra cualquiera de los cuatro lados o esquinas.
- **Pantalla completa:** botón **Pantalla completa** en la preview o en
  **Ventana**; **Restaurar** devuelve el tamaño anterior. Con el overlay a
  pantalla completa, un clic derecho lo restaura y abre ajustes.
- **Abrir ajustes:** clic derecho sobre el avatar.
- **Ajustes:** la ventana no se puede cerrar; usa **Minimizar** o
  **Maximizar** de Windows. Sigue en la barra de tareas.
- **Ocultar de la pantalla sin cerrar:** en **Ventana** o en la preview, botón
  **Ocultar**. El avatar se aparca fuera de los monitores y **sigue
  renderizando para OBS**. **Mostrar** lo devuelve a su última posición.
- **Salir del todo:** botón **Salir de la app** en el pie de ajustes.

> Minimizar con la barra de Windows congela la captura de OBS. Para OBS usa
> **Ocultar**: la ventana se mueve fuera del escritorio pero sigue dibujando
> frames.

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
├── win32.rs         shim aislado: arrastre, redimensionado y siempre visible
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

MIT. El binario no empaqueta fuentes ni iconos de terceros.
