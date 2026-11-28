# Ajustes sin cierre: solo minimizar o maximizar

- **Estado:** `checking` · CI en verde (run `45736058660`) · Verificación Win32 directa correcta
- **Fecha:** 2026-11-26
- **Origen:** prueba humana del `.exe` del run `45734803073`
- **Fase anterior:** [`2026-11-26-overlay-robustez.md`](2026-11-26-overlay-robustez.md)

## Qué pidió el humano

Quitar la X interna de la cabecera de ajustes. La ventana de configuración no
debe poder ocultarse ni cerrarse: solo minimizarse o maximizarse, como una
ventana normal. Si se ocultaba el avatar y se cerraban los ajustes, no quedaba
ninguna forma de volver. El botón **Salir de la app** sigue siendo la única
salida real.

## Decisiones

- **D1 · Sin X interna.** Se elimina `icon_button("close-settings", "×")` de
  la cabecera. La cabecera queda con marca, título y subtítulo.
- **D2 · Cierre nativo bloqueado.** `Window::on_window_should_close` devuelve
  `false` para la ventana de ajustes. El botón X del marco, `Alt+F4`, el menú
  del sistema y “Cerrar” de la barra de tareas no cierran la ventana.
- **D3 · X nativa en gris.** `win32::disable_close` elimina `SC_CLOSE` del
  menú del sistema con `DeleteMenu`. El usuario ve que la X no está
  disponible; minimizar y maximizar siguen activos.
- **D4 · Ventana redimensionable.** `is_resizable: true` e
  `is_minimizable: true`. El cuerpo ya tiene scroll; el contenido se centra y
  se limita a 760 px para que maximizar no estire las cards.
- **D5 · Recuperación.** Con ajustes minimizados, el clic derecho sobre el
  avatar llama `activate_window`, que en Windows hace `SW_RESTORE` si la
  ventana está minimizada. La barra de tareas también la restaura.

## Criterios de aceptación

- [x] No hay X en la cabecera de ajustes.
- [x] La X nativa aparece deshabilitada y no cierra la ventana.
- [x] `WM_CLOSE`/`Alt+F4` no cierra la ventana de ajustes.
- [ ] Minimizar y maximizar funcionan; el contenido se adapta y sigue con
      scroll.
- [ ] Clic derecho sobre el avatar restaura ajustes si estaban minimizados.
- [ ] **Salir de la app** sigue cerrando todo.
- [x] `fmt`, `clippy -D warnings`, tests y release pasan en Actions.

## Evidencia

- CI en verde en el run [`45736058660`](https://github.com/roldyoran/pngtuber-lite/actions/runs/45736058660):
  `fmt`, `clippy --all-targets -- -D warnings`, tests y release. Artefacto
  `pngtuber-lite-windows-x86_64` (11.643.392 B = 11,1 MiB).
- Prueba Win32 sobre el `.exe` del CI, con `%APPDATA%` aislado:
  - `STYLE` de la ventana de ajustes: `MINIMIZEBOX=True`, `MAXIMIZEBOX=True`,
    `THICKFRAME=True`.
  - `SC_CLOSE` ya no existe en el menú del sistema (`GetMenuState` devuelve
    `0xFFFFFFFF`).
  - `WM_CLOSE` enviado a la ventana: `IsWindow=true`, `IsWindowVisible=true`
    y el proceso sigue vivo.
- Pendiente de prueba humana: minimizar/maximizar y restaurar desde la barra
  de tareas o con clic derecho sobre el avatar.
