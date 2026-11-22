//! YapTap: hold a key, talk, let go. Fully local dictation.

mod cli;
mod engine;
mod history;
mod hotkeys;
mod insert;
mod models;
mod overlay;
mod recorder;
mod session;
mod settings;
mod sounds;
mod text;
mod tray;

use anyhow::{anyhow, Result};
use parking_lot::Mutex;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use settings::{Settings, SettingsStore};

pub struct App {
    pub handle: AppHandle,
    pub settings: SettingsStore,
    pub models: models::ModelStore,
    pub history: history::History,
    pub engine: engine::Engine,
    pub mic: recorder::Mic,
    pub hotkeys: hotkeys::Hotkeys,
    pub phase: Mutex<session::Phase>,
    engine_state: Mutex<EngineState>,
    downloads: Mutex<HashMap<String, (Arc<AtomicBool>, u64)>>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineState {
    /// none | loading | ready | error
    status: &'static str,
    model: Option<String>,
    error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    platform: &'static str,
    version: &'static str,
    engine: EngineState,
    shortcut_ready: bool,
    shortcut_error: Option<String>,
    microphone: &'static str,
    /// macOS only; `None` elsewhere.
    accessibility: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelEntry {
    #[serde(flatten)]
    info: models::ModelInfo,
    installed: bool,
    /// Bytes downloaded so far while a download is running.
    downloading: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress {
    id: String,
    downloaded: u64,
    total: u64,
}

const PLATFORM: &str = if cfg!(target_os = "macos") {
    "macos"
} else if cfg!(target_os = "windows") {
    "windows"
} else {
    "linux"
};

impl App {
    fn status(&self) -> Status {
        use handy_recorder::Permission;
        Status {
            platform: PLATFORM,
            version: env!("CARGO_PKG_VERSION"),
            engine: self.engine_state.lock().clone(),
            shortcut_ready: self.hotkeys.is_running(),
            shortcut_error: self.hotkeys.error.lock().clone(),
            microphone: match handy_recorder::permission_status() {
                Permission::Granted => "granted",
                Permission::Denied => "denied",
                Permission::NotDetermined => "notDetermined",
                _ => "unknown",
            },
            #[cfg(target_os = "macos")]
            accessibility: Some(handy_keys::check_accessibility()),
            #[cfg(not(target_os = "macos"))]
            accessibility: None,
        }
    }

    pub fn emit_status(&self) {
        let _ = self.handle.emit("status", self.status());
    }

    fn set_engine(&self, status: &'static str, model: Option<String>, error: Option<String>) {
        *self.engine_state.lock() = EngineState { status, model, error };
        self.emit_status();
    }
}

/// Loads the selected model if it isn't loaded yet. Blocks while loading.
pub fn ensure_engine(app: &App) -> Result<()> {
    let wanted = app.settings.get().model;
    {
        let state = app.engine_state.lock();
        if state.status == "ready" && state.model.as_deref() == Some(wanted.as_str()) {
            return Ok(());
        }
    }
    let model = models::find(&wanted).ok_or_else(|| anyhow!("Unknown model {wanted}"))?;
    if !app.models.is_installed(model) {
        app.set_engine("none", None, None);
        return Err(anyhow!("Download a speech model first"));
    }
    let vad = models::find(models::VAD_MODEL)
        .filter(|v| app.models.is_installed(v))
        .map(|v| app.models.path(v));
    app.set_engine("loading", Some(wanted.clone()), None);
    match app.engine.load(app.models.path(model), vad) {
        Ok(()) => {
            app.set_engine("ready", Some(wanted), None);
            Ok(())
        }
        Err(e) => {
            app.set_engine("error", Some(wanted), Some(e.to_string()));
            Err(e)
        }
    }
}

fn preload_engine(app: &Arc<App>) {
    let app = app.clone();
    std::thread::spawn(move || {
        let _ = ensure_engine(&app);
    });
}

pub fn open_settings(handle: &AppHandle) {
    if let Some(w) = handle.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

type Res<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

// ---------- commands ----------

#[tauri::command]
fn get_settings(app: State<Arc<App>>) -> Settings {
    app.settings.get()
}

#[tauri::command]
fn set_settings(app: State<Arc<App>>, settings: Settings) -> Res<Settings> {
    let app = app.inner().clone();
    let old = app.settings.get();
    app.settings.set(settings.clone()).map_err(err)?;
    if old.shortcut != settings.shortcut {
        if let Err(e) = app.hotkeys.start(&settings.shortcut) {
            // Keep the old, working shortcut.
            let _ = app.settings.set(old.clone());
            let _ = app.hotkeys.start(&old.shortcut);
            return Err(format!("That shortcut can't be used: {e}"));
        }
    }
    if old.model != settings.model {
        preload_engine(&app);
    }
    if old.history_retention != settings.history_retention {
        app.history.apply_retention(settings.history_retention);
        let _ = app.handle.emit("history-changed", ());
    }
    let _ = app.handle.emit("settings-changed", &settings);
    Ok(settings)
}

#[tauri::command]
fn get_status(app: State<Arc<App>>) -> Status {
    app.status()
}

#[tauri::command]
fn list_models(app: State<Arc<App>>) -> Vec<ModelEntry> {
    let downloads = app.downloads.lock();
    models::CATALOG
        .iter()
        .map(|m| ModelEntry {
            info: m.clone(),
            installed: app.models.is_installed(m),
            downloading: downloads.get(m.id).map(|(_, done)| *done),
        })
        .collect()
}

#[tauri::command]
async fn download_model(app: State<'_, Arc<App>>, id: String) -> Res<()> {
    let app = app.inner().clone();
    if app.settings.get().offline {
        return Err("Offline mode is on. Turn it off in Privacy to download models.".into());
    }
    let model = models::find(&id).ok_or("Unknown model")?;
    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut d = app.downloads.lock();
        if d.contains_key(&id) {
            return Ok(());
        }
        d.insert(id.clone(), (cancel.clone(), 0));
    }
    // The VAD is tiny and always comes along with the first speech model.
    if model.kind == models::Kind::Speech {
        if let Some(vad) = models::find(models::VAD_MODEL) {
            if !app.models.is_installed(vad) {
                let _ = app.models.download(vad, cancel.clone(), |_, _| {}).await;
            }
        }
    }
    let mut last_emit = std::time::Instant::now();
    let result = app
        .models
        .download(model, cancel, |done, total| {
            if let Some(entry) = app.downloads.lock().get_mut(&id) {
                entry.1 = done;
            }
            if last_emit.elapsed().as_millis() > 100 || done == total {
                last_emit = std::time::Instant::now();
                let _ = app.handle.emit(
                    "download-progress",
                    Progress { id: id.clone(), downloaded: done, total },
                );
            }
        })
        .await;
    app.downloads.lock().remove(&id);
    let _ = app.handle.emit("models-changed", ());
    match result {
        Ok(()) => {
            if app.settings.get().model == id {
                preload_engine(&app);
            }
            Ok(())
        }
        Err(e) if e.to_string() == "cancelled" => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn cancel_download(app: State<Arc<App>>, id: String) {
    if let Some((cancel, _)) = app.downloads.lock().get(&id) {
        cancel.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
fn delete_model(app: State<Arc<App>>, id: String) -> Res<()> {
    let model = models::find(&id).ok_or("Unknown model")?;
    let loaded = app.engine_state.lock().model.clone();
    if loaded.as_deref() == Some(id.as_str()) {
        app.engine.unload();
        app.set_engine("none", None, None);
    }
    app.models.delete(model).map_err(err)?;
    let _ = app.handle.emit("models-changed", ());
    Ok(())
}

#[tauri::command]
fn list_mics() -> Vec<recorder::MicInfo> {
    recorder::list_mics()
}

#[tauri::command]
fn get_history(app: State<Arc<App>>) -> Vec<history::Entry> {
    app.history.list()
}

#[tauri::command]
fn delete_history(app: State<Arc<App>>, id: u64) {
    app.history.delete(id, app.settings.get().history_retention);
    let _ = app.handle.emit("history-changed", ());
}

#[tauri::command]
fn clear_history(app: State<Arc<App>>) {
    app.history.clear();
    let _ = app.handle.emit("history-changed", ());
}

#[tauri::command]
fn copy_text(text: String) -> Res<()> {
    insert::copy(&text).map_err(err)
}

#[tauri::command]
fn toggle_dictation(app: State<Arc<App>>) {
    session::toggle(app.inner());
}

#[tauri::command]
fn cancel_dictation(app: State<Arc<App>>) {
    session::cancel(app.inner());
}

#[tauri::command]
async fn record_shortcut() -> Res<String> {
    tauri::async_runtime::spawn_blocking(|| hotkeys::record(std::time::Duration::from_secs(10)))
        .await
        .map_err(err)?
        .map_err(err)
}

/// Opening the mic once makes the OS ask for permission.
#[tauri::command]
async fn request_microphone(app: State<'_, Arc<App>>) -> Res<()> {
    let app = app.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mic = recorder::Mic::default();
        mic.start(None, Arc::new(|_| {}), Box::new(|_| {}));
        std::thread::sleep(std::time::Duration::from_millis(300));
        let r = mic.stop().map(|_| ());
        app.emit_status();
        r
    })
    .await
    .map_err(err)?
    .map_err(err)
}

#[tauri::command]
fn open_privacy_settings(kind: String) {
    let url = match (PLATFORM, kind.as_str()) {
        ("macos", "microphone") => "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone",
        ("macos", "accessibility") => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        ("macos", "keyboard") => "x-apple.systempreferences:com.apple.Keyboard-Settings.extension",
        ("windows", _) => "ms-settings:privacy-microphone",
        _ => return,
    };
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    #[cfg(target_os = "linux")]
    let _ = url;
}

#[tauri::command]
fn get_launch_at_login(app: AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
fn set_launch_at_login(app: AppHandle, enabled: bool) -> Res<()> {
    use tauri_plugin_autostart::ManagerExt;
    let al = app.autolaunch();
    if enabled { al.enable() } else { al.disable() }.map_err(err)
}

#[tauri::command]
fn data_folder(app: State<Arc<App>>) -> String {
    app.handle
        .path()
        .app_data_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

// ---------- startup ----------

pub fn run() {
    env_logger_init();
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = cli::run(&args) {
        std::process::exit(code);
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|handle, args, _cwd| {
            log::info!("Second launch with {args:?}");
            if args.iter().any(|a| a == "--toggle") {
                if let Some(app) = handle.try_state::<Arc<App>>() {
                    session::toggle(app.inner());
                }
            } else {
                open_settings(handle);
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_status,
            list_models,
            download_model,
            cancel_download,
            delete_model,
            list_mics,
            get_history,
            delete_history,
            clear_history,
            copy_text,
            toggle_dictation,
            cancel_dictation,
            record_shortcut,
            request_microphone,
            open_privacy_settings,
            get_launch_at_login,
            set_launch_at_login,
            data_folder,
        ])
        .setup(|tauri_app| {
            #[cfg(target_os = "macos")]
            {
                tauri_app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                let _ = tauri_app.handle().plugin(tauri_nspanel::init());
            }
            let handle = tauri_app.handle().clone();
            let dir = handle.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let settings = SettingsStore::load(&dir);
            let app = Arc::new(App {
                handle: handle.clone(),
                models: models::ModelStore::new(&dir),
                history: history::History::load(&dir),
                engine: engine::Engine::spawn(),
                mic: recorder::Mic::default(),
                hotkeys: hotkeys::Hotkeys::default(),
                phase: Mutex::new(session::Phase::Idle),
                engine_state: Mutex::new(EngineState { status: "none", ..Default::default() }),
                downloads: Mutex::new(HashMap::new()),
                settings,
            });
            handle.manage(app.clone());

            overlay::create(&handle);
            tray::create(&handle)?;
            hotkeys::spawn(app.clone());
            preload_engine(&app);

            // Keep the settings window alive: closing only hides it.
            if let Some(main) = handle.get_webview_window("main") {
                let w = main.clone();
                main.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
                if !app.settings.get().onboarded {
                    open_settings(&handle);
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building YapTap")
        .run(|handle, event| match event {
            // A tray app keeps running when its windows are closed.
            tauri::RunEvent::ExitRequested { api, code: None, .. } => api.prevent_exit(),
            tauri::RunEvent::Exit => {
                if let Some(app) = handle.try_state::<Arc<App>>() {
                    app.mic.cancel();
                    app.engine.shutdown();
                }
            }
            _ => {}
        });
}

fn env_logger_init() {
    struct Logger;
    impl log::Log for Logger {
        fn enabled(&self, m: &log::Metadata) -> bool {
            m.level() <= log::Level::Info && !m.target().starts_with("tao")
        }
        fn log(&self, r: &log::Record) {
            if self.enabled(r.metadata()) {
                eprintln!("[{}] {}", r.level(), r.args());
            }
        }
        fn flush(&self) {}
    }
    static LOGGER: Logger = Logger;
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info));
}
