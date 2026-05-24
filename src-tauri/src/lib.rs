mod app;
mod proxy;

use anyhow::{Context, Result};
use app::RuntimeState;
use std::sync::Arc;
use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartExt};

const TRAY_MENU_SHOW: &str = "tray_show";
const TRAY_MENU_QUIT: &str = "tray_quit";

pub fn run() {
    let runtime = Arc::new(RuntimeState::new().expect("initialize codex-switch runtime"));
    let setup_runtime = runtime.clone();
    let close_runtime = runtime.clone();
    let run_runtime = runtime.clone();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None::<Vec<&'static str>>,
        ))
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(runtime)
        .setup(move |app| {
            if let Err(error) = sync_launch_at_login_preference(app.handle(), &setup_runtime) {
                log::error!("failed to sync launch-at-login setting: {error}");
            }
            setup_tray(app)?;
            let runtime = setup_runtime.clone();
            tauri::async_runtime::spawn(async move {
                runtime.start_proxy_if_enabled().await;
            });
            Ok(())
        })
        .on_window_event(move |window, event| {
            if !matches!(event, WindowEvent::CloseRequested { .. }) {
                return;
            }

            if close_runtime.is_exiting() {
                return;
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(error) = window.hide() {
                    log::error!("failed to hide main window: {error}");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            app::snapshot,
            app::enable,
            app::disable,
            app::save_provider,
            app::delete_provider,
            app::switch_provider,
            app::parse_import_url,
            app::import_provider,
            app::provider_api_key,
            app::fetch_provider_models,
            app::stats_summary,
            app::update_info,
            app::update_settings,
            app::restore_backup,
            app::select_codex_directory,
        ])
        .build(tauri::generate_context!())
        .expect("error while building codex-switch");

    app.run(move |_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            if let Err(error) = run_runtime.cleanup_before_exit() {
                log::error!("failed to restore Codex config before exit: {error}");
                api.prevent_exit();
            }
        }
    });
}

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text(TRAY_MENU_SHOW, "打开 codex-switch")
        .separator()
        .text(TRAY_MENU_QUIT, "退出")
        .build()?;

    let icon = app.default_window_icon().cloned();
    let mut tray = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("codex-switch")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            TRAY_MENU_SHOW => show_main_window(app),
            TRAY_MENU_QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::DoubleClick { .. }
            | TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => show_main_window(tray.app_handle()),
            _ => {}
        });

    if let Some(icon) = icon {
        tray = tray.icon(icon);
    }
    tray.build(app)?;
    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.show() {
            log::error!("failed to show main window: {error}");
        }
        if let Err(error) = window.set_focus() {
            log::error!("failed to focus main window: {error}");
        }
    }
}

fn sync_launch_at_login_preference(app: &AppHandle, runtime: &RuntimeState) -> Result<()> {
    let desired = runtime.launch_at_login_enabled();
    let autostart = app.autolaunch();
    let current = autostart
        .is_enabled()
        .context("read autostart status from the operating system")?;

    if desired == current {
        return Ok(());
    }

    if desired {
        autostart
            .enable()
            .context("register autostart with the operating system")?;
    } else {
        autostart
            .disable()
            .context("remove autostart from the operating system")?;
    }

    Ok(())
}
