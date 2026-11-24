# gpui-hola-mundo

Hola mundo mínimo con **GPUI + gpui-kit**: ventana de 800x600 centrada en pantalla con texto centrado.

Proyecto de testeo. Este PC no compila (pocos recursos): solo se escribe código aquí y se compila en **GitHub Actions (windows-latest)**.

## Estructura

```text
gpui-hola-mundo/
├── Cargo.toml              # unica dependencia: gpui-kit 0.11.0 (ultima estable)
├── src/main.rs             # vista HolaMundo centrada con flex + JetBrains Mono
├── assets/fonts/           # JetBrainsMono Regular+Bold (OFL-1.1) empaquetadas
├── assets/images/          # slava-jamm.jpg (fondo, 119 KB empaquetado)
├── rustfmt.toml            # edition 2024
└── .github/workflows/build.yml  # build release + upload artifact
```

Todo el texto usa **JetBrains Mono**: los TTF van empaquetados en el binario
con `include_bytes!` y se registran con `TextSystem::add_fonts` antes del
primer frame (patron de `gpui-kit.com/docs/fonts`). No depende de que la
fuente este instalada en el equipo. Licencia en `assets/fonts/OFL.txt`.

## Uso (sin compilar en local)

```powershell
cd gpui-hola-mundo
git init
git add .
git commit -m "test: hola mundo gpui centrado"
# crea el repo en GitHub y sube main
gh repo create gpui-hola-mundo --public --source=. --push
```

Luego:

1. Ve a la pestaña **Actions** del repo → job `Build` → artifact `gpui-hola-mundo-windows-x86_64`.
2. Descarga el zip, extrae `gpui-hola-mundo.exe` y ejecútalo.
3. Si SmartScreen avisa (publisher desconocido): **Más información → Ejecutar de todas formas**.

## Versiones (ultimas verificadas 24-nov-2026)

| Tecnologia | Version usada | Fuente |
| --- | --- | --- |
| Rust estable (CI) | 1.100.0 (12-nov-2026) | blog.rust-lang.org, `dtolnay/rust-toolchain@stable` |
| Rust local / MSRV del crate | 1.99 / tienes 1.99.1 | `rust-version = "1.99"`, `edition = "2024"` |
| gpui-kit | 0.11.0 (23-nov-2026, ultima) | crates.io/api/v1/crates/gpui-kit |
| anyhow | 1 (para `AssetSource`) | crates.io (dependencia minima) |
| GPUI interno (via gpui-kit) | gpui-pre =0.3.14 exacto | crates.io/api/v1/crates/gpui-kit/0.11.0/dependencies |
| actions/checkout | v7.0.1 + SHA `3d3c42e` | api.github.com/repos/actions/checkout/git/refs/tags/v7.0.1 |
| actions/upload-artifact | v7.0.2 + SHA `cf430e0` | api.github.com/repos/actions/upload-artifact/git/refs/tags/v7.0.2 |
| Swatinem/rust-cache | v2.9.3 + SHA `d1527c8` | api.github.com/repos/Swatinem/rust-cache (tag v2.9.3 -> ese commit) |
| dtolnay/rust-toolchain | SHA `3900882` (`stable`, 15-nov-2026) | api.github.com/repos/dtolnay/rust-toolchain/commits/stable |

## Notas de investigación

- Rust: `doc.rust-lang.org/book`, `edition 2024`, `rust-version = "1.99"` (tu toolchain 1.99.1 vale; CI usa estable 1.100.0).
- GPUI: `gpui.rs`, `zed-industries/zed/crates/gpui` (ejemplo `hello_world.rs` con `Bounds::centered` + `open_window`), `docs.rs/gpui`. Modelo `Application::run` → `open_window` → vista `Render` → `div().flex().items_center().justify_center()`.
- gpui-kit: `gpui-kit.com/docs/getting-started` (dependencia única `gpui-kit = "0.11"`, `application().with_assets().run()` + `init(cx)` + `open_window` + `Root`, `Label` accesible). En macOS requiere feature `font-kit` en `gpui-pre-platform`; en Windows no requiere nada.
- crates.io: `gpui-kit` 0.11.0 ultima (23-nov-2026); `gentle-shell-gpui` fija `gpui-kit 0.9.0` + `gpui-pre =0.3.11` por estabilidad. Este testeo usa 0.11.0 + `gpui-pre =0.3.14` que trae el propio Kit.
- Referencia `Gentleman-Programming/gentle-shell-gpui`: workspace 3 crates, `AGENTS.md` (fmt+clippy+test antes de commit), CI con `Swatinem/rust-cache` + `dtolnay/rust-toolchain` + actions pineadas por SHA, `release.yml` con matrix 4 targets + `dist` profile + SHA256SUMS. Este testeo simplifica a 1 job Windows para ahorrar minutos (Windows x2) y storage.
- `lotus --help`: `scrape` (rápido), `js` (SPAs con Chromium), `search` (DuckDuckGo sin API key), `map`/`crawl`, `--formats markdown,links,metadata,headings`, `--pick`, `--limit-chars`, `--quiet`.

## Ahorro capa gratuita

- Un solo `windows-latest` (sin matrix macOS/Linux que multiplican minutos).
- `concurrency.cancel-in-progress: true`, `paths-ignore: md`.
- `Swatinem/rust-cache` con `save-if: main`.
- `retention-days: 7` en el artifact.
