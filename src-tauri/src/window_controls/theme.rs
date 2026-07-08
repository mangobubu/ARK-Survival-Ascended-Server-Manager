use crate::models::GlobalSettings;
use tauri::{AppHandle, Manager, Runtime, Theme, WebviewWindow, webview::Color};

const DARK_WINDOW_BACKGROUND: Color = Color(2, 10, 19, 255);
const LIGHT_WINDOW_BACKGROUND: Color = Color(243, 247, 251, 255);

pub(super) fn apply_window_theme_preference<R: Runtime>(
    app_handle: &AppHandle<R>,
    settings: &GlobalSettings,
) {
    let native_theme = native_theme_preference(settings);
    app_handle.set_theme(native_theme);

    for window in app_handle.webview_windows().values() {
        apply_single_window_theme(window, settings);
    }
}

fn apply_single_window_theme<R: Runtime>(window: &WebviewWindow<R>, settings: &GlobalSettings) {
    let _ = window.set_theme(native_theme_preference(settings));
    let _ = window.set_background_color(Some(window_background_color(window, settings)));
}

fn native_theme_preference(settings: &GlobalSettings) -> Option<Theme> {
    match settings.theme.as_str() {
        "light" => Some(Theme::Light),
        "dark" => Some(Theme::Dark),
        _ => None,
    }
}

fn window_background_color<R: Runtime>(
    window: &WebviewWindow<R>,
    settings: &GlobalSettings,
) -> Color {
    match settings.theme.as_str() {
        "light" => LIGHT_WINDOW_BACKGROUND,
        "dark" => DARK_WINDOW_BACKGROUND,
        _ => match window.theme().unwrap_or(Theme::Dark) {
            Theme::Light => LIGHT_WINDOW_BACKGROUND,
            Theme::Dark => DARK_WINDOW_BACKGROUND,
            _ => DARK_WINDOW_BACKGROUND,
        },
    }
}
