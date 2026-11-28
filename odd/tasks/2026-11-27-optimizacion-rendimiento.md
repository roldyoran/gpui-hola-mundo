# Optimización de rendimiento y código

- **Estado:** `checking` · OD-14r, OD-15, OD-17 y OD-20 implementados en el working tree, pendientes de CI
- **Fecha:** 2026-11-27
- **Origen:** revisión de rendimiento pedida por el humano + incidencia de captura en OBS
- **Fase anterior:** [`2026-11-26-overlay-robustez.md`](2026-11-26-overlay-robustez.md)
- **Restricción vigente:** **D5** — este PC no compila. El CI de GitHub Actions
  es el único verificador (`fmt`, `clippy -D warnings`, `test`, `build --release`).

## Resultado esperado

Bajar el coste por frame y por evento, reducir picos de memoria al importar PNG
y hacer que el overlay sea seleccionable en OBS, **sin cambiar el aspecto ni el
comportamiento visible** (salvo la integración con OBS).

Regla del repo: **sin número no hay optimización**. Las mejoras de higiene de
esta fase (notificaciones, copias de PNG, escrituras redundantes) no cambian el
producto y se validan con el CI; la medición (OD-13) sigue siendo la puerta
para las optimizaciones condicionales.

## Alcance

| Estado | Slice | Qué hace |
| --- | --- | --- |
| **Implementado** | OD-15 | PNG: una copia menos y sin leer todo el archivo |
| **Implementado** | OD-17 | Trabajo redundante: lote de eventos, comparar dispositivos, no persistir sin cambio, quitar doble notify |
| **Implementado** | OD-14r | Notificaciones inútiles del medidor: solo con audio `Running`, umbral 0,25 dB |
| **Implementado** | OD-20 | Overlay detectable por OBS (y transparencia con WGC) |
| **Diferido** | OD-13 | Medición base de CPU/GPU/arranque y RAM con PNG grandes |
| **Diferido** | OD-16 | Caché de umbrales/tiempos del VAD |
| **Diferido** | OD-21 | Cadencia del medidor (45→66 ms) y tablero por `canvas` |
| **Diferido** | OD-18 | Canal de eventos sin pérdidas + error de arranque del hilo de audio |
| **Diferido** | OD-19 | `panic = "abort"` en release |
| **Condicional** | — | Carga de PNG en segundo plano y downscale (solo si OD-13 lo pide) |

## Hallazgos de la investigación

### Código de la app y GPUI

| # | Hallazgo | Evidencia | Impacto |
| --- | --- | --- | --- |
| H1 | El medidor notifica cada 45 ms (≈22 Hz) y **cada notify reconstruye todo el árbol de ajustes**. | `src/features/settings/view.rs:25`, `:60-75`, `:988-1089`; `src/shared/ui.rs:190-205` | CPU con ajustes visibles |
| H2 | GPUI no permite aislar el medidor en una vista hija: marcar una vista dirty marca a sus ancestros, y `refreshing` desactiva la caché de los descendientes. | `gpui-pre-0.3.14/src/window.rs:2563`; `src/view.rs:549` | Descarta extraer `MeterView` |
| H3 | El umbral de notify es 0,05 dB ≈ 0,03 px de barra: se redibuja por cambios invisibles. | `settings/view.rs:88-101` | Notificaciones de más |
| H4 | El ticker corre también con el audio parado o en error. | `settings/view.rs:60-75` | Trabajo inútil en reposo |
| H5 | `decode_png(&bytes)` hace `bytes.to_vec()` y `Image::from_bytes` guarda ese `Vec`: **dos copias** del PNG. | `src/features/avatar/mod.rs:65-73`; `gpui-pre-0.3.14/src/platform.rs:3539` | RAM y tiempo con PNG grandes |
| H6 | `import_png` lee todo el archivo para validar 8 bytes y luego lo reescribe. | `src/features/avatar/mod.rs:51-62` | Pico de RAM y bloqueo de UI |
| H7 | La importación y la lectura de PNG corren en el hilo de GPUI. | `settings/view.rs:115-135`; `avatar/mod.rs:51-73` | Micro-cortes |
| H8 | Un `Mouth` descartado (canal acotado a 8) desincroniza la boca hasta la siguiente transición. | `src/app.rs:44`; `audio/stream.rs:168-170` | Robustez (diferido) |
| H9 | `Devices` devuelve “cambio” aunque la lista sea igual; los setters persisten y reaplican audio sin cambio. | `src/app.rs:200-204`, `:125-183` | Redraws y escrituras de más |
| H10 | `update_state` notifica dos veces: el observer de la propia vista ya notifica. | `settings/view.rs:103-109` | Redraw duplicado |
| H11 | El VAD recalcula umbrales y frames de ataque/cierre en cada buffer (~100/s). | `audio/vad.rs:63-107` | Micro (diferido) |
| H12 | Los RGBA de ambos PNG viven en la caché de GPUI; dos 2048² ≈ 32 MB. | `gpui-pre-0.3.14/src/assets.rs:43-86` | RAM si el humano usa PNG grandes |
| H13 | GPUI solo dibuja si el invalidator está dirty y limita ventanas inactivas a ~30 fps; el piso de VSync no es optimizable. | `gpui-pre-0.3.14/src/window.rs:2062`, `platform.rs:2896-2898` | Contexto |

### Incidencia OBS (ventana transparente no detectable)

| # | Hallazgo | Evidencia |
| --- | --- | --- |
| H14 | **OBS rechaza cualquier ventana con `WS_EX_TOOLWINDOW`.** No es un problema de método de captura: la ventana ni siquiera entra en la lista. GPUI crea el overlay `PopUp` con `WS_EX_TOOLWINDOW \| WS_EX_TOPMOST`; la ventana de ajustes es `Normal` con `WS_EX_APPWINDOW`, por eso sí aparece. | `libobs/util/windows/window-helpers.c:292-312` (`check_window_valid`); `gpui-pre-windows-0.3.14/src/window.rs:494-495` (PopUp) y `:502-510` (Normal/APPWINDOW) |
| H15 | Ambas ventanas se crean con **título vacío** (`titlebar: None`). OBS arma la selección como `título:clase:exe` y, con la prioridad por defecto (`Title`), puntúa cualquier título vacío como coincidencia; al recargar la fuente podría reenganchar la ventana de ajustes en vez del overlay. | `gpui-pre-windows-0.3.14/src/window.rs:487-492`; `window-helpers.c:233-276` (`add_window`) y `:450-456` (`window_rating`, prioridad `TITLE`) |
| H16 | El método **Automático** de OBS solo usa WGC para clases conocidas (navegadores); para el resto elige **BitBlt**. El overlay usa DirectComposition con `WS_EX_NOREDIRECTIONBITMAP` y swapchain `DXGI_ALPHA_MODE_PREMULTIPLIED`: BitBlt no puede capturarlo. Hay que elegir **Windows 10 (1903 y posteriores)**. | `plugins/win-capture/window-capture.c:140-169` (`choose_method`); `gpui-pre-windows-0.3.14/src/window.rs:516`; `src/directx_renderer.rs:1362` |
| H17 | El camino WGC de OBS mezcla con **alfa premultiplicado** (`GS_BLEND_ONE, GS_BLEND_INVSRCALPHA`), que es el formato del swapchain de GPUI. Debería conservar la transparencia; hay que validarlo en el equipo del humano (GPU/driver/versión de OBS). | `libobs-winrt/winrt-capture.cpp:528-558` |

## Diagnóstico de la incidencia OBS

1. La ventana del overlay **existe y es visible**, pero OBS la filtra por
   `WS_EX_TOOLWINDOW` antes de mostrarla en la lista (`check_window_valid`,
   H14). Por eso ninguna opción de captura la detecta: el filtro ocurre al
   poblar el desplegable, no al capturar.
2. La ventana de ajustes no lleva ese estilo (`WS_EX_APPWINDOW`), por eso sí se
   detecta.
3. Aun quitando el filtro, si se deja el método **Automático**, OBS elegirá
   BitBlt para esta clase y la captura será negra (H16). El método debe ser
   WGC.
4. Los títulos vacíos pueden hacer que OBS reencuentre la ventana equivocada
   al recargar la fuente (H15). Hay que poner un título único.

## Decisiones pendientes (OBS)

| # | Decisión | Opciones | Recomendación |
| --- | --- | --- | --- |
| D-OBS-1 | Cómo quitar `WS_EX_TOOLWINDOW` | (A) siempre al crear el overlay; (B) ajuste “Detectable por OBS” con valor por defecto; (C) no quitarlo y documentar el parche de `obs.dll` | **A**: es el caso de uso principal; C no es aceptable como producto |
| D-OBS-2 | Conservar “sin barra de tareas” | (A) `ITaskbarList::DeleteTab` al crear la ventana (COM ya inicializado por GPUI); (B) aceptar el botón en la barra | **A**. Añade `Win32_UI_Shell` + `Win32_System_Com` a `windows`; no añade crates |
| D-OBS-3 | Alt+Tab | Sin `WS_EX_TOOLWINDOW`, la ventana puede aparecer en Alt+Tab; `DeleteTab` no lo evita. `WS_EX_TOOLWINDOW` es la única forma estándar de ocultarla de ambos. | Aceptarlo y documentarlo; si molesta, se puede evaluar un dueño oculto en otra fase (no en esta) |
| D-OBS-4 | Si WGC devuelve fondo negro | (A) decidir un modo alternativo (color key / Spout / renderer por capas); (B) no soportar OBS en ese equipo | Volver a `deciding` **solo si la prueba humana falla**; no implementar preventivamente |

## Slices aprobados en principio

### OD-14r · Notificaciones inútiles del medidor

- **Cambios:**
  - El ticker solo actualiza mientras `AudioStatus::Running`. En
    `Stopped`/`Starting`/`Error` fija el suelo, notifica una sola vez y duerme
    en un intervalo largo (≈500 ms) hasta que el estado cambie.
  - Umbral de notify 0,05 → **0,25 dB** (sigue por debajo de 1 px).
  - No se cambia la cadencia (45 ms), ni el suavizado, ni el tablero. Eso queda
    en OD-21, diferido.
- **Archivos:** `src/features/settings/view.rs`.
- **Tests (puros):** extraer el suavizado a `smooth_db` si la lógica se puede
  probar sin `Context`; cubrir suelo y convergencia. Si no, el CI + prueba
  humana bastan para este cambio.
- **DoD:** CI verde; con audio apagado no hay redraws del medidor; hablando se
  ve igual.

### OD-15 · PNG: una copia menos y sin leer todo el archivo

- **Cambios:**
  - `decode_png(bytes: Vec<u8>)` consume el `Vec`; `reload_avatars` pasa la
    propiedad. Elimina el `to_vec()` de H5.
  - `import_png`: abrir, leer **8 bytes** con `read_exact` para validar la
    firma y `fs::copy` al destino. Elimina la lectura completa de H6. Un
    archivo de menos de 8 bytes se informa como “no es un PNG válido”.
  - `is_png(bytes)` se conserva para tests y para el helper de firma.
- **Archivos:** `src/features/avatar/mod.rs`, `src/app.rs`.
- **DoD:** CI verde; importar un PNG grande ya no duplica su tamaño en RAM
  (revisión de código; la medición con número queda en OD-13).
- **Fuera de este slice:** cargar en segundo plano y downscale (condicional).

### OD-17 · Trabajo redundante

- **Cambios:**
  - `drain_audio`: tras `recv().await`, drenar con `try_recv` y aplicar el lote
    en orden con **un solo** `cx.notify()`.
  - `AudioEvent::Devices`: devolver `changed` solo si `self.devices != list`.
  - Setters: salir sin persistir ni reaplicar si el valor no cambia
    (`set_audio_enabled`, `set_device`, `set_sensitivity`,
    `set_always_on_top`, `set_width`, `set_height`). En `set_avatar`, recargar
    siempre (el archivo pudo cambiar), pero persistir solo si cambió la ruta.
  - Quitar el `cx.notify()` final de `update_state` (H10).
- **Archivos:** `src/app.rs`, `src/features/settings/view.rs`.
- **DoD:** CI verde; pulsar `+` en sensibilidad 100 no escribe
  `settings.json`; refrescar dispositivos sin cambios no redibuja la ventana.
  (Comprobación por código y prueba humana; no hay test de UI/fs.)

## New slice — OD-20 · Overlay detectable por OBS

- **Objetivo:** que `Window Capture` (método WGC) liste y capture el overlay,
  manteniendo la transparencia y sin botón en la barra de tareas.
- **Cambios:**
  1. Título único: `"PNGTuber Lite · avatar"` para el overlay y
     `"PNGTuber Lite · ajustes"` para ajustes, con
     `Window::set_window_title` (`gpui-pre-0.3.14/src/window.rs:3439`).
     Evita la colisión de títulos vacíos de H15.
  2. `win32::prepare_capture(window)`:
     - `GetWindowLongPtrW(hwnd, GWL_EXSTYLE)`;
     - `SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex & !WS_EX_TOOLWINDOW)`;
     - `SetWindowPos(..., SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE |
       SWP_NOZORDER | SWP_NOACTIVATE)`;
     - reaplicar `set_topmost` y `raise` (el cambio de frame puede alterar el
       orden Z).
  3. Ocultar el botón de la barra: `ITaskbarList` con
     `CoCreateInstance(&TaskbarList)`, `HrInit()` y `DeleteTab(hwnd)`.
     GPUI ya hizo `OleInitialize` en el hilo principal
     (`gpui-pre-windows-0.3.14/src/platform.rs:186`). Añadir a `Cargo.toml` las
     features `Win32_UI_Shell` y `Win32_System_Com` de `windows 0.62`.
  4. Llamar a `prepare_capture` justo después de crear el overlay y de fijar el
     título, antes de `remember_origin`/`raise`.
  5. Documentar en `README.md` el flujo de OBS: fuente **Captura de ventana** →
     método **Windows 10 (1903 y posteriores)** → ventana
     **PNGTuber Lite · avatar**. Si la fuente existía antes, **borrarla y
     crearla de nuevo** (la selección guardada apuntaba a un título vacío).
- **Criterios de aceptación:**
  - `GWL_EXSTYLE & WS_EX_TOOLWINDOW == 0` en el HWND del overlay.
  - OBS lista “PNGTuber Lite · avatar” con la app abierta (también aparcada).
  - WGC muestra el avatar con transparencia (sin fondo negro).
  - La ventana de ajustes sigue siendo detectable y el overlay sigue encima.
  - Sin botón en la barra de tareas. Alt+Tab puede listarlo (documentado).
  - CI verde.
- **Riesgos:** Alt+Tab (D-OBS-3); alfa WGC variable por driver (D-OBS-4);
  `DeleteTab` puede no estar disponible en algún caso COM (fallback: aceptar el
  botón y documentarlo); OBS debe reabrir propiedades para refrescar la lista.
- **No se hace aquí:** parchear `obs.dll`, una segunda ventana de captura,
  Spout/NDI ni cambiar el renderer a `WS_EX_LAYERED`.

## Slices diferidos (no se implementan en esta fase)

- **OD-13 · Medición base:** 4 escenarios (reposo, hablando con ajustes,
  minimizado, PNG 2048²) y tabla en `docs/benchmarks.md`. Es la puerta de OD-21
  y del downscale.
- **OD-16 · VAD:** cachear `attack_frames`/`release_frames` y umbrales en
  `new`/`set_sensitivity`; los 9 tests actuales deben pasar sin tocarlos.
- **OD-21 · Medidor caro:** 45→66 ms con suavizado por tiempo y tablero por
  `canvas` + `fill` (verificado: ambos exportados por gpui). Solo si OD-13
  muestra CPU por encima del presupuesto.
- **OD-18 · Eventos:** canal sin límite y un solo error de cpal por stream;
  manejar el fallo de `thread::Builder::spawn`.
- **OD-19 · Release:** `panic = "abort"` si GPUI no depende de `catch_unwind`.

## No se hará (descartado con motivo)

| Idea | Motivo |
| --- | --- |
| Extraer el medidor a una vista hija | H2: el ancestro se re-renderiza igual |
| Animar el medidor con springs/`with_animation` | GPUI re-renderiza la vista en cada frame: 60 Hz |
| RMS en `f32` o submuestreo | No es el cuello de botella |
| Cambiar el piso de VSync de GPUI | Coste aceptado (D2) |
| Pedir al usuario que parchee `obs.dll` | El producto debe funcionar sin parches |
| Quitar `WS_EX_TOOLWINDOW` y dejar el método Automático | BitBlt no captura DirectComposition (H16) |
| Segunda ventana solo para OBS | Duplica render, memoria y estado |

## Protocolo de medición (para OD-13)

El de `docs/benchmarks.md`: RAM/hilos a los 5 s; CPU por delta de
`TotalProcessorTime` en 60 s; GPU en el Administrador de tareas; arranque con
`Measure-Command`. Todo número va a la tabla con fecha, máquina, resolución y
comando. La prueba de OBS es humana: seleccionar la ventana, método WGC,
confirmar transparencia y orden Z.

## Verificación

Solo GitHub Actions (D5):

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings   (si cambia src/ o Cargo.*)
cargo test                                    (si cambia src/ o Cargo.*)
cargo build --release
```

Cada push debe salir con la superficie de API ya contrastada contra
`~/.cargo/registry` (gpui-pre 0.3.14, gpui-kit 0.11.0, windows 0.62.2). Al
cerrar: `docs/architecture.md`, `docs/design-system.md` y `README.md` al día.

## Log

- **L1.** Revisión estática del código y de las fuentes reales de
  `gpui-pre-0.3.14`, `gpui-kit-0.11.0` y `cpal-0.18.2`. Hallazgos H1–H13.
- **L2.** Humano acota el alcance: OD-15 (PNG), OD-17 (trabajo redundante) y
  OD-14r (notificaciones del medidor). El resto queda diferido.
- **L3.** Humano reporta que OBS no detecta el overlay; se investiga con el
  código de OBS: `check_window_valid` filtra `WS_EX_TOOLWINDOW` (H14), el
  método Automático elige BitBlt (H16) y los títulos vacíos colisionan (H15).
  Se añade OD-20 y las decisiones D-OBS-1..4.
- **L4.** OK final del humano: implementar OD-15 + OD-17 + OD-14r y OD-20
  (D-OBS-1 A + D-OBS-2 A).
- **L5.** Implementación en el working tree (sin commit): `decode_png` consume
  `Vec<u8>`; `import_png` valida 8 bytes y copia con `fs::copy`;
  `drain_audio` aplica el lote con un solo `notify`; setters con early-return;
  `update_state` sin el `notify` final; ticker del medidor solo en `Running`
  con pausa de 500 ms en reposo y umbral 0,25 dB; `win32::prepare_capture`
  (quita `WS_EX_TOOLWINDOW`, `ITaskbarList::DeleteTab`), títulos únicos en
  ambas ventanas y docs de OBS en `README` y `architecture`.
- **L6.** Correcciones de revisión: `is_png` vuelve a `len() > 8`; el rechazo
  del PNG truncado de 8 bytes lo hace `import_png` (firma + longitud) con test
  `la_firma_sola_no_alcanza`; `set_avatar` compara con `relative.as_str()`;
  se salta `fs::copy` si origen y destino resuelven al mismo path;
  `Devices`/`Status` cuentan como cambio haber limpiado `devices_error` o
  `audio_error`; se quita el espacio inicial de la continuación de Notas en
  el `README`. Verificación pendiente del CI de GitHub Actions.
