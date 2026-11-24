# Sistema de diseño

Tokens y piezas copiados de `gentle-shell-gpui` (`crates/app/src/shared/ui.rs`)
y adaptados a un solo tema oscuro. Regla: ningun feature inventa colores,
bordes o radios; todo sale de `src/shared/ui.rs`.

## Tokens (`shared::ui`)

| Token | Uso |
| --- | --- |
| `panel` | Fondo del panel de chat |
| `surface` | Tarjetas, burbuja del asistente |
| `raised` | Burbuja del usuario |
| `well` | Composer, huecos oscuros |
| `border`, `border_strong`, `input_border` | Bordes |
| `fg`, `muted`, `faint` | Texto principal, secundario, tenue |
| `accent` | Rosa de marca: enviar, avatar |
| `success`, `info`, `warning`, `danger` | Chips de estado |
| `mono()` | JetBrains Mono empaquetada |

## Componentes

| Pieza | Origen | Uso aqui |
| --- | --- | --- |
| `card()` | `ui.rs:card` | Base de tarjetas |
| `chip(text, kind)` | `ui.rs:chip` | `maqueta visual`, `sin backend` (variantes `ChipKind`, sin colores sueltos) |
| `kbd(text)` | `ui.rs:kbd` | Atajos (reservado para el composer interactivo) |
| `section_label(text)` | `ui.rs:section_label` | Titulos de seccion |
| `screen_header()` | `ui.rs:screen_header` | Cabecera del panel (marca, titulo, acciones) |
| `field()` | `ui.rs:field` | Marco del composer |
| `shortcut(key)` | `ui.rs:shortcut` | `Ctrl+K` / `⌘K` segun SO |

## Iconos Tabler

6 SVG de [Tabler Icons](https://tabler.io) (MIT, © Tabler): `send`,
`paperclip`, `plus`, `settings`, `copy`, `check`. Van empaquetados en
`assets/icons/` y se sirven por clave (`icons/send.svg`) desde
`shared::assets::AppAssets`. Se dibujan con `svg().path(...).text_color(...)`
(ver guia `gpui-kit.com/docs/image`): el color **debe** ir en el elemento.

Para agregar uno: descargalo a `assets/icons/`, registralo en el `match` de
`assets.rs` y usalo con la clave `icons/<nombre>.svg`.

## Fuentes

JetBrains Mono Regular + Bold empaquetadas (`assets/fonts/`, OFL-1.1) y
registradas con `TextSystem::add_fonts` antes del primer frame
(patron de `gpui-kit.com/docs/fonts`). Todo el chat la usa via `ui::mono()`.

## Fondo

`assets/images/slava-jamm.jpg` (119 KB) a pantalla completa con
`ObjectFit::Cover` + velo negro al 65 % (`rgba(0x000000a6)`): la foto es muy
clara arriba y el texto blanco necesita contraste.
