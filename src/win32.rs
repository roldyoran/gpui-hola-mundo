//! Shim Win32 mínimo: arrastrar la ventana y alternar *siempre visible*.
//!
//! GPUI 0.3.14 no expone ninguna de las dos cosas en Windows:
//! `Window::start_window_move()` es un no-op y `WindowOptions` no tiene
//! `always_on_top`. Todo el `unsafe` de la aplicación vive en este módulo y
//! tiene una justificación por llamada. En plataformas no-Windows ambas
//! funciones son no-ops para que el resto del crate siga compilando.

#[cfg(windows)]
mod imp {
    use core::ffi::c_void;

    use gpui_kit::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
    use windows::Win32::UI::WindowsAndMessaging::{
        HTCAPTION, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SendMessageW, SetWindowPos, WM_NCLBUTTONDOWN,
    };

    /// Obtiene el `HWND` nativo de la ventana de GPUI.
    fn hwnd(window: &Window) -> Option<HWND> {
        let handle = HasWindowHandle::window_handle(window).ok()?;
        match handle.as_raw() {
            RawWindowHandle::Win32(win32) => Some(HWND(win32.hwnd.get() as *mut c_void)),
            _ => None,
        }
    }

    /// Arrastra la ventana delegando en el bucle modal nativo de Windows.
    ///
    /// Se llama desde el handler de `mouse down`; `SendMessageW` no regresa
    /// hasta que el usuario suelta el botón.
    pub fn start_drag(window: &Window) {
        let Some(hwnd) = hwnd(window) else {
            return;
        };
        // SAFETY: `hwnd` pertenece a esta ventana y el mensaje es el estándar
        // para iniciar un arrastre de la barra de título. `ReleaseCapture`
        // suelta la captura que GPUI tomó al recibir el clic; sin ella el
        // bucle de movimiento no arranca. No se retiene ningún puntero
        // después de la llamada.
        unsafe {
            let _ = ReleaseCapture();
            let _ = SendMessageW(
                hwnd,
                WM_NCLBUTTONDOWN,
                Some(WPARAM(HTCAPTION as usize)),
                Some(LPARAM(0)),
            );
        }
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
}

#[cfg(windows)]
pub use imp::{set_topmost, start_drag};

#[cfg(not(windows))]
pub fn start_drag(_window: &gpui_kit::Window) {}

#[cfg(not(windows))]
pub fn set_topmost(_window: &gpui_kit::Window, _always_on_top: bool) {}
