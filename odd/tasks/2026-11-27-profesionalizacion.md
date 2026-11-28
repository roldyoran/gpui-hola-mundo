# Profesionalización: icono, metadatos, borde y nombre

- **Estado:** `closing` · CI verde (run `46104090583`) y prueba humana confirmada
- **Fecha:** 2026-11-27
- **Origen:** prueba humana de OBS (WGC funciona; aparece un borde grisáceo) y
  pedido de dejar el proyecto como una aplicación de escritorio terminada.
- **Fase anterior:** [`2026-11-27-optimizacion-rendimiento.md`](2026-11-27-optimizacion-rendimiento.md)
- **Restricción vigente:** **D5** — este PC no compila; CI es el verificador.

## Resultado esperado

1. El overlay capturado por OBS no muestra ningún borde ni esquina redondeada
   de Windows; solo el PNG y su transparencia.
2. El `.exe` tiene icono propio, metadatos de versión y las dos ventanas usan
   ese icono. La UI muestra la versión.
3. El repositorio se llama `pngtuber-lite` en GitHub y en el remoto local, sin
   romper enlaces ni el CI.

## Diagnóstico

| # | Hecho | Evidencia |
| --- | --- | --- |
| P1 | Windows 11 dibuja un borde DWM de 1 px y redondea las esquinas de las ventanas top-level, incluso con `WS_POPUP` y sin `WS_EX_TOOLWINDOW`. Eso es el reborde grisáceo que WGC entrega a OBS. | `DwmSetWindowAttribute`, `DWMWA_BORDER_COLOR = DWMWA_COLOR_NONE`, `DWMWA_WINDOW_CORNER_PREFERENCE = DWMWCP_DONOTROUND` (windows 0.62.2, `Win32_Graphics_Dwm`). En Windows 10 la llamada devuelve error y se ignora. |
| P2 | El icono del `.exe` (Explorer, Propiedades, Alt+Tab) es un recurso PE, no algo que se pueda cambiar solo en runtime. | `winresource 0.1.34` compila `assets/icon.ico` con `rc.exe` del Windows SDK (ya presente en `windows-latest` MSVC). |
| P3 | El icono de cada ventana (barra de tareas y Alt+Tab) se fija con `WM_SETICON` desde el recurso embebido. | `LoadImageW` con `MAKEINTRESOURCE(1)` + `SendMessageW(WM_SETICON, ICON_BIG/ICON_SMALL)`. |
| P4 | El repo remoto seguía llamándose `gpui-hola-mundo`; `gh repo rename` lo cambió a `pngtuber-lite` y actualizó el remoto. GitHub redirige las URLs viejas, pero los enlaces del repo deben quedar con el nombre nuevo. | `git remote -v` ya apunta a `roldyoran/pngtuber-lite`. |

## Diseño del icono

- SVG fuente en `assets/icon.svg`, ICO multi-tamaño en `assets/icon.ico`
  (16, 24, 32, 48, 64, 128 y 256) y PNG de referencia en `assets/icon.png`.
- Fondo `#09090b` con radio 56, borde sutil `#27272a`, burbuja blanca
  `#fafafa`, tres barras de onda oscuras y punto verde `#34d399`.
- Minimalista, legible a 16 px y coherente con los tokens de `shared/ui.rs`.

## Slices

### OD-22 · Sin borde ni esquinas en el overlay

- En `win32::prepare_capture` (se llama solo al crear el overlay) añadir:
  - `DWMWA_BORDER_COLOR` con `DWMWA_COLOR_NONE`;
  - `DWMWA_WINDOW_CORNER_PREFERENCE` con `DWMWCP_DONOTROUND`;
  - ignorar el error en Windows 10.
- Feature nueva: `Win32_Graphics_Dwm`.
- **DoD:** humano confirma que OBS ya no muestra el reborde; si quedara una
  línea de 1 px, probar **Client Area** en las propiedades de la fuente OBS.

### OD-23 · Icono, metadatos y versión visible

- `build.rs` con `winresource 0.1.34` (build-dependency, solo Windows):
  `set_icon("assets/icon.ico")`, `ProductName`, `FileDescription`,
  `OriginalFilename`, `LegalCopyright`. Versión heredada de `Cargo.toml`.
- `win32::set_window_icon(window)` y llamada al crear ajustes y overlay.
  Features: `Win32_System_LibraryLoader` (y las ya presentes).
- `Cargo.toml`: `build = "build.rs"`, `authors`, `readme`, `repository`,
  `homepage`, `keywords`, `categories`; comentario con el motivo de
  `winresource`.
- `settings/view.rs`: versión en la cabecera
  (`concat!(..., env!("CARGO_PKG_VERSION"))`).
- `README.md`: icono al inicio, enlace al repo, sección de licencia y nota de
  OBS (WGC + Client Area).
- `LICENSE` MIT (titular `roldyoran`).
- `docs/architecture.md` y `docs/development.md`: `build.rs`, recurso e icono.

### OD-24 · Nombre del repositorio y enlaces

- `gh repo rename pngtuber-lite` **ya ejecutado**; el remoto ya es
  `git@github.com:roldyoran/pngtuber-lite.git`.
- Hecho histórico: se reemplazaron las URLs `gpui-hola-mundo/actions` por
  `pngtuber-lite/actions` en `odd/tasks/*.md`.
- Mantener las menciones históricas a `gpui-hola-mundo` (son parte del
  historial del proyecto), pero no las URLs.
- `Cargo.toml` y README apuntan al nombre nuevo.

## Evidencia

- Icono generado y verificado visualmente a 256 px; el ICO se extrajo del
  `.exe` final y se comprobó que Windows lo lee.
- CI en verde (run [`46104090583`](https://github.com/roldyoran/pngtuber-lite/actions/runs/46104090583)):
  fmt, clippy, 13 tests, release y artefacto `pngtuber-lite-windows-x86_64`
  (4.733.703 B). `.exe`: **12.062.208 B (11,5 MiB)**, dentro del límite.
- Metadatos PE verificados con `FileVersionInfo`:
  `ProductName=PNGTuber Lite`, `FileDescription=PNGTuber Lite - overlay
  PNGTuber ultraligero`, `FileVersion=ProductVersion=0.1.0`,
  `OriginalFilename=pngtuber-lite.exe`, `LegalCopyright=MIT - roldyoran`.
- El primer push falló en clippy (`manual_dangling_ptr` en el cast del ID de
  recurso); se corrigió con `std::ptr::without_provenance::<u16>(1)` y el
  segundo run salió en verde.
- Prueba humana del artefacto: funcionamiento correcto, sin incidencias de
  borde, icono ni versión.

## Riesgos

| Riesgo | Mitigación |
| --- | --- |
| `rc.exe` no se encuentra en el runner | `windows-latest` MSVC incluye el Windows SDK; si falla, se cae a `embed-resource` o se quita el build script |
| `DwmSetWindowAttribute` falla en Windows 10 | Se ignoran los errores; en Windows 11 aplica |
| El borde persiste por el método de OBS | Probar **Client Area** en la fuente; si sigue, decidir recorte/margen |
| La redirección del repo viejo se pierde | Los enlaces del repo se actualizan al nombre nuevo |
| El icono aumenta el binario | Verificar el tamaño contra el límite de 16 MB |

## Verificación

- `cargo fmt --check` siempre; `clippy -D warnings` y `test` por cambio de
  `src/`; `build --release` siempre (GitHub Actions).
- Humano: abrir OBS con WGC y confirmar que no hay borde; revisar icono en
  Explorer, barra de tareas y Alt+Tab; ver la versión en la cabecera.
- Registrar en `docs/benchmarks.md` el tamaño del nuevo binario.

## Log

- **L1.** Humano confirma que WGC captura el overlay y reporta un borde
  grisáceo. Se identifica el borde DWM de Windows 11 (P1).
- **L2.** Se genera el icono SVG/ICO/PNG y se elige `winresource 0.1.34`
  (P2/P3). Repo renombrado a `pngtuber-lite` (P4).
- **L3.** Pendiente: implementación, CI y prueba humana.
- **L4.** Implementación (sin commit): OD-22 (DWM en `prepare_capture` con
  `SAFETY`, feature `Win32_Graphics_Dwm`); OD-23 (`build.rs`, `winresource`
  0.1.34, `set_window_icon` con `WM_SETICON`, versión en cabecera,
  metadatos en `Cargo.toml`); OD-24 (iconos/badge/licencia en README,
  `LICENSE`, `architecture.md` y `development.md` al día, URLs de
  `odd/tasks` al repo nuevo). Firmas contrastadas en `~/.cargo/registry`
  (windows 0.62.2, winresource 0.1.34). Pendiente: CI y prueba humana.
- **L5.** CI verde en el run
  [`46104090583`](https://github.com/roldyoran/pngtuber-lite/actions/runs/46104090583)
  (commit `d4e3b6f`): fmt, clippy, 13 tests, release y artefacto. El `.exe`
  mide 12.062.208 B (11,5 MiB); `FileVersionInfo` y el icono embebido se
  verificaron sobre el artefacto descargado.
- **L6.** El humano probó el artefacto y confirmó que todo funciona
  correctamente; no reportó incidencias de borde, icono ni versión. Fase
  cerrada.
