# PNGTuber Lite — plan ODD

- **Resultado:** app PNGTuber ultraligera en Rust + GPUI, con overlay transparente, VAD por RMS y ajustes mínimos.
- **Estado:** `checking` · CI en verde (run `45727850849`) · Cierre pendiente de prueba humana, CPU/GPU/arranque y soak
- **Fecha:** 2026-11-25 · **Proyecto:** `pngtuber-lite` (ex `gpui-hola-mundo`)
- **Convenciones ODD:** las de `gentle-shell-gpui` (`crates/core/src/gentle_state.rs:149-203`): fases `authorizing · exploring · researching · deciding · planning · implementing · checking · closing`, `gentle_odd_phase` para fijarlas explícitamente y `odd/tasks/` como ubicación del plan (un `edit` ahí infiere `planning`).

## Resultado esperado

Una herramienta que se queda abierta todo el día sin notarse: una ventana sin bordes con el avatar PNG encima del escritorio, que abre y cierra la boca con la voz, y una ventana de ajustes que solo existe mientras se configura. El criterio de éxito no es “tiene funciones”, es **cabe en el presupuesto de recursos y no parpadea**.

### Criterios de aceptación (outcome)

- **S1.** Dos PNG (boca cerrada/abierta) visibles en una ventana independiente.
- **S2.** Sin decoraciones, sin entrada en la barra de tareas y con transparencia **por píxel** (se ve el escritorio, no un rectángulo).
- **S3.** *Siempre visible* conmutable en caliente (Win32 `SetWindowPos`), sin reiniciar.
- **S4.** La boca sigue la voz con RMS + histéresis + hold: sin parpadeo por ruido ambiental, sin IA.
- **S5.** Micrófono seleccionable y persistido por `DeviceId` estable.
- **S6.** Sensibilidad, activar/desactivar detección, tamaño y topmost desde ajustes; guardado automático al confirmar cada control.
- **S7.** Cerrar ajustes no detiene el avatar; se reabren con clic derecho; se sale desde ajustes.
- **S8.** Presupuesto: RAM privada ≤ 40 MB · WS ≤ 70 MB · CPU reposo ≤ 1,5 % de un núcleo · hablando ≤ 3 % · GPU reposo ≤ 1 % · arranque < 400 ms · binario ≤ 16 MB.
- **S9.** Un cambio de boca = **una** notificación de ventana; en silencio, ninguna.
- **S10.** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` en verde; CI release en Windows.
- **S11.** 4 h con micrófono activo sin crecer en RAM, hilos ni handles.
- **S12.** Errores recuperables (PNG inválido, mic desconectado/ocupado) visibles en ajustes sin matar la app.

**Fuera de alcance:** tray, atajos globales, perfiles/múltiples avatares, animaciones o física, Live2D, loopback, reconocimiento de voz, IA, plugins, updater, i18n, editor de imágenes.

## Mapa de fases

Cada fila es una puerta: no se entra en la siguiente sin la evidencia de la anterior. Las acciones están elegidas para que la inferencia de fase de Gentle Shell las lea correctamente.

> Recorrido real de esta sesión: `exploring` → `researching` → `deciding` → `planning`. La única puerta que falta es la aprobación del plan, que habilita `implementing`.

| Fase | Entrada | Acción (herramienta/comando) | Salida y evidencia | Prohibido en la fase |
| --- | --- | --- | --- | --- |
| `authorizing` | Este plan | Preguntar y registrar aprobación | Decisiones firmadas abajo | Escribir código |
| `exploring` | Aprobación | `rg`, `ls`, `git status`, leer fuentes | Tabla de contexto verificado con `archivo:línea` | Editar `src/` |
| `researching` | Huecos de API | Consultar docs oficiales (crates.io, docs.rs, repos) | Versiones y firmas verificadas | Implementar contra suposiciones |
| `deciding` | Alternativas | `ask_user_*` solo si hay una bifurcación real | Decisión con motivo | Decidir en silencio algo que cambia el producto |
| `planning` | Decisiones | `todo` + editar este archivo (`odd/tasks/…`) | Slices OD-00…OD-08 con DoD | Tocar `src/` |
| `implementing` | Plan cerrado | `write`/`edit` en `src/` | Código compilable al final de cada slice | Cambiar el alcance sin volver a `planning` |
| `checking` | Slice implementado | `cargo fmt --check; cargo clippy …; cargo test …` en una sola llamada | Salida real de los comandos | Declarar “verde” sin salida |
| `closing` | S1–S12 con evidencia | `gentle_odd_phase(closing)` + informe | Registro de evidencia completo | Cerrar con deuda no declarada |

> Nota de implementación: `cargo fmt --check` no aparece en el patrón `CHECKING` (`gentle_state.rs:205-214`), así que se ejecuta **en la misma llamada** que `clippy`/`test` para que la fase sea `checking` y no quede la etiqueta anterior.

## Decisiones autorizadas (2026-11-25)

| # | Decisión | Motivo |
| --- | --- | --- |
| D1 | Reutilizar `gpui-hola-mundo`, renombrar la carpeta a `pngtuber-lite` y borrar lo no usado | Es un proyecto de test; conserva CI, docs y el patrón `features/` |
| D2 | Aceptar el piso de GPUI en Windows (≤ 1,5 % de un núcleo en reposo) | Medido: 1,20–1,41 %; sigue 3–10× por debajo de un PNGTuber Electron |
| D3 | Controles propios; `gpui-kit` con `default-features = false` | Sin `gpui-component` ni assets del Kit: menos crates, menos binario |
| D4 | Shim Win32 aislado y documentado | GPUI 0.3.14 no tiene API de movimiento ni de topmost en Windows |
| D5 | **Ningún build local**: ni `cargo check`, ni `clippy`, ni `build` | Decisión humana 2026-11-25. GitHub Actions es el único verificador; el código se escribe contra las firmas reales leídas en `~/.cargo/registry` y la documentación oficial |

## Contexto verificado (`exploring` + `researching`)

| Hecho | Evidencia |
| --- | --- |
| `gpui-kit 0.11.0` fija `gpui-pre 0.3.14` | `gpui-kit-0.11.0/Cargo.toml:275-278` |
| `gpui-kit` por defecto arrastra `component` + `assets` + `x11` | `gpui-kit-0.11.0/Cargo.toml:46-50` |
| `WindowKind::PopUp` → `WS_EX_TOOLWINDOW \| WS_EX_TOPMOST`, estilo 0 | `gpui-pre-windows-0.3.14/src/window.rs:494-495` |
| Transparencia real: swapchain `DXGI_ALPHA_MODE_PREMULTIPLIED` y clear `[0,0,0,0]` | `gpui-pre-windows-0.3.14/src/directx_renderer.rs:1362` y `374-381` |
| No hay `always_on_top` en `WindowOptions` | `gpui-pre-0.3.14/src/platform.rs:2691-2776` |
| `start_window_move()` es no-op en Windows; no hay `move_to` | `gpui-pre-0.3.14/src/platform.rs:1461` |
| El HWND es accesible: `impl HasWindowHandle for Window` | `gpui-pre-0.3.14/src/window.rs:8112` |
| PNG en runtime, un decode: `Image::from_bytes` + `use_render_image` | `gpui-pre-0.3.14/src/platform.rs:3539` y `3552` |
| GPUI es *pull*; la animación continua se pide con `request_animation_frame()` | `gpui-pre-0.3.14/src/window.rs:3084-3099` |
| Windows mantiene un hilo VSync que invalida las ventanas visibles en cada vsync | `gpui-pre-windows-0.3.14/src/platform.rs:429-486` |
| `cpal 0.18.2`: `device.id()`, `host.device_by_id(&id)`, `build_input_stream(config, cb, err, timeout)` | docs.rs cpal 0.18.2 + `examples/record_wav.rs` |
| `rfd 0.18.0` sin features por defecto evita GTK/portal/wayland | features del crate |
| “Rust GPU Kit” no existe (`rust-gpu` es tooling de shaders; `wgpu-kit`, placeholder) | crates.io |
| BongoCat: workspace Rust 2024, GPUI solo ajustes y overlay nativo D3D11/Metal | `BongoCat/Cargo.toml`, `docs/technical-design.md` |
| El audio de BongoCat es solo reproducción; no hay VAD que reutilizar | `crates/bongocat-audio/src/lib.rs` |

**Línea base** (binario release de `gpui-hola-mundo`, 1000×700 con JPEG, 4 núcleos): WS 47,3–57,0 MB · privada 36,3 MB · 16 hilos · CPU 1,20 % y 1,41 % de un núcleo en dos muestras de 30 s. Toolchain local: `cargo 1.99.1`, `rustc 1.99.1`, target `x86_64-pc-windows-gnu`; `cargo check` en 53 s.

## Arquitectura objetivo

```text
pngtuber-lite/
├── Cargo.toml
├── .github/workflows/build.yml
├── odd/tasks/2026-11-25-pngtuber-lite-mvp.md   # este plan
├── docs/{architecture,design-system,benchmarks}.md
└── src/
    ├── main.rs          arranque y composición, sin lógica
    ├── lib.rs
    ├── app.rs           AppState (Global) + bomba audio → Window::refresh
    ├── window.rs        WindowOptions y persistencia de bounds
    ├── win32.rs         #[cfg(windows)] drag + topmost
    ├── shared/ui.rs     tokens y controles sin lógica de producto
    └── features/
        ├── avatar/{mod.rs,view.rs}      carga y overlay
        ├── audio/{mod.rs,devices.rs,stream.rs,vad.rs}
        └── settings/{mod.rs,view.rs}    esquema y UI
```

| Módulo | Responsabilidad | No hace |
| --- | --- | --- |
| `main` | `application().run`, `init(cx)`, cargar ajustes, abrir overlay, arrancar audio | UI ni reglas |
| `app` | `AppState` global; bomba `async-channel` → `Window::refresh` | decidir VAD ni dibujar |
| `window` | opciones de overlay/ajustes; leer bounds vivos y guardarlos | crear controles |
| `win32` | `ReleaseCapture`+`WM_NCLBUTTONDOWN`; `SetWindowPos(HWND_TOPMOST)` | reglas de negocio |
| `avatar` | validar PNG, copiar a datos, decodificar una vez, `Arc<RenderImage>` | conocer el audio |
| `audio` | enumerar, dueño del stream, RMS+VAD, eventos de boca | tocar GPUI |
| `settings` | `schema_version: 1`, defaults, escritura atómica | dibujar |
| `shared/ui` | tokens y piezas | conocer features |

**Hilos.** Principal: GPUI (overlay + ajustes). Dueño de audio: dormido en `recv`, abre/cierra el stream. Captura WASAPI (de cpal): RMS + VAD por buffer de ~10 ms, sin asignar ni consultar el reloj. En silencio no se notifica nada; hablando, ~1 frame por transición.

**VAD.** `dBFS(RMS)` contra `umbral_apertura = f(sensibilidad 0..100)` con `umbral_cierre = apertura − 6 dB`, `attack = 40 ms`, `release = 250 ms`. `vad.rs` es puro y se prueba con `cargo test`: ruido constante, voz con pausas, clic aislado, cambio de umbral en caliente, contador de muestras.

**Ajustes.** `%APPDATA%\pngtuber-lite\settings.json` (fallback XDG/HOME):

```json
{
  "schema_version": 1,
  "audio": { "enabled": true, "device_id": "…", "sensitivity": 50 },
  "avatar": { "closed": "avatars/closed.png", "open": "avatars/open.png" },
  "window": { "x": 1200.0, "y": 640.0, "height": 320.0, "always_on_top": true }
}
```

`schema_version ≠ 1` o JSON corrupto → defaults. Escritura en `.tmp` + `fs::rename`. Los PNG se copian a `avatars/`.

**Dependencias.** `gpui-kit 0.11.0` (`default-features = false`), `cpal 0.18.2`, `serde`+`serde_json` (`1` / `1.0.154`), `rfd 0.18.0` (`default-features = false`, `common-controls-v6`), `async-channel 2.5`, `windows 0.62` (`Win32_Foundation` + `Win32_UI_WindowsAndMessaging`), `raw-window-handle 0.6`, `anyhow 1`. Descartadas: `wgpu`, `image`, `dirs`, `atomic-write-file`, `tray-icon`, `log`.

## Tareas (`todo`)

Estados del `todo`: `pending` → `in_progress` → `done`. Cada slice termina compilando y arrancando.

| ID | Slice | Fase que lo cierra | DoD |
| --- | --- | --- | --- |
| **OD-00** | Higiene: renombrar, borrar chat/foto/iconos/`dist`, `Cargo.toml`, README/AGENTS/docs, CI | `implementing` + `checking` | CI en verde (fmt, clippy, test, build); sin restos del chat ni assets sin uso |
| **OD-01** | Núcleo puro: `Settings` (defaults, load/save atómico) y `Vad` con tests | `implementing` → `checking` | Tests RED→GREEN; round-trip; corrupción → defaults |
| **OD-02** | Overlay: PopUp + transparente + `win32.rs` (drag/topmost) + persistencia de bounds | `implementing` → `checking` | Sin borde, sin taskbar, transparente, arrastrable, topmost conmutable |
| **OD-03** | Avatar: validar PNG, copiar, decodificar una vez, alternar texturas | `implementing` → `checking` | Cambio solo con el estado; un decode por archivo |
| **OD-04** | Audio: enumerar, hilo dueño, stream, RMS+VAD, eventos | `implementing` → `checking` | Boca con latencia percibida < 150 ms; callback sin allocs |
| **OD-05** | Ajustes: pickers, lista de micrófonos, steppers, toggle, chip de voz, *Salir* | `implementing` → `checking` | Flujo completo sin consola; guardado automático; reapertura con clic derecho |
| **OD-06** | Integración: primera ejecución sin avatar abre ajustes; cerrar ajustes no cierra la app; reinicio de audio al cambiar de dispositivo | `implementing` → `checking` | S1–S7 a mano |
| **OD-07** | Medición y optimización justificada (downscale, perfil release, notificaciones) | `checking` → `deciding` si hay bifurcación | Antes/después documentado; nada optimizado sin número |
| **OD-08** | Pruebas y release: fmt/clippy/tests, soak 4 h, CI, binario | `checking` → `closing` | S8–S12 con evidencia |
| **OD-09** | **Siguiente fase (no MVP):** adoptar el sistema de diseño de `gentle-shell-gpui` y sus iconos; medir el coste en binario | `planning` | Tokens alineados, iconos servidos sin empaquetar el Kit completo, presupuesto re-medido |

## Verificación

> **D5 (2026-11-25):** no se ejecuta nada que compile en local — ni `cargo check`, ni `clippy`, ni `build`. El CI de GitHub Actions es el único verificador: `cargo fmt --check` descubre formato, y `clippy`/`test`/`build --release` descubren errores. Cada push debe salir con la superficie de API ya verificada contra `~/.cargo/registry` y docs oficiales, porque el ciclo de corrección es el runner.

```powershell
# Estos comandos los ejecuta GitHub Actions, nunca este PC.
cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test
cargo build --release

# RAM + hilos
$p = Start-Process .\target\release\pngtuber-lite.exe -PassThru
Start-Sleep 5; $p.Refresh()
"WS $([math]::Round($p.WorkingSet64/1MB,1)) MB | Priv $([math]::Round($p.PrivateMemorySize64/1MB,1)) MB | Hilos $($p.Threads.Count)"

# CPU reposo 60 s (% de un núcleo)
$p.Refresh(); $c1 = $p.TotalProcessorTime.TotalMilliseconds
Start-Sleep 60; $p.Refresh(); $c2 = $p.TotalProcessorTime.TotalMilliseconds
"$([math]::Round(($c2-$c1)/60000*100,3))"
```

Todo número va a `docs/benchmarks.md` con fecha, máquina, resolución y comando. GPU se lee en la columna GPU del Administrador de tareas (no hay contador estable por proceso sin ETW).

## Riesgos

| Riesgo | Mitigación |
| --- | --- |
| Piso VSync de GPUI (~1,2 % de un núcleo) | Aceptado en S8. Si se exigiera 0 %, el overlay migra a `WS_EX_LAYERED` + `UpdateLayeredWindow` sin tocar la lógica pura |
| `SendMessageW(WM_NCLBUTTONDOWN)` dentro del handler de GPUI deja estado de ratón inconsistente | Probar en OD-02; si ocurre, `PostMessageW` o arrastre manual con `SetWindowPos` |
| Cambio de micrófono en caliente | Recrear el stream en el hilo dueño, nunca en el callback |
| Toolchain local `windows-gnu` | El CI compila con MSVC; el PC local no compila (D5) |
| GPUI cambia API | Superficie aislada en `window.rs`, `win32.rs` y `avatar` |
| PNG enorme (4000×4000) | OD-07 mide y decide downscale con `image` solo si el número lo justifica |

## Registro de evidencia

Formato: una línea por slice, con el comando real y su salida. Sin salida no hay evidencia.

### `exploring` / `researching` (completado)

- BongoCat leído: `Cargo.toml`, `AGENTS.md`, `docs/technical-design.md`, `crates/bongocat-audio`, `crates/bongocat-overlay/{idle,timing}.rs`.
- GPUI verificado en fuente: `WindowKind::PopUp`, transparencia premultiplicada, ausencia de topmost/move, `Image::from_bytes`/`use_render_image`, redibujado *pull*, hilo VSync.
- Versiones comprobadas en crates.io: `cpal 0.18.2`, `rfd 0.18.0`, `windows 0.62.2`, `async-channel 2.5`.
- Alternativas descartadas con motivo: “Rust GPU Kit”, `wgpu`, `egui`, `Slint`, `Iced`.
- Línea base de recursos y toolchain, arriba.

### `implementing` / `checking` (implementado; CI pendiente)

- **OD-00.** `gpui-hola-mundo` renombrado a `pngtuber-lite`; borrados `src/features/chat`, `assets/` (fuentes, iconos, foto), `dist/` y el `AssetSource`. `Cargo.toml` con las dependencias definitivas; README, AGENTS, `docs/` y workflow reescritos. `cargo fmt --all` local terminó con código 0 (formatea, no compila). **Evidencia real pendiente del run del CI.**
- **OD-01.** `features/settings`: esquema v1, valores por defecto, saneado, `Paths` (`%APPDATA%/pngtuber-lite`, fallback XDG/HOME) y guardado `settings.json.tmp` + `rename`. `features/audio/vad.rs`: 9 tests (ataque, release, histéresis, clic aislado, pausas cortas, sensibilidad en caliente, desbordamiento, NaN, umbrales). `avatar/mod.rs`: 3 tests de firma/nombres PNG. Total: 12 tests.
- **OD-02.** `window::avatar_options` con `WindowKind::PopUp`, `WindowBackgroundAppearance::Transparent`, sin redimensionar ni minimizar; `win32.rs` con `ReleaseCapture` + `WM_NCLBUTTONDOWN` y `SetWindowPos(HWND_TOPMOST/NOTOPMOST)`, todo el `unsafe` aislado y comentado; bounds persistidos tras el arrastre.
- **OD-03.** `avatar::import_png` valida firma, copia a `avatars/closed.png` o `open.png`; `decode_png` produce `Arc<Image>` una vez y `img(Arc<Image>)` reutiliza la caché de GPUI. La vista recarga solo cuando sube `avatar_generation`.
- **OD-04.** `features/audio`: hilo dueño con `recv_blocking`, comandos `Apply`/`RefreshDevices`, enumeración de dispositivos y stream cpal **dentro de ese hilo**; callback sin allocs, sin locks y sin reloj; RMS → dBFS → `Vad` → `try_send` solo al cambiar. Sensibilidad por `AtomicU8` (sin reconstruir el stream).
- **OD-05.** Ajustes completos con piezas propias: PNG, lista de micrófonos con predeterminado, sensibilidad ±5, tamaño ±40 px, topmost, chip SILENCIO/HABLANDO, estado del micrófono, actualizar dispositivos y salir. Guardado automático; errores visibles.
- **OD-06.** Primera ejecución sin PNG abre ajustes; el overlay vive aunque se cierre ajustes; clic derecho lo reabre; cambiar de dispositivo o de sensibilidad se aplica en caliente; *Salir* usa `cx.quit()`.
- **OD-07.** Medición parcial con el artefacto del CI (`pngtuber-lite.exe`, 11.433.984 B = 10,9 MiB): overlay 320×320 + ajustes abiertos, sin PNG → **WS 51,2 MB**, **privada 30,7 MB**, **17 hilos**. CPU, GPU y arranque quedan para la prueba humana: `WaitForInputIdle` no es fiable con esta app y no se inventa el número. El protocolo completo está en `docs/benchmarks.md`.
- **OD-08.** CI en verde en el run [`45727850849`](https://github.com/roldyoran/gpui-hola-mundo/actions/runs/45727850849): `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (12 tests) y `cargo build --release`; artefacto `pngtuber-lite-windows-x86_64` subido. `Cargo.lock` regenerado con Cargo 1.99.1 (693 paquetes, sin `gpui-component` ni assets del Kit). Faltan la prueba funcional del humano y el soak de 4 h.

## Cierre

- [ ] S1–S12 con evidencia enlazada.
- [x] `docs/benchmarks.md` con la medición parcial del CI y fecha.
- [x] `docs/architecture.md` y `docs/design-system.md` al día.
- [x] `README.md` con requisitos, primer arranque (sin avatar → ajustes) y clic derecho.
- [x] Sin `TODO`, `dbg!`, `unwrap()` en rutas de audio o ventana, ni `unsafe` fuera de `win32.rs`.
- [ ] Soak de 4 h registrado (RAM, hilos, handles al inicio y al final).
- [ ] Riesgos residuales declarados (piso de GPUI incluido).

## Log

- **L1.** Pedido: PNGTuber en Rust, mínimo consumo, sin IA; análisis y aprobación antes de codificar.
- **L2.** Investigación de BongoCat, GPUI y alternativas; medición del piso de GPUI.
- **L3.** Aprobadas D1–D4 (reutilizar/renombrar, aceptar piso, controles propios, shim Win32).
- **L4.** Plan reescrito con el modelo ODD de `gentle-shell-gpui` (fases y `odd/tasks/`), sin mezclar formatos de otros repos.
- **L5.** Aprobado para implementar. D5: sin builds locales; CI como único verificador. Se auditaron las firmas reales de `gpui-pre 0.3.14`, `cpal 0.18.2`, `rfd 0.18.0` y `windows 0.62.2` antes de escribir código.
- **L6.** Pedido humano (2026-11-25): en la siguiente fase, usar los iconos de `gentle-shell-gpui` y alinear el sistema de diseño con el suyo. Queda como **OD-09**, fuera del MVP, para no mezclar alcance antes de cerrar la verificación del CI.
- **L7.** CI en verde (run `45727850849`). El humano descargará el `.exe` y hará la prueba funcional; por eso se detiene la medición automática y no se registran CPU/GPU/arranque sin número real.
