//! Shim Win32: arrastre manual, siempre visible, modo OBS y atajo global.
//!
//! GPUI 0.3.14 no expone arrastre ni `always_on_top` en Windows. Todo el
//! `unsafe` de la aplicación vive en este módulo y tiene una justificación por
//! llamada. El arrastre **no** usa el bucle modal de `WM_NCLBUTTONDOWN`:
//! consulta el cursor y mueve con `SetWindowPos`, así un clic derecho nunca
//! anida eventos de GPUI.
//!
//! En plataformas no-Windows todas las funciones son no-ops para que el resto
//! del crate siga compilando.

/// Resultado de un paso del arrastre manual.
#[derive(Clone, Copy, Debug)]
pub enum DragStep {
    /// El botón izquierdo sigue pulsado; la ventana ya se movió.
    Active,
    /// Se soltó el botón; estas son las bounds lógicas finales.
    Finished {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
}

#[cfg(windows)]
mod imp {
    use core::ffi::c_void;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

    use gpui_kit::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::{HWND, POINT, RECT};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, RegisterHotKey, VK_LBUTTON, VK_P,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetMessageW, GetSystemMetrics, GetWindowRect, HWND_NOTOPMOST, HWND_TOPMOST,
        MSG, SM_CXVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, WM_HOTKEY,
    };

    use super::DragStep;

    /// Separación a la derecha del escritorio virtual para aparcar el avatar.
    const PARK_GAP: i32 = 240;
    /// Identificador del atajo global.
    const HOTKEY_ID: i32 = 0x504E47;

    #[derive(Clone, Copy)]
    struct DragState {
        hwnd: isize,
        start_cursor: (i32, i32),
        start_window: (i32, i32),
        logical: (f32, f32, f32, f32),
        scale: f32,
    }

    static DRAG: Mutex<Option<DragState>> = Mutex::new(None);
    static AVATAR: AtomicIsize = AtomicIsize::new(0);
    static PARKED: AtomicBool = AtomicBool::new(false);
    static VISIBLE: Mutex<Option<(i32, i32)>> = Mutex::new(None);

    fn hwnd_from(raw: isize) -> HWND {
        HWND(raw as *mut c_void)
    }

    /// Obtiene el `HWND` nativo de la ventana de GPUI.
    fn hwnd(window: &Window) -> Option<HWND> {
        let handle = HasWindowHandle::window_handle(window).ok()?;
        match handle.as_raw() {
            RawWindowHandle::Win32(win32) => Some(HWND(win32.hwnd.get() as *mut c_void)),
            _ => None,
        }
    }

    fn cursor_pos() -> Option<(i32, i32)> {
        let mut point = POINT::default();
        // SAFETY: `point` es una variable local válida y `GetCursorPos` solo
        // escribe en ella.
        unsafe { GetCursorPos(&mut point).ok()? };
        Some((point.x, point.y))
    }

    fn window_rect(hwnd: HWND) -> Option<RECT> {
        let mut rect = RECT::default();
        // SAFETY: `rect` es una variable local válida y `hwnd` pertenece a
        // una ventana viva.
        unsafe { GetWindowRect(hwnd, &mut rect).ok()? };
        Some(rect)
    }

    fn set_visible_origin(rect: &RECT) {
        if let Ok(mut visible) = VISIBLE.lock() {
            *visible = Some((rect.left, rect.top));
        }
    }

    /// Empieza un arrastre manual; no bloquea ni abre bucles modales.
    pub fn begin_drag(window: &Window, scale: f32, x: f32, y: f32, width: f32, height: f32) {
        let Some(hwnd) = hwnd(window) else {
            return;
        };
        let Some((cursor_x, cursor_y)) = cursor_pos() else {
            return;
        };
        let Some(rect) = window_rect(hwnd) else {
            return;
        };
        let scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        if let Ok(mut drag) = DRAG.lock() {
            *drag = Some(DragState {
                hwnd: hwnd.0 as isize,
                start_cursor: (cursor_x, cursor_y),
                start_window: (rect.left, rect.top),
                logical: (x, y, width, height),
                scale,
            });
        }
    }

    /// Avanza el arrastre; `None` si no hay ninguno activo.
    pub fn drag_step() -> Option<DragStep> {
        let mut guard = DRAG.lock().ok()?;
        let state = *guard.as_ref()?;
        let (cursor_x, cursor_y) = cursor_pos()?;
        // SAFETY: `GetAsyncKeyState` solo consulta el estado global del teclado
        // y del ratón; no retiene punteros.
        let pressed = (unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } as u16 & 0x8000) != 0;

        let window_x = state.start_window.0 + (cursor_x - state.start_cursor.0);
        let window_y = state.start_window.1 + (cursor_y - state.start_cursor.1);
        // SAFETY: el `HWND` se guardó de una ventana viva; `SWP_NOSIZE |
        // SWP_NOZORDER | SWP_NOACTIVATE` solo cambia la posición y no roba el
        // foco.
        unsafe {
            let _ = SetWindowPos(
                hwnd_from(state.hwnd),
                None,
                window_x,
                window_y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }

        if pressed {
            return Some(DragStep::Active);
        }

        *guard = None;
        if let Ok(mut visible) = VISIBLE.lock() {
            *visible = Some((window_x, window_y));
        }
        let dx = (cursor_x - state.start_cursor.0) as f32 / state.scale;
        let dy = (cursor_y - state.start_cursor.1) as f32 / state.scale;
        Some(DragStep::Finished {
            x: state.logical.0 + dx,
            y: state.logical.1 + dy,
            width: state.logical.2,
            height: state.logical.3,
        })
    }

    /// Pone la ventana por encima (o por debajo) del resto sin moverla.
    pub fn set_topmost(window: &Window, always_on_top: bool) {
        let Some(hwnd) = hwnd(window) else {
            return;
        };
        let insert_after = if always_on_top {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        // SAFETY: `hwnd` es válido mientras la ventana exista; el flag
        // `SWP_NOMOVE | SWP_NOSIZE` garantiza que solo cambia el orden Z y
        // `SWP_NOACTIVATE` evita robar el foco.
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                Some(insert_after),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    fn park_hwnd(hwnd: HWND) {
        if PARKED.swap(true, Ordering::Relaxed) {
            return;
        }
        let Some(rect) = window_rect(hwnd) else {
            return;
        };
        set_visible_origin(&rect);
        // Derecha de todos los monitores: sigue renderizando, pero no ocupa
        // espacio visible. OBS puede capturarla con Window Capture.
        // SAFETY: `GetSystemMetrics` solo lee métricas del sistema.
        let left = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
        let top = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
        let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        // SAFETY: `hwnd` es válido; no se cambia tamaño ni orden Z.
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                None,
                left + width + PARK_GAP,
                top,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    fn show_hwnd(hwnd: HWND) {
        if !PARKED.swap(false, Ordering::Relaxed) {
            return;
        }
        let origin = VISIBLE.lock().ok().and_then(|visible| *visible);
        let (x, y) = origin.unwrap_or_else(|| {
            // SAFETY: `GetSystemMetrics` solo lee métricas del sistema.
            unsafe {
                (
                    GetSystemMetrics(SM_XVIRTUALSCREEN) + 40,
                    GetSystemMetrics(SM_YVIRTUALSCREEN) + 40,
                )
            }
        });
        // SAFETY: `hwnd` es válido; solo se restaura la posición guardada.
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                None,
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    /// Saca el avatar de todos los monitores sin dejar de renderizar.
    pub fn park(window: &Window) {
        if let Some(hwnd) = hwnd(window) {
            park_hwnd(hwnd);
        }
    }

    /// Devuelve el avatar a su última posición visible.
    pub fn show(window: &Window) {
        if let Some(hwnd) = hwnd(window) {
            show_hwnd(hwnd);
        }
    }

    fn toggle_park() {
        let raw = AVATAR.load(Ordering::Relaxed);
        if raw == 0 {
            return;
        }
        let hwnd = hwnd_from(raw);
        if PARKED.load(Ordering::Relaxed) {
            show_hwnd(hwnd);
        } else {
            park_hwnd(hwnd);
        }
    }

    /// Registra `Ctrl+Alt+P` para alternar visible/aparcado sin abrir ajustes.
    ///
    /// El hilo solo atiende `WM_HOTKEY`; no toca GPUI ni el hilo principal.
    pub fn install_hotkey(window: &Window) {
        let Some(hwnd) = hwnd(window) else {
            return;
        };
        AVATAR.store(hwnd.0 as isize, Ordering::Relaxed);
        if let Some(rect) = window_rect(hwnd) {
            set_visible_origin(&rect);
        }
        let _ = std::thread::Builder::new()
            .name("pngtuber-hotkey".to_string())
            .spawn(|| {
                // SAFETY: el atajo se registra en este hilo y `GetMessageW`
                // consume su propia cola; no hay punteros compartidos.
                unsafe {
                    if RegisterHotKey(
                        None,
                        HOTKEY_ID,
                        MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
                        VK_P.0 as u32,
                    )
                    .is_err()
                    {
                        return;
                    }
                    let mut message = MSG::default();
                    while GetMessageW(&mut message, None, 0, 0).as_bool() {
                        if message.message == WM_HOTKEY {
                            toggle_park();
                        }
                    }
                }
            });
    }
}

#[cfg(windows)]
pub use imp::{begin_drag, drag_step, install_hotkey, park, set_topmost, show};

#[cfg(not(windows))]
pub fn begin_drag(
    _window: &gpui_kit::Window,
    _scale: f32,
    _x: f32,
    _y: f32,
    _width: f32,
    _height: f32,
) {
}

#[cfg(not(windows))]
pub fn drag_step() -> Option<DragStep> {
    None
}

#[cfg(not(windows))]
pub fn set_topmost(_window: &gpui_kit::Window, _always_on_top: bool) {}

#[cfg(not(windows))]
pub fn park(_window: &gpui_kit::Window) {}

#[cfg(not(windows))]
pub fn show(_window: &gpui_kit::Window) {}

#[cfg(not(windows))]
pub fn install_hotkey(_window: &gpui_kit::Window) {}
