# Benchmarks

> **Estado:** pendiente de la primera build del CI. Este PC no compila, así que
> no hay números propios todavía. La línea base de abajo es el binario del
> proyecto de test (`gpui-hola-mundo`, 1000×700 con JPEG), no este producto.

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
| — | MVP | overlay 320×320 | — | — | — | — | pendiente de CI |
