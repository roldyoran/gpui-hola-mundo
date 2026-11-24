# Desarrollo

Este PC no compila (pocos recursos): solo se escribe codigo aqui y todo se
verifica en GitHub Actions.

## Flujo

```powershell
cd gpui-hola-mundo
git add .
git commit -m "alcance: descripcion"
git push
```

Cada push a `main` (o PR) lanza `Build` en `windows-latest`: `fmt --check`,
`clippy -D warnings`, `test` y `cargo build --release`, y sube el `.exe` como
artifact `gpui-hola-mundo-windows-x86_64` (retencion 7 dias).

## Descargar el binario

```powershell
gh run download <run-id> -n gpui-hola-mundo-windows-x86_64 -D .\dist
.\dist\gpui-hola-mundo.exe
```

O desde la web: Actions → run verde → Artifacts (abajo del todo).

## Agregar un feature

1. Crea `src/features/<nombre>/` con `mod.rs` (API publica), `state.rs`
   (logica pura + tests) y `view.rs` (Render).
2. Registralo en `src/features/mod.rs`.
3. Si una pieza la usan dos features, subela a `src/shared/`.
4. Documenta el cambio en `docs/` si mueve una frontera.

## Agregar un icono Tabler

1. Descarga el SVG a `assets/icons/`.
2. Registra la clave en `src/shared/assets.rs`.
3. Usalo: `svg().path("icons/<nombre>.svg").text_color(...)`.

## Licencias de assets

- JetBrains Mono: OFL-1.1 (`assets/fonts/OFL.txt`).
- Tabler Icons: MIT (© Tabler, https://tabler.io).
