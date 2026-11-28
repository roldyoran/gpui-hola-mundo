# Primer arranque: preview lateral y overlay bajo demanda

- **Estado:** `checking` · CI en verde (run `45740820917`) · Falta prueba humana de resize, posición y orden Z
- **Fecha:** 2026-11-26
- **Origen:** prueba humana con una persona ajena al proyecto
- **Fase anterior:** [`2026-11-26-settings-sin-cerrar.md`](2026-11-26-settings-sin-cerrar.md)

## Qué pidió el humano

Al abrir la app por primera vez, la ventana transparente aparecía encima de los
ajustes y la persona no sabía qué era, que se podía arrastrar ni cómo usarla.
Ahora el arranque debe mostrar solo los ajustes con una **previsualización** del
avatar; el overlay real no existe hasta pulsar **Mostrar**. Además:

- La previsualización va en un **panel lateral** dentro de ajustes.
- Se elimina el atajo global `Ctrl+Alt+P`; se conservan **Ocultar** y
  **Mostrar**.
- El overlay se redimensiona **arrastrando los bordes**. No tiene pantalla
  completa: solo se muestra u oculta, siempre en la última posición.
- Los ajustes siguen siendo libres de redimensionar y maximizar, y no se
  cierran. El cuerpo debe poder desplazarse con la rueda.

## Decisiones

- **D1 · Arranque sin overlay.** `main` ya no crea la ventana del avatar.
  Siempre abre ajustes. El hilo de audio arranca igual y el VAD sigue
  actualizando la previsualización.
- **D2 · Overlay perezoso.** `window::show_avatar_overlay` crea la ventana la
  primera vez que se pulsa **Mostrar**. Después, **Mostrar** solo la
  desaparca. Si el overlay todavía no existe, no hay ventana transparente.
- **D3 · Preview en panel lateral.** El cuerpo de ajustes usa dos columnas
  cuando hay ancho suficiente: contenido a la izquierda (máximo 760 px) y
  preview a la derecha (300 px). Si la ventana es estrecha, la preview se apila
  arriba para no romper el layout.
- **D4 · Tablero de transparencia.** La preview se dibuja sobre un tablero
  oscuro de 8×8 para que se entienda que el fondo real es transparente. La
  imagen cambia entre boca cerrada y abierta con el VAD, como el overlay.
- **D5 · Sin atajo global.** Se eliminan `RegisterHotKey`, el hilo
  `pngtuber-hotkey`, `WM_HOTKEY` y el estado `AVATAR`. Quedan los botones
  **Ocultar**/**Mostrar**.
- **D6 · Redimensionado manual.** El overlay no usa el borde nativo. Ocho
  zonas invisibles (4 lados y 4 esquinas) inician un arrastre con
  `SetWindowPos`, igual que el movimiento, sin bucles modales. El tamaño
  mínimo es 120 px y el resultado se persiste en `settings.json`.
- **D7 · Sin pantalla completa (ajuste posterior).** El humano pidió quitar la
  acción de pantalla completa del overlay: solo debe existir
  **Mostrar**/**Ocultar**, y al mostrarlo debe reaparecer donde estaba. Se
  elimina `avatar_maximized`, `set_fullscreen` y sus botones.
- **D8 · Scroll real.** El contenedor con scroll no puede ser un flex en fila:
  estiraba el contenido a la altura visible y por eso la rueda no tenía nada
  que desplazar. Pasa a ser un bloque con un hijo en columna.
- **D9 · Ajustes siempre accesibles.** La ventana sigue sin cerrarse y se
  puede minimizar/maximizar; al pulsar **Mostrar** o el clic derecho del
  avatar se reactiva.
- **D10 · Mostrar siempre al frente.** `win32::raise` usa `HWND_TOP` al crear
  o desaparcar el overlay: queda por encima de ajustes sin robarle el foco.
  Así **Ocultar → Mostrar** no deja el avatar escondido detrás del menú.
- **D11 · Sin consola en release.** `main.rs` declara
  `windows_subsystem = "windows"` fuera de debug. El artefacto del CI ya era
  `--release`; lo que se veía era la consola del subsistema, no un build debug.

## Criterios de aceptación

- [x] Al arrancar no existe ninguna ventana transparente.
- [x] Ajustes muestra el panel de preview con el PNG actual y el estado de
      boca.
- [x] **Mostrar** crea el overlay real; antes de pulsarlo no hay overlay.
- [ ] **Ocultar** sigue aparcando el avatar fuera de pantalla y **Mostrar** lo
      devuelve exactamente a la posición donde estaba (o lo crea ahí) y por
      encima de ajustes.
- [x] El overlay no tiene botón ni acción de pantalla completa.
- [x] `Ctrl+Alt+P` ya no existe.
- [ ] Arrastrar lados y esquinas cambia el tamaño; se guarda al soltar.
- [x] La rueda desplaza el cuerpo de ajustes hasta la última sección.
- [x] Ajustes se redimensiona, se maximiza y no se cierra.
- [x] El `.exe` de release no abre ninguna ventana de consola.
- [x] `fmt`, `clippy -D warnings`, tests y release pasan en Actions.

## Evidencia

- CI en verde en el run [`45740820917`](https://github.com/roldyoran/pngtuber-lite/actions/runs/45740820917):
  `fmt`, `clippy --all-targets -- -D warnings`, tests y release. Artefacto
  `pngtuber-lite-windows-x86_64` (11.685.376 B = 11,1 MiB).
- Cabecera PE del `.exe` del run: `SUBSYSTEM = 2` (`WINDOWS_GUI`), no `3`
  (`CONSOLE`). Es un build `--release` de subsistema Windows: no abre consola.
- Prueba Win32 sobre el `.exe` del CI, con `%APPDATA%` aislado y dos PNG de
  prueba:
  - Arranque: **una sola** ventana visible (ajustes, 1076×759) y ninguna
    ventana de 320×320. El overlay no existe.
  - Preview: se ve el PNG con el tablero de transparencia y el chip de estado;
    no aparece ningún botón de pantalla completa.
  - **Mostrar overlay** con clic real: crea una ventana de 336×328 en
    `(92, 96)`, que corresponde a 320×320 en `(100, 100)` guardados más el
    marco invisible. La preview cambia a **Ocultar**/**Mostrar**.
  - Rueda sobre la columna izquierda: la captura cambia en 5360 muestras
    (paso de 4 px), es decir, el cuerpo sí se desplaza hasta **Ventana**.
  - `WM_CLOSE` sobre ajustes no cierra la ventana (comportamiento heredado de
    la fase anterior, sin cambios en este código).
- **Mostrar al frente:** `win32::raise` se llama al crear y al desaparcar el
  overlay. Usa `HWND_TOP` y `SWP_NOACTIVATE`, así queda encima de ajustes sin
  quitarle el foco. Pendiente de confirmación humana con `Siempre visible`
  desactivado.
- Pendiente de prueba humana: arrastrar bordes, confirmar que **Ocultar** →
  **Mostrar** devuelve el avatar al mismo píxel y que queda por encima de la
  ventana de ajustes.
