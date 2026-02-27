use super::*;

impl TerminalView {
    pub(in super::super) fn execute_app_system_command_action(
        &mut self,
        action: CommandAction,
        cx: &mut Context<Self>,
    ) -> bool {
        match action {
            CommandAction::OpenConfig => {
                self.open_config_action(cx);
                true
            }
            CommandAction::ImportColors => {
                self.import_colors_action(cx);
                true
            }
            CommandAction::AppInfo => {
                self.app_info_action(cx);
                true
            }
            CommandAction::NativeSdkExample => {
                self.native_sdk_example_action(cx);
                true
            }
            CommandAction::OpenSettings => {
                self.open_settings_action(cx);
                true
            }
            CommandAction::CheckForUpdates => {
                self.check_for_updates_action(cx);
                true
            }
            _ => false,
        }
    }

    fn open_config_action(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = config::open_config_file() {
            log::error!("Failed to open config file from command action: {}", error);
            termy_toast::error(error.to_string());
            cx.notify();
        }
    }

    fn import_colors_action(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx: &mut AsyncApp| {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("JSON", &["json"])
                .set_title("Import Colors")
                .pick_file()
                .await;

            let Some(file) = file else {
                return;
            };

            let path = file.path().to_path_buf();
            let result = config::import_colors_from_json(&path);

            let _ = cx.update(|cx| {
                this.update(cx, |view, cx| {
                    match result {
                        Ok(msg) => {
                            termy_toast::success(msg);
                            view.reload_config(cx);
                        }
                        Err(err) => {
                            termy_toast::error(err);
                        }
                    }
                    cx.notify();
                })
            });
        })
        .detach();
    }

    fn app_info_action(&self, cx: &mut Context<Self>) {
        let config_path = self
            .config_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".to_string());
        let message = format!(
            "Termy v{} | {}-{} | config: {}",
            crate::APP_VERSION,
            std::env::consts::OS,
            std::env::consts::ARCH,
            config_path
        );
        termy_toast::info(message);
        cx.notify();
    }

    fn native_sdk_example_action(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx: &mut AsyncApp| {
            termy_native_sdk::show_alert(
                "Update Available",
                "A new Termy update is available and ready to install.",
            );
            let confirmed = termy_native_sdk::confirm(
                "Install Update",
                "Would you like to install the latest update now?",
            );

            let _ = cx.update(|cx| {
                this.update(cx, |_view, cx| {
                    if confirmed {
                        termy_toast::success("Update install confirmed");
                    } else {
                        termy_toast::info("Update installation postponed");
                    }
                    cx.notify();
                })
            });
        })
        .detach();
    }

    fn open_settings_action(&mut self, cx: &mut Context<Self>) {
        use crate::settings_view::SettingsWindow;
        use gpui::{Bounds, WindowBounds, WindowOptions, px, size};
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

        let open_result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar,
                ..Default::default()
            },
            |window, cx| cx.new(|cx| SettingsWindow::new(window, cx)),
        );
        if let Err(error) = open_result {
            log::error!("Failed to open settings window: {}", error);
            termy_toast::error(format!("Failed to open settings window: {}", error));
            cx.notify();
        }
    }

    fn check_for_updates_action(&mut self, cx: &mut Context<Self>) {
        #[cfg(target_os = "macos")]
        {
            if let Some(updater) = self.auto_updater.as_ref() {
                AutoUpdater::check(updater.downgrade(), cx);
                self.update_check_toast_id = Some(termy_toast::loading("Checking for updates"));
                cx.notify();
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            termy_toast::info("Auto updates are only available on macOS");
            cx.notify();
        }
    }
}
