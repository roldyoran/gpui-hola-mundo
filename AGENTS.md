# AGENTS.md

Maqueta visual de chat (GPUI + gpui-kit). Este PC no compila: todo se verifica
en el CI (`fmt --check` siempre; `clippy -D warnings` y `test` solo si cambia
`src/`, `Cargo.toml` o `Cargo.lock`; build release siempre).

## Donde va cada cosa

| Necesitas… | Va en |
| --- | --- |
| Una pantalla o feature nuevo | `src/features/<nombre>/` (`mod.rs` = API, `state.rs` = puro, `view.rs` = Render) |
| Logica pura testeable | `state.rs` de su feature, con tests |
| Una pieza visual de varios features | `src/shared/` (`ui.rs`, nunca con logica de producto) |
| Un icono o imagen | `assets/` + clave en `src/shared/assets.rs` |
| Documentar una frontera | `docs/` |

## Reglas

- **Diseno:** tokens y piezas de `src/shared/ui.rs`; nada de colores, bordes o radios sueltos. Detalles en `docs/design-system.md`.
- **Orden:** `mod.rs` guarda la API publica; los submodulos anaden `impl`. Tipos privados salvo contrato real.
- **Sin abstracciones de un solo uso:** nada sube a `shared/` hasta que lo usan dos features.
- **Tests:** la logica pura vive en `state.rs` con tests; nada de tests que repitan la implementacion.
- **Seguridad:** crates solo de crates.io, actions pineadas por SHA, token read-only.
- **Licencias:** JetBrains Mono (OFL) y Tabler (MIT) van empaquetadas con su atribucion en `docs/`.
