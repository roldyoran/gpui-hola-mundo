# Benchmarks

> **Estado:** el primer build del CI salió en verde (run `45727850849`, 2026-11-26): fmt, clippy, 12 tests, release y artefacto. La medición es parcial: RAM e hilos ya tienen número; CPU, GPU y arranque los valida el usuario con el `.exe` descargado.
>
> **Última verificación:** run `46101092930` (2026-11-27) con la optimización y la preparación para OBS: fmt, clippy, 13 tests, release y artefacto. Falta la prueba humana de OBS/WGC y la medición de CPU, GPU y arranque.

## Límites del MVP

| Métrica | Límite | Cómo se mide |
| --- | --- | --- |
| RAM privada | ≤ 40 MB | `PrivateMemorySize64` |
| Working set | ≤ 70 MB | `WorkingSet64` |
| CPU en reposo (60 s) | ≤ 1,5 % de un núcleo | delta de `TotalProcessorTime` |
| CPU hablando (60 s) | ≤ 3 % de un núcleo | ídem, hablando en voz normal |
| GPU en reposo | ≤ 1 % | columna GPU del Administrador de tareas |
| Arranque | < 400 ms | `Measure-Command` hasta ventana visible |
| Binario | ≤ 16 MB | `target/release/pngtuber-lite.exe` |

## Línea base conocida (proyecto de test)

| Métrica | Valor |
| --- | --- |
| Working set | 47,3–57,0 MB |
| Memoria privada | 36,3 MB |
| Hilos | 16 |
| CPU en reposo | 1,20 % y 1,41 % de un núcleo (dos muestras de 30 s) |

El piso lo marca el hilo de VSync de GPUI en Windows: `DwmFlush()` +
`RedrawWindow(RDWM_INVALIDATE)` en cada refresco, para todas las ventanas
visibles. No se puede desactivar desde la aplicación. Se acepta como coste del
MVP; el presupuesto de CPU lo incluye.

## Protocolo

Descarga el `.exe` del artefacto del CI, colócalo en una carpeta propia y
ejecuta en PowerShell:

```powershell
# RAM + hilos
$p = Start-Process .\pngtuber-lite.exe -PassThru
Start-Sleep 5; $p.Refresh()
"WS $([math]::Round($p.WorkingSet64/1MB,1)) MB | Priv $([math]::Round($p.PrivateMemorySize64/1MB,1)) MB | Hilos $($p.Threads.Count)"

# CPU en reposo (60 s)
$p.Refresh(); $c1 = $p.TotalProcessorTime.TotalMilliseconds
Start-Sleep 60; $p.Refresh(); $c2 = $p.TotalProcessorTime.TotalMilliseconds
"CPU % de un núcleo: $([math]::Round(($c2-$c1)/60000*100,3))"

# Arranque (con el proceso ya cerrado)
Measure-Command { Start-Process .\pngtuber-lite.exe; Start-Sleep 1 }
```

Para CPU hablando, repite la medición de 60 s leyendo un texto en voz normal.
La GPU se lee en la columna GPU del Administrador de tareas (no hay contador
estable por proceso sin ETW).

## Registro

Cada medición se anota aquí con fecha, máquina, resolución, comando y resultado.
Sin número no hay optimización.

| Fecha | Versión | Escenario | WS | Privada | CPU | GPU | Notas |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 2026-11-25 | test base | 1000×700 JPEG | 47–57 MB | 36,3 MB | 1,20–1,41 % | — | referencia, no este producto |
| 2026-11-26 | CI `d2e13c4` | overlay 320×320 + ajustes abiertos, sin PNG | 51,2 MB | 30,7 MB | pendiente | pendiente | 17 hilos; binario 11.433.984 B (10,9 MiB); `WaitForInputIdle` falló, arranque no medido; lo prueba el humano |
| 2026-11-26 | CI `6563b38` | UI shadcn + medidor de nivel + modo OBS | — | — | — | — | binario 11.640.320 B (11,1 MiB); arranque aislado correcto; medición de RAM/CPU pendiente |
| 2026-11-26 | CI `ae791b3` | ajustes sin cierre + minimizar/maximizar | — | — | — | — | binario 11.643.392 B (11,1 MiB); `WM_CLOSE` bloqueado y `SC_CLOSE` ausente verificados por Win32 |
| 2026-11-26 | CI `ff8d90c` | arranque solo ajustes + preview con 2 PNG (audio off) | 53,1 MB | 38,7 MB | — | — | 17 hilos; binario 11.685.376 B (11,1 MiB); scroll verificado; **Mostrar** crea el overlay 320×320 en la posición guardada; sin pantalla completa |
| 2026-11-26 | CI `f6d53b0` | release con subsistema Windows + `Mostrar` al frente | — | — | — | — | binario 11.685.376 B (11,1 MiB); cabecera PE `SUBSYSTEM=2` (GUI), sin consola; pendiente prueba humana de orden Z |
| 2026-11-27 | CI `337e0bd` | optimización + OBS (`prepare_capture`), sin medición aún | — | — | — | — | fmt, clippy, 13 tests, release; artefacto 4.711.535 B; binario 11.699.200 B (11,2 MiB); pendiente prueba humana de OBS/WGC (transparencia, barra de tareas) y medición |
