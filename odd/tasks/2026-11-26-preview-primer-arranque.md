# Primer arranque: preview lateral y overlay bajo demanda

- **Estado:** `implementing` · Verificación **solo por GitHub Actions**
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

## Criterios de aceptación

- [ ] Al arrancar no existe ninguna ventana transparente.
- [ ] Ajustes muestra el panel de preview con el PNG actual y el estado de
      boca.
- [ ] **Mostrar** crea el overlay real; antes de pulsarlo no hay overlay.
- [ ] **Ocultar** sigue aparcando el avatar fuera de pantalla y **Mostrar** lo
      devuelve exactamente a la posición donde estaba (o lo crea ahí).
- [ ] El overlay no tiene botón ni acción de pantalla completa.
- [ ] `Ctrl+Alt+P` ya no existe.
- [ ] Arrastrar lados y esquinas cambia el tamaño; se guarda al soltar.
- [ ] La rueda desplaza el cuerpo de ajustes hasta la última sección.
- [ ] Ajustes se redimensiona, se maximiza y no se cierra.
- [ ] `fmt`, `clippy -D warnings`, tests y release pasan en Actions.

## Evidencia

- Pendiente del run de Actions.
