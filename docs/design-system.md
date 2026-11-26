# Sistema de diseño

Una sola ventana de ajustes, un solo tema oscuro y controles propios. La
paleta replica las variables de **shadcn/ui en modo oscuro**; la composición
sigue a `gentle-shell-gpui`. Todo sale de `src/shared/ui.rs`.

## Tokens

| Token | Valor | Uso |
| --- | --- | --- |
| `window_bg` | `#09090b` | fondo de la ventana (`--background`) |
| `card_bg` | `#0c0c0e` | tarjetas y fondos de botón secundario |
| `muted_bg` | `#18181b` | filas, pozos, controles segmentados |
| `hover_bg` | `#27272a` | hover de filas y botones |
| `active_bg` | `#3f3f46` | control pulsado |
| `border` | `#27272a` | bordes de tarjetas y controles |
| `border_subtle` | `#1f1f23` | separadores internos |
| `fg` | `#fafafa` | texto principal (`--foreground`) |
| `muted` | `#a1a1aa` | texto secundario (`--muted-foreground`) |
| `faint` | `#71717a` | pistas, escalas y placeholders |
| `primary` | `#fafafa` | botón principal y switch encendido |
| `primary_fg` | `#18181b` | texto sobre el botón principal |
| `success` | `#34d399` | micrófono escuchando / boca abierta |
| `warning` | `#fbbf24` | umbral del VAD y avisos |
| `danger` | `#f87171` | errores |
| `destructive` | `#7f1d1d` | salir de la aplicación |

## Piezas

- `section(title, description)`: título de sección + descripción; la card va
  debajo con `gap_3`.
- `card()`: superficie `card_bg`, borde `1px`, radio `lg`, `overflow_hidden`.
  Las filas ponen su padding; `card_body()` añade `p(16)` y `gap_4`.
- `screen_header()`: 64 px, marca/título/subtítulo y acción ghost, borde
  inferior.
- `row()`: 52 px, padding horizontal 12, radio `md`.
- `label()` / `hint()` / `section_label()`: 14 px principal, 12 px tenue,
  12 px mayúsculas.
- `button()`: secundario (fondo oscuro + borde). `primary_button()`: claro
  sobre oscuro. `ghost_button()`: acciones discretas. `destructive_button()`:
  salir. `icon_button()`: cuadrado de 32 px.
- `switch(on)`: 36×20, knob de 16, animación de estado con color.
- `list_row(selected)`: fila de micrófono con `radio()` y hover.
- `segmented()` / `segment()`: presets de sensibilidad.
- `meter(fill, threshold, speaking)`: barra de 12 px con relleno, umbral ámbar
  y color de éxito al hablar.
- `notice(message, color)`: aviso con tinte, descartable desde la vista.

## Reglas

1. Ningún feature escribe `rgb`/`rgba` directamente: usa un token.
2. Radios: `md` (botones, filas, segmentos), `lg` (tarjetas), `full` (switch,
   radio, medidor). No se inventan otros.
3. Alturas: 32 px (botón), 40 px (fila de micrófono), 52 px (fila de ajuste),
   64 px (cabecera). El cuerpo usa `p(24)`.
4. Texto: `text_sm` para contenido, `text_xs` para apoyo, `text_lg` solo para
   la lectura de dB.
5. Separación: `gap_2` dentro de un control, `gap_3` dentro de una card,
   `gap_6` entre secciones.
6. Todo control clicable tiene `cursor_pointer`, `hover` y `active`. El estado
   seleccionado no depende solo del color: usa radio, chip o texto.
7. El overlay no pinta fondo: cualquier `bg` en su raíz rompería la
   transparencia.
8. La ventana de ajustes tiene cabecera y pie fijos; el cuerpo es el único
   `overflow_y_scroll`. Ninguna sección puede quedar cortada.
9. No hay animaciones de entrada. El medidor es el único movimiento y solo
   corre mientras la ventana de ajustes existe.
