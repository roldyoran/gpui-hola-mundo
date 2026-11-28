# Desarrollo

## Regla de oro: no se compila en local

Este equipo no ejecuta `cargo check`, `cargo clippy` ni `cargo build`. El único
verificador es GitHub Actions. Antes de un push, la superficie de API se
contrasta leyendo:

- `~/.cargo/registry/src/index.crates.io-*/gpui-pre-0.3.14/`
- `~/.cargo/registry/src/index.crates.io-*/gpui-kit-0.11.0/`
- docs.rs para `cpal 0.18.2`, `rfd 0.18.0` y `windows 0.62.x`

El ciclo de corrección es: push → Actions → leer el error → corregir → push.

## Qué corre el CI

`.github/workflows/build.yml`, en `windows-latest`:

1. `cargo fmt --check` (siempre).
2. `cargo clippy --all-targets -- -D warnings` (si cambió Rust).
3. `cargo test` (si cambió Rust).
4. `cargo build --release` (siempre).
5. Sube `target/release/pngtuber-lite.exe` como
   `pngtuber-lite-windows-x86_64` (7 días).

No hay matrix ni jobs paralelos: la capa gratuita se cuida con caché de Cargo y
`concurrency` con cancelación.

## Cargo.lock

El lock se resuelve en el runner porque el PC local nunca compila. El workflow
**no** usa `--locked`: si el lock del repo va por detrás del `Cargo.toml`,
Cargo lo actualiza en el runner sin abortar. Al añadir una dependencia, el
`Cargo.lock` del repo puede quedar desactualizado; es esperado y no rompe el
build.

## Añadir una dependencia

1. Justifica el motivo en el PR o en `odd/tasks/`.
2. Fija la versión (`"0.18.2"`, no `"*"`).
3. Comprueba features: por defecto, nada de GTK, X11 ni assets innecesarios.
4. Si es solo para Windows, métela en `[target.'cfg(windows)'.dependencies]`.
5. Actualiza el presupuesto de `docs/benchmarks.md` si afecta al binario.

## Ramas y commits

- `main` es lo que se publica; el CI sube el artefacto de cada push.
- Commits en imperativo y con alcance: `feat(audio): ...`, `fix(win32): ...`,
  `docs(odd): ...`.
- No se commitea `target/` ni binarios.

## Depuración sin consola

El binario de release no abre consola. Los errores recuperables (PNG inválido,
micrófono ocupado o desconectado) se muestran en la ventana de ajustes. Para
ver el detalle completo, ejecuta el `.exe` desde PowerShell y observa el
diálogo de ajustes; no hay `println!` en el código de producto.

## Recursos de Windows

El icono y los metadatos del `.exe` se embeben en tiempo de compilación:

- `build.rs` compila `assets/icon.ico` con `winresource 0.1.34` (usa `rc.exe`
  del Windows SDK, ya presente en el runner `windows-latest` con MSVC). Solo
  corre en target Windows; en otras plataformas no hace nada.
- `assets/icon.ico` es el ICO multi-tamaño que se embebe; `assets/icon.svg`
  es la fuente y `assets/icon.png` la referencia.
- `win32::set_window_icon` fija ese mismo icono en cada ventana con
  `WM_SETICON` (`ICON_BIG`/`ICON_SMALL`).

Para regenerar el icono tras editar `assets/icon.svg`:

```text
python tools/make_icon.py
```

…o, con `uv` (requiere Pillow):

```text
uv run --with pillow python tools/make_icon.py
```
