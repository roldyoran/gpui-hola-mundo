# Arquitectura

Monolito modular por features, con fronteras explícitas entre estado, sistema
operativo y vista. Es la misma forma de `gentle-shell-gpui`, reducida a lo que
este producto necesita.

## Componentes

```mermaid
flowchart TD
    Main["main: arranque y composición"] --> Window["window: opciones y apertura"]
    Main --> App["app: AppState global"]
    Window --> Settings["features/settings: esquema, UI y preview"]
    Window -->|"al pulsar Mostrar"| Avatar["features/avatar: PNG y overlay"]

    App --> Audio["features/audio: hilo dueño, RMS, VAD"]
    App --> Settings
    Avatar --> State["AppState (observado)"]
    Settings --> State
    Avatar --> UI["shared/ui: tokens"]
    Settings --> UI
    Settings --> Native["win32: topmost, ocultar/mostrar"]
    Avatar --> Native2["win32: arrastre y resize"]
```

| Pieza | Responsabilidad | No hace |
| --- | --- | --- |
| `src/main.rs` | `application().run`, rutas, ajustes y apertura inicial de la ventana de ajustes. Nunca crea el overlay. | lógica de producto |
| `src/app.rs` | `AppState`: ajustes, boca, estado de audio, handles y flag de aparcado; bomba canal → `notify` | dibujar ni decidir VAD |
| `src/window.rs` | `WindowOptions` de overlay/ajustes, abrir ajustes, crear el overlay al pulsar **Mostrar**, redimensionar y aparcar/mostrar | controles |
| `src/win32.rs` | arrastre y redimensionado manuales con `SetWindowPos`, topmost, traer al frente, aparcado fuera de pantalla, `SC_CLOSE` deshabilitado y preparación para OBS (`prepare_capture`: quita `WS_EX_TOOLWINDOW` y oculta el botón de la barra con `ITaskbarList`) | reglas de negocio |
| `features/avatar` | validar/copiar PNG, decodificar una vez, render del overlay y zonas de resize | conocer el audio |
| `features/audio` | enumerar dispositivos, hilo dueño, stream cpal, VAD, eventos | tocar GPUI |
| `features/settings` | esquema v1, rutas de datos, escritura atómica, UI y previsualización lateral | dibujar el overlay real |
| `shared/ui` | tokens y piezas visuales | conocer features |

## Hilos

```text
Hilo principal (GPUI)
  overlay + ajustes
        ▲
        │ async-channel (solo cambios de boca / estado)
        │
Tarea GPUI ── recv().await ── AppState::on_audio_event → notify
        ▲
        │ async-channel acotado (8)
        │
Hilo dueño de audio ── dormido en recv_blocking
        │  Apply(config)
        ▼
Stream WASAPI (cpal) ── callback: RMS → VAD → try_send solo al cambiar
        │
        └─ AtomicU32 con el último dBFS (solo lo lee la vista de ajustes)
```

- El callback de captura no asigna, no bloquea y no consulta el reloj.
- El hilo dueño reconcilia la configuración: apaga, enciende, cambia de
  dispositivo o solo actualiza la sensibilidad (atómica, sin reconstruir).
- En silencio no se notifica a la UI. Hablando hay una notificación por
  transición, no un frame por buffer.
- Un cambio de boca produce **una** notificación; la vista observa `AppState`.
- El medidor de nivel no usa el canal: lee un `AtomicU32` cada 45 ms mientras
  la ventana de ajustes está abierta. El overlay no se redibuja por el nivel.
- El arrastre y el redimensionado no usan el bucle modal de Windows: una tarea
  de GPUI lee el cursor cada 16 ms y mueve el `HWND` con `SetWindowPos`. El
  overlay no se crea al arrancar; `window::show_avatar_overlay` lo abre la
  primera vez que el usuario pulsa **Mostrar**. El VAD sigue corriendo aunque
  no exista overlay, porque el audio vive en `AppState`.
- La previsualización de ajustes escucha el mismo `mouth_open` y comparte las
  texturas ya decodificadas de `AppState`; no crea una segunda ventana, no
  duplica el stream y no vuelve a decodificar los PNG.
- Cada ventana tiene título único (`PNGTuber Lite · avatar` y
  `PNGTuber Lite · ajustes`) para que OBS las distinga. Al crear el overlay
  se fija el título, se llama a `win32::prepare_capture` y después se
  reaplican `set_topmost` y `raise`; el overlay puede aparecer en Alt+Tab,
  pero no deja botón en la barra de tareas.
- La ventana de ajustes no se cierra: `on_window_should_close` devuelve
  `false` y `SC_CLOSE` se elimina del menú del sistema. El botón **Salir de la
  app** es la única salida; minimizar/maximizar siguen siendo nativos.

## Flujo de datos

```mermaid
flowchart LR
    Mic["Micrófono"] --> RMS["RMS → dBFS"]
    RMS --> VAD["Vad (ataque 40 ms / cierre 250 ms)"]
    VAD -->|cambio| Chan["async-channel"]
    Chan --> App["AppState.mouth_open"]
    App -->|notify| Overlay["AvatarView: img(cerrado|abierto)"]
    App -->|notify| Preview["SettingsView: preview lateral"]
```

La posición y el tamaño se leen de `Window::bounds()` después de arrastrar o
redimensionar y se persisten en `settings.json`. `window::avatar_bounds`
restaura esos valores la primera vez que se pulsa **Mostrar**.

## Persistencia

```text
%APPDATA%\pngtuber-lite\
├── settings.json      esquema v1
└── avatars\
    ├── closed.png
    └── open.png
```

- JSON corrupto o `schema_version != 1` → valores por defecto.
- Escritura en `settings.json.tmp` + `fs::rename` (en Windows reemplaza).
- Los PNG se copian al directorio de datos para no depender del original.

## Reglas de dependencia

1. `vad.rs` y `settings/mod.rs` no dependen de GPUI ni de cpal.
2. `stream.rs` depende de cpal, nunca de GPUI.
3. Las vistas dependen de `AppState` y `shared`; nunca al revés.
4. `shared` no conoce ningún feature.
5. `unsafe` solo en `win32.rs` (plataforma) y aislado por función.
6. Los tipos se quedan privados salvo que crucen una frontera real.

## Decisión sobre GPUI

GPUI en Windows mantiene un hilo de VSync que invalida las ventanas visibles en
cada refresco. Eso fija un piso de CPU (~1,2 % de un núcleo medido en el
proyecto base de 1000×700). Se acepta en el MVP. Si en el futuro se exigiera
0 %, el overlay migraría a `WS_EX_LAYERED` + `UpdateLayeredWindow` sin tocar
`vad.rs`, `settings` ni `app.rs`; el resto de la arquitectura ya está separado.
