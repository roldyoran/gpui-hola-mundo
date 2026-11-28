# Robustez del overlay: arrastre, tamaño libre y modo OBS

- **Estado:** `checking` · Implementado · CI en verde (run `45734803073`) · Prueba humana pendiente
- **Fecha:** 2026-11-26
- **Origen:** prueba humana del `.exe` del run `45732240210`
- **Fase anterior:** [`2026-11-25-ui-shadcn.md`](2026-11-25-ui-shadcn.md)

## Qué reportó el humano

1. Al alternar clic izquierdo y derecho sobre el avatar, la aplicación se
   cierra por completo. No debe pasar nunca.
2. Quiere minimizar/ocultar el avatar del escritorio porque solo lo usa en
   OBS, pero no encuentra cómo.
3. Quiere cambiar **ancho y alto por separado**, porque sus PNG tienen
   proporciones distintas.

## Diagnóstico

- El arrastre actual llama `SendMessageW(WM_NCLBUTTONDOWN)` desde el handler
  de GPUI. Windows abre un bucle modal de movimiento **dentro** del handler;
  un clic derecho durante ese bucle encola otro evento de GPUI y puede anidar
  bucles. Es el riesgo ya anotado en el plan del MVP.
- `set_height` conserva la proporción, así que no hay forma de ajustar el
  ancho.
- El overlay es `WindowKind::PopUp` sin taskbar. Minimizarlo con
  `ShowWindow(SW_MINIMIZE)` haría que OBS dejara de recibir frames: la captura
  se congela o se pone negra. Para “no ocupar pantalla y seguir en OBS” hay
  que **aparcarlo fuera de todos los monitores**, no minimizarlo.

## Decisiones

- **D1 · Arrastre manual.** `begin_drag` guarda el cursor y la posición de la
  ventana; una tarea de GPUI consulta `GetCursorPos` y
  `GetAsyncKeyState(VK_LBUTTON)` cada 16 ms y mueve con `SetWindowPos`.
  `ReleaseCapture`/`SendMessageW` desaparecen. Sin bucles modales no hay
  anidamiento.
- **D2 · Clic derecho seguro.** Mientras `dragging` está activo, el clic
  derecho se ignora. Al soltar, la tarea persiste la posición y baja el flag.
- **D3 · Tamaño libre.** `set_width` y `set_height` cambian cada eje por
  separado; los steppers usan pasos de 40 px y los límites existentes
  (120–1200 × 120–900).
- **D4 · Modo OBS.** Botones **Ocultar de la pantalla** y **Traer a la
  pantalla**. Ocultar mueve la ventana fuera del escritorio virtual con
  `SetWindowPos`; sigue renderizando y OBS la puede capturar. No se persiste
  la posición off-screen: el `settings.json` conserva la posición visible.
- **D5 · Restaurar sin depender de clic derecho.** Atajo global
  `Ctrl+Alt+P` registrado con `RegisterHotKey` en un hilo con `GetMessageW`;
  alterna aparcado/visible. Si el registro falla, los botones de ajustes
  siguen funcionando.
- **D6 · Minimizar real no se ofrece.** Se documenta que minimizar congela la
  captura de OBS; el modo OBS es la alternativa correcta.

## Criterios de aceptación

- [ ] Alternar clic izquierdo/derecho sobre el avatar no cierra la app.
- [ ] Arrastrar mueve la ventana y persiste la posición al soltar.
- [ ] El clic derecho abre ajustes y no interfiere con un arrastre activo.
- [ ] Ancho y alto se ajustan por separado y redimensionan el overlay.
- [ ] **Ocultar de la pantalla** saca el avatar de todos los monitores y
      **Traer a la pantalla** lo devuelve a la última posición visible.
- [ ] `Ctrl+Alt+P` alterna visible/aparcado sin abrir ajustes.
- [ ] La posición off-screen no se guarda en `settings.json`.
- [ ] `fmt`, `clippy -D warnings`, tests y build pasan en Actions.
- [ ] El `.exe` sigue por debajo de 16 MB.

## Evidencia

- CI en verde en el run [`45734803073`](https://github.com/roldyoran/pngtuber-lite/actions/runs/45734803073):
  `fmt`, `clippy --all-targets -- -D warnings`, tests y release; artefacto
  `pngtuber-lite-windows-x86_64` (11.640.320 B = 11,1 MiB).
- Arranque aislado con `%APPDATA%` temporal: el proceso sigue vivo, abre la
  ventana de ajustes y crea el hilo `pngtuber-hotkey` sin caer.
- Pendiente de prueba humana: clics alternos, arrastre, ancho/alto, Ocultar /
  Mostrar y `Ctrl+Alt+P`.

