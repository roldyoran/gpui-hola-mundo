//! Opciones de las dos ventanas y apertura de ajustes.
//!
//! El overlay usa `WindowKind::PopUp` (sin borde, sin caption, fuera de la
//! barra de tareas) con fondo transparente. Ajustes es una ventana normal,
//! no redimensionable, que se puede cerrar sin matar la aplicación porque el
//! avatar sigue vivo.

use gpui_kit::*;

use crate::app::AppState;
use crate::features::settings::{Settings, SettingsView};

/// Bounds iniciales del avatar: los guardados o centrado en pantalla.
pub fn avatar_bounds(settings: &Settings, cx: &mut App) -> Bounds<Pixels> {
    let window_size = size(px(settings.window.width), px(settings.window.height));
    match (settings.window.x, settings.window.y) {
        (Some(x), Some(y)) => Bounds::new(point(px(x), px(y)), window_size),
        _ => Bounds::centered(None, window_size, cx),
    }
}

/// Ventana del avatar: transparente, sin bordes, sin taskbar.
pub fn avatar_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        focus: false,
        show: true,
        kind: WindowKind::PopUp,
        is_movable: true,
        is_resizable: false,
        is_minimizable: false,
        window_background: WindowBackgroundAppearance::Transparent,
        ..Default::default()
    }
}

/// Ventana de ajustes: normal y de tamaño fijo.
pub fn settings_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        kind: WindowKind::Normal,
        is_resizable: false,
        ..Default::default()
    }
}

/// Abre ajustes o reactiva la ventana si ya existe.
pub fn open_settings(state: Entity<AppState>, cx: &mut App) -> anyhow::Result<()> {
    let settings_window = state.read(cx).settings_window;
    let already_open = settings_window.is_some_and(|handle| {
        handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    });
    if already_open {
        return Ok(());
    }

    let bounds = Bounds::centered(None, size(px(620.), px(680.)), cx);
    let view_state = state.clone();
    let handle = cx.open_window(settings_options(bounds), move |_, cx| {
        cx.new(|cx| SettingsView::new(view_state, cx))
    })?;
    let handle: AnyWindowHandle = handle.into();
    state.update(cx, |state, _| state.settings_window = Some(handle));
    Ok(())
}

/// Redimensiona el overlay con los valores ya persistidos.
pub fn resize_avatar(state: &Entity<AppState>, cx: &mut App) {
    let (handle, window_size) = {
        let state = state.read(cx);
        (
            state.avatar_window,
            size(
                px(state.settings.window.width),
                px(state.settings.window.height),
            ),
        )
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| window.resize(window_size));
    }
}

/// Saca el avatar de todos los monitores sin dejar de renderizar.
pub fn park_avatar(state: &Entity<AppState>, cx: &mut App) {
    let handle = state.read(cx).avatar_window;
    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| crate::win32::park(window));
    }
}

/// Devuelve el avatar a su última posición visible.
pub fn show_avatar(state: &Entity<AppState>, cx: &mut App) {
    let handle = state.read(cx).avatar_window;
    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| crate::win32::show(window));
    }
}
