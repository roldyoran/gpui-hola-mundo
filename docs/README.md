# Documentación

Maqueta visual de chat en Rust (GPUI + gpui-kit), organizada como el
`gentle-shell-gpui`: monorepo por features, tokens en `shared` y logica pura
testeable. Esta carpeta explica como esta construido y como trabajar en el.

## Que leer para que estas haciendo

| Vas a… | Lee |
| --- | --- |
| Entender el codigo y donde va algo nuevo | [`architecture.md`](architecture.md) |
| Cambiar un token, un componente o los iconos | [`design-system.md`](design-system.md) |
| Compilar en CI y descargar el binario | [`development.md`](development.md) |

## Mapa rapido

```text
src/
├── main.rs              # arranque: assets, fuentes, ventana
├── features/chat/       # el chat: mod.rs (API), state.rs (puro), view.rs (Render)
└── shared/              # piezas de varios features: ui (tokens), assets (fondo+iconos)
assets/
├── fonts/               # JetBrains Mono empaquetada (OFL-1.1)
├── icons/               # 6 iconos Tabler SVG (MIT)
└── images/              # slava-jamm.jpg de fondo
```
