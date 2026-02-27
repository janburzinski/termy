use crate::config;
use crate::settings_view::SettingsWindow;
use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};

pub(crate) fn open_config_file() {
    if let Err(error) = config::open_config_file() {
        log::error!("Failed to open config file: {}", error);
        termy_toast::error(error.to_string());
    }
}

pub(crate) fn open_settings_window(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

    #[cfg(target_os = "macos")]
    let titlebar = Some(gpui::TitlebarOptions {
        title: Some("Settings".into()),
        appears_transparent: true,
        traffic_light_position: Some(gpui::point(px(12.0), px(10.0))),
        ..Default::default()
    });
    #[cfg(target_os = "windows")]
    let titlebar = Some(gpui::TitlebarOptions {
        title: Some("Settings".into()),
        ..Default::default()
    });
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let titlebar = Some(gpui::TitlebarOptions {
        title: Some("Settings".into()),
        appears_transparent: true,
        ..Default::default()
    });

    let result = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar,
            ..Default::default()
        },
        |window, cx| cx.new(|cx| SettingsWindow::new(window, cx)),
    );

    if let Err(error) = result {
        log::error!("Failed to open settings window: {}", error);
        termy_toast::error(format!("Failed to open settings window: {}", error));
    }
}
