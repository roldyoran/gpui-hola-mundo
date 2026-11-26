//! Opciones de las dos ventanas y ciclo de vida del overlay.
//!
//! El overlay usa `WindowKind::PopUp` (sin borde, sin caption, fuera de la
//! barra de tareas) con fondo transparente. No se crea al arrancar: nace la
//! primera vez que el usuario pulsa **Mostrar**. Ajustes es una ventana normal,
//! redimensionable, que no se puede cerrar; la única salida es **Salir de la
//! app**.

use gpui_kit::*;

use crate::app::AppState;
use crate::features::avatar::AvatarView;
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

/// Ventana de ajustes: normal, redimensionable y sin cierre.
pub fn settings_options(bounds: Bounds<Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        kind: WindowKind::Normal,
        is_resizable: true,
        is_minimizable: true,
        window_min_size: Some(size(px(560.), px(560.))),
        ..Default::default()
    }
}

/// Abre ajustes o reactiva la ventana si ya existe.
pub fn open_settings(state: Entity<AppState>, cx: &mut App) -> anyhow::Result<()> {
    // Si el overlay está a pantalla completa taparía los ajustes. Al pedir
    // ajustes se restaura primero su tamaño para que la ventana sea usable.
    let (avatar_window, maximized) = {
        let state = state.read(cx);
        (state.avatar_window, state.avatar_maximized)
    };
    if maximized {
        if let Some(handle) = avatar_window {
            let _ = handle.update(cx, |_, window, _| {
                crate::win32::set_fullscreen(window, false)
            });
        }
        state.update(cx, |state, cx| {
            state.avatar_maximized = false;
            cx.notify();
        });
    }

    let settings_window = state.read(cx).settings_window;
    let already_open = settings_window.is_some_and(|handle| {
        handle
            .update(cx, |_, window, _| window.activate_window())
            .is_ok()
    });
    if already_open {
        return Ok(());
    }

    let bounds = Bounds::centered(None, size(px(1060.), px(740.)), cx);
    let view_state = state.clone();
    let handle = cx.open_window(settings_options(bounds), move |_, cx| {
        cx.new(|cx| SettingsView::new(view_state, cx))
    })?;
    let handle: AnyWindowHandle = handle.into();
    // La ventana de ajustes no se cierra: solo se minimiza, se maximiza o se
    // sale desde el botón destructivo. `Alt+F4`, la X nativa y el menú del
    // sistema pasan por aquí.
    let _ = handle.update(cx, |_, window, cx| {
        window.on_window_should_close(cx, |_, _| false);
        crate::win32::disable_close(window);
    });
    state.update(cx, |state, _| state.settings_window = Some(handle));
    Ok(())
}

/// Crea el overlay la primera vez o lo devuelve a su última posición.
///
/// Mientras no se llame a esta función no existe ninguna ventana transparente:
/// el primer arranque solo enseña la previsualización de ajustes.
pub fn show_avatar_overlay(state: &Entity<AppState>, cx: &mut App) {
    let (existing, settings, topmost) = {
        let state = state.read(cx);
        (
            state.avatar_window,
            state.settings.clone(),
            state.settings.window.always_on_top,
        )
    };

    if let Some(handle) = existing {
        let _ = handle.update(cx, |_, window, _| crate::win32::show(window));
        state.update(cx, |state, cx| {
            state.avatar_parked = false;
            cx.notify();
        });
        return;
    }

    let bounds = avatar_bounds(&settings, cx);
    let view_state = state.clone();
    let view = cx.new(|cx| AvatarView::new(view_state, cx));
    let Ok(handle) = cx.open_window(avatar_options(bounds), move |_, _| view) else {
        return;
    };
    let handle: AnyWindowHandle = handle.into();
    state.update(cx, |state, cx| {
        state.avatar_window = Some(handle);
        state.avatar_parked = false;
        state.avatar_maximized = false;
        cx.notify();
    });
    let _ = handle.update(cx, |_, window, _| {
        crate::win32::set_topmost(window, topmost);
        crate::win32::remember_origin(window);
    });
}

/// Redimensiona el overlay con los valores ya persistidos.
pub fn resize_avatar(state: &Entity<AppState>, cx: &mut App) {
    let (handle, window_size, maximized) = {
        let state = state.read(cx);
        (
            state.avatar_window,
            size(
                px(state.settings.window.width),
                px(state.settings.window.height),
            ),
            state.avatar_maximized,
        )
    };

    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| {
            if maximized {
                crate::win32::set_fullscreen(window, false);
            }
            window.resize(window_size);
        });
        if maximized {
            state.update(cx, |state, cx| {
                state.avatar_maximized = false;
                cx.notify();
            });
        }
    }
}

/// Saca el avatar de todos los monitores sin dejar de renderizar.
pub fn park_avatar(state: &Entity<AppState>, cx: &mut App) {
    let handle = state.read(cx).avatar_window;
    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| {
            crate::win32::set_fullscreen(window, false);
            crate::win32::park(window);
        });
        state.update(cx, |state, cx| {
            state.avatar_parked = true;
            state.avatar_maximized = false;
            cx.notify();
        });
    }
}

/// Devuelve el avatar a su última posición visible; lo crea si no existe.
pub fn show_avatar(state: &Entity<AppState>, cx: &mut App) {
    if state.read(cx).avatar_window.is_none() {
        show_avatar_overlay(state, cx);
        return;
    }
    let handle = state.read(cx).avatar_window;
    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| crate::win32::show(window));
        state.update(cx, |state, cx| {
            state.avatar_parked = false;
            cx.notify();
        });
    }
}

/// Entra o sale de pantalla completa en el overlay real.
pub fn set_avatar_fullscreen(state: &Entity<AppState>, fullscreen: bool, cx: &mut App) {
    let handle = state.read(cx).avatar_window;
    if let Some(handle) = handle {
        let _ = handle.update(cx, |_, window, _| {
            crate::win32::set_fullscreen(window, fullscreen);
        });
        state.update(cx, |state, cx| {
            state.avatar_maximized = fullscreen;
            if fullscreen {
                state.avatar_parked = false;
            }
            cx.notify();
        });
    }
}
