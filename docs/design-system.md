# Sistema de diseño

Una sola ventana de ajustes, un solo tema oscuro y controles propios. Las
reglas existen para que un agente no invente estilos: todo sale de
`src/shared/ui.rs`.

## Tokens

| Token | Valor | Uso |
| --- | --- | --- |
| `panel` | `#121216` | fondo de la ventana de ajustes |
| `surface` | `#1e1e28` | tarjetas |
| `well` | `#0e0e13` | listas, medidores, fondos de botón |
| `button_bg` | `#2a2a38` | botón neutro |
| `danger_bg` | `#3a1f26` | botón de salida |
| `border` | blanco 12 % | bordes |
| `fg` | `#f2f0eb` | texto principal |
| `muted` | `#c9c4bb` | texto secundario |
| `faint` | `#8a857c` | pistas y etiquetas |
| `accent` | `#e85d9e` | selección, medidor, foco |
| `success` | `#7ee2a8` | micrófono escuchando |
| `warning` | `#f5c97a` | conectando |
| `danger` | `#f58a8a` | errores |
| `selected` | acento 15 % | fila activa |

## Piezas

- `card()`: sección con `gap_3`, padding 14, radio `xl`, borde `1px`.
- `screen_header()`: fila con título y acción, borde inferior.
- `section_label()`: 11 px, semibold, mayúsculas, `faint`.
- `label()` / `hint()`: 13 px principal / 11 px tenue.
- `button()` / `danger_button()`: 28 px de alto, padding horizontal 12, radio
  `lg`. Nunca un botón fuera de estas dos piezas.
- `list_row(selected)`: 30 px, fondo `well` o `selected`, borde acento si está
  activa.
- `meter_fill(percent)`: barra de 6 px; el ancho lo calcula el token.

## Reglas

1. Ningún feature escribe `rgb`/`rgba` directamente: usa un token.
2. Radios permitidos: `md` (chips), `lg` (botones/filas), `xl` (tarjetas),
   `full` (medidor).
3. Alturas de control: 28 px (botón), 30 px (fila). No inventar otras.
4. Texto: `text_sm` para contenido, `text_xs` para apoyo. Semibold solo en
   títulos y etiquetas de sección.
5. Separación: `gap_2` dentro de un control, `gap_3` entre bloques, `p(20)`
   en el cuerpo de la ventana.
6. El overlay no pinta fondo: cualquier `bg` en la raíz rompería la
   transparencia.
7. Los estados (`SILENCIO`/`HABLANDO`, `Activada`/`Desactivada`) se comunican
   con color **y** texto; nunca solo con color.
8. No hay animaciones en el MVP. Un cambio de estado es un repintado, no un
   tween.
