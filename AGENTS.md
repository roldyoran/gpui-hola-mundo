# AGENTS.md

Overlay PNGTuber en Rust + GPUI. **Este PC no compila.** La verificación es
siempre GitHub Actions: `fmt --check` en cada push; `clippy -D warnings` y
`test` solo si cambia `src/`, `Cargo.toml` o `Cargo.lock`; `build --release`
siempre. No ejecutes `cargo check`, `clippy` ni `build` en local.

## Dónde va cada cosa

| Necesitas… | Va en |
| --- | --- |
| Una pantalla o feature nuevo | `src/features/<nombre>/` (`mod.rs` = API, `view.rs` = Render) |
| Lógica pura testeable | `features/<nombre>/mod.rs` o un submódulo sin GPUI, con tests |
| Una pieza visual de varios features | `src/shared/ui.rs` (sin lógica de producto) |
| Una llamada nativa de Windows | `src/win32.rs`, con `#[cfg(windows)]` y comentario `SAFETY` |
| Documentar una frontera | `docs/` |
| El plan y la evidencia | `odd/tasks/` |

## Reglas

- **Build:** prohibido compilar en local. El ciclo de corrección es el runner;
  antes de cada push, contrasta las firmas con `~/.cargo/registry` o docs.rs.
- **Diseño:** tokens y piezas de `src/shared/ui.rs`; nada de colores, bordes,
  radios o alturas sueltas. Detalles en `docs/design-system.md`.
- **Sin componentes de terceros:** `gpui-kit` va con `default-features = false`.
  No añadas `gpui-component`, `gpui-kit-assets`, `tray-icon` ni frameworks UI.
- **`unsafe`:** solo en `win32.rs`, aislado por función y con justificación.
- **Audio:** el callback de cpal no asigna, no bloquea y no consulta el reloj.
  El VAD es puro (`vad.rs`) y se prueba con `cargo test` en el CI.
- **Hilos:** la UI nunca construye ni destruye el stream; eso ocurre en el hilo
  dueño del audio, que duerme en `recv_blocking`.
- **Persistencia:** esquema v1, escritura temporal + `rename`, sin migraciones.
- **Dependencias:** solo crates.io, versión fijada y motivo en `Cargo.toml` o
  en el plan ODD. Nada de dependencias "por comodidad".
- **Tests:** la lógica pura vive junto a su módulo; no repitas la implementación
  en el test.
