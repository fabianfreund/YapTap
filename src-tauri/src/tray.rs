//! The menu bar / system tray icon and its menu.

use crate::{open_settings, session, App};
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

pub fn create(handle: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(handle, "toggle", "Start / Stop Dictation", true, None::<&str>)?;
    let paste_last = MenuItem::with_id(handle, "paste-last", "Copy Last Transcript", true, None::<&str>)?;
    let settings = MenuItem::with_id(handle, "settings", "Settings…", true, Some("CmdOrCtrl+,"))?;
    let quit = MenuItem::with_id(handle, "quit", "Quit YapTap", true, Some("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(
        handle,
        &[
            &toggle,
            &paste_last,
            &PredefinedMenuItem::separator(handle)?,
            &settings,
            &PredefinedMenuItem::separator(handle)?,
            &quit,
        ],
    )?;

    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;
    TrayIconBuilder::with_id("main")
        .icon(icon)
        .icon_as_template(true)
        .tooltip("YapTap")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|handle, event| {
            let Some(app) = handle.try_state::<Arc<App>>() else { return };
            match event.id.as_ref() {
                "toggle" => session::toggle(app.inner()),
                "paste-last" => {
                    if let Some(last) = app.history.last() {
                        let _ = crate::insert::copy(&last.text);
                    }
                }
                "settings" => open_settings(handle),
                "quit" => handle.exit(0),
                _ => {}
            }
        })
        .build(handle)?;
    Ok(())
}
