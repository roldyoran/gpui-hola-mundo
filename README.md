# gpui-hola-mundo

Maqueta visual de chat con **GPUI + gpui-kit**: panel centrado sobre foto,
burbujas, composer con iconos **Tabler** y texto en **JetBrains Mono**.

Organizado como `gentle-shell-gpui`: un feature por carpeta, tokens en
`shared`, logica pura testeable y `docs/` que lo documenta todo.
Ver [`docs/`](docs/README.md) y [`AGENTS.md`](AGENTS.md).

Este PC no compila (pocos recursos): solo se escribe codigo aqui y se
compila en **GitHub Actions (windows-latest)**.

## Estructura

```text
src/
├── main.rs              # arranque: assets, fuentes, ventana 1000x700
├── features/chat/       # mod.rs (API) + state.rs (puro+tests) + view.rs (Render)
└── shared/              # ui.rs (tokens: card, chip, kbd, header, field) + assets.rs
assets/
├── fonts/               # JetBrains Mono Regular+Bold (OFL-1.1)
├── icons/               # send, paperclip, plus, settings, copy, check (Tabler MIT)
└── images/              # slava-jamm.jpg de fondo
docs/                    # architecture, design-system, development
.github/workflows/build.yml  # fmt + clippy + test + build release + artifact
```

Todo el chat es estatico a proposito: los mensajes salen de
`ChatState::seed()` y ningun control tiene handler.

## Uso (sin compilar en local)

```powershell
git add .
git commit -m "alcance: descripcion"
git push
```

Luego: Actions → run verde → artifact `gpui-hola-mundo-windows-x86_64`
(abajo del todo) o:

```powershell
gh run download <run-id> -n gpui-hola-mundo-windows-x86_64 -D .\dist
.\dist\gpui-hola-mundo.exe
```

Si SmartScreen avisa: Más información → Ejecutar de todas formas.

## Versiones (ultimas verificadas 24-nov-2026)

| Tecnologia | Version usada | Fuente |
| --- | --- | --- |
| Rust estable (CI) | 1.100.0 (12-nov-2026) | blog.rust-lang.org, `dtolnay/rust-toolchain@stable` |
| Rust local / MSRV del crate | 1.99 / tienes 1.99.1 | `rust-version = "1.99"`, `edition = "2024"` |
| gpui-kit | 0.11.0 (23-nov-2026, ultima) | crates.io/api/v1/crates/gpui-kit |
| GPUI interno (via gpui-kit) | gpui-pre =0.3.14 exacto | crates.io/api/v1/crates/gpui-kit/0.11.0/dependencies |
| anyhow | 1 (para `AssetSource`) | crates.io |
| Tabler Icons | set 6 SVG (MIT, © Tabler) | cdn.jsdelivr.net/npm/@tabler/icons |
| actions/checkout | v7.0.1 + SHA `3d3c42e` | api.github.com/.../checkout/git/refs/tags/v7.0.1 |
| actions/upload-artifact | v7.0.2 + SHA `cf430e0` | api.github.com/.../upload-artifact/git/refs/tags/v7.0.2 |
| Swatinem/rust-cache | v2.9.3 + SHA `d1527c8` | api.github.com/.../rust-cache (tag v2.9.3) |
| dtolnay/rust-toolchain | SHA `3900882` (`stable`, 15-nov-2026) | api.github.com/.../rust-toolchain/commits/stable |

## Ahorro capa gratuita

- Un solo `windows-latest` (Windows x2 min, macOS x10).
- `concurrency.cancel-in-progress: true`, `paths-ignore: md`.
- `Swatinem/rust-cache` con `save-if: main`.
- `retention-days: 7` en el artifact.
- Los checks (`fmt`, `clippy`, `test`) corren en el mismo job, sin jobs extra.
