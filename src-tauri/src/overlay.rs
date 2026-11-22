//! The floating pill. A small transparent window that never takes focus, so
//! the text still lands in the app you were typing in.
//! macOS: a non-activating NSPanel (tauri-nspanel) on every Space.
//! Windows: an always-on-top, non-focusable tool window.
//! Linux: gtk-layer-shell on Wayland compositors that support it, else a
//! plain always-on-top window.

use crate::settings::OverlayPosition;
use tauri::{AppHandle, LogicalPosition, Manager, WebviewUrl};

pub const LABEL: &str = "overlay";
const WIDTH: f64 = 300.0;
const HEIGHT: f64 = 72.0;
const MARGIN: f64 = 56.0;

#[cfg(target_os = "macos")]
tauri_nspanel::tauri_panel! {
    panel!(PillPanel {
        config: {
            can_become_key_window: false,
            is_floating_panel: true
        }
    })
}

#[cfg(target_os = "macos")]
pub fn create(app: &AppHandle) {
    use tauri_nspanel::{CollectionBehavior, PanelBuilder, PanelLevel, StyleMask};
    let built = PanelBuilder::<_, PillPanel>::new(app, LABEL)
        .url(WebviewUrl::App("index.html#overlay".into()))
        .title("YapTap")
        .level(PanelLevel::Status)
        .size(tauri::Size::Logical(tauri::LogicalSize {
            width: WIDTH,
            height: HEIGHT,
        }))
        .has_shadow(false)
        .transparent(true)
        .no_activate(true)
        .corner_radius(0.0)
        .style_mask(StyleMask::empty().borderless().nonactivating_panel())
        .with_window(|w| w.decorations(false).transparent(true).focusable(false))
        .collection_behavior(
            CollectionBehavior::new()
                .can_join_all_spaces()
                .full_screen_auxiliary(),
        )
        .build();
    match built {
        Ok(panel) => panel.hide(),
        Err(e) => log::error!("Could not create the overlay: {e}"),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn create(app: &AppHandle) {
    let built = tauri::WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html#overlay".into()))
        .title("YapTap")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focusable(false)
        .focused(false)
        .visible(false)
        .build();
    match built {
        #[allow(unused_variables)]
        Ok(window) => {
            #[cfg(target_os = "linux")]
            linux_layer_shell(&window);
        }
        Err(e) => log::error!("Could not create the overlay: {e}"),
    }
}

#[cfg(target_os = "linux")]
fn linux_layer_shell(window: &tauri::WebviewWindow) {
    use gtk::prelude::GtkWindowExt;
    use gtk_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
    let Ok(gtk_window) = window.gtk_window() else { return };
    if gtk_layer_shell::is_supported() {
        gtk_window.init_layer_shell();
        gtk_window.set_layer(Layer::Overlay);
        gtk_window.set_keyboard_mode(KeyboardMode::None);
        gtk_window.set_anchor(Edge::Bottom, true);
        gtk_window.set_layer_shell_margin(Edge::Bottom, MARGIN as i32);
    } else {
        gtk_window.set_focus_on_map(false);
    }
}

/// Shows the pill on the screen the mouse is on.
pub fn show(app: &AppHandle, position: OverlayPosition) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Some(window) = handle.get_webview_window(LABEL) else { return };
        if let Some((x, y)) = place(&handle, position) {
            let _ = window.set_position(LogicalPosition::new(x, y));
        }
        let _ = window.show();
    });
}

pub fn hide(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window(LABEL) {
            let _ = window.hide();
        }
    });
}

fn place(app: &AppHandle, position: OverlayPosition) -> Option<(f64, f64)> {
    let cursor = app.cursor_position().ok();
    let monitor = cursor
        .and_then(|c| app.monitor_from_point(c.x, c.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten())?;
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let x0 = area.position.x as f64 / scale;
    let y0 = area.position.y as f64 / scale;
    let w = area.size.width as f64 / scale;
    let h = area.size.height as f64 / scale;
    let x = x0 + (w - WIDTH) / 2.0;
    let y = match position {
        OverlayPosition::Bottom => y0 + h - HEIGHT - MARGIN + 24.0,
        OverlayPosition::Top => y0 + MARGIN - 24.0,
    };
    Some((x, y))
}
