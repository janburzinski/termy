pub(crate) const DEFAULT_TAB_TITLE_FALLBACK: &str = "Terminal";
pub(crate) const DEFAULT_TAB_TITLE_EXPLICIT_PREFIX: &str = "termy:tab:";
pub(crate) const DEFAULT_TAB_TITLE_PROMPT_FORMAT: &str = "{cwd}";
pub(crate) const DEFAULT_TAB_TITLE_COMMAND_FORMAT: &str = "{command}";
pub(crate) const DEFAULT_TERM: &str = "xterm-256color";
pub(crate) const DEFAULT_COLORTERM: &str = "truecolor";
pub(crate) const DEFAULT_MOUSE_SCROLL_MULTIPLIER: f32 = 3.0;
pub(crate) const DEFAULT_SCROLLBACK_HISTORY: usize = 2000;
pub(crate) const MAX_SCROLLBACK_HISTORY: usize = 100_000;
pub(crate) const DEFAULT_INACTIVE_TAB_SCROLLBACK: Option<usize> = None;
pub(crate) const MIN_MOUSE_SCROLL_MULTIPLIER: f32 = 0.1;
pub(crate) const MAX_MOUSE_SCROLL_MULTIPLIER: f32 = 1_000.0;
pub(crate) const DEFAULT_CURSOR_BLINK: bool = true;
pub(crate) const DEFAULT_WARN_ON_QUIT_WITH_RUNNING_PROCESS: bool = true;

pub const VALID_ROOT_KEYS: &[&str] = &[
    "theme",
    "working_dir",
    "working_dir_fallback",
    "default_working_dir",
    "warn_on_quit_with_running_process",
    "tab_title_priority",
    "tab_title_mode",
    "tab_title_fallback",
    "tab_title_explicit_prefix",
    "tab_title_shell_integration",
    "tab_title_prompt_format",
    "tab_title_command_format",
    "tab_close_visibility",
    "tab_width_mode",
    "show_termy_in_titlebar",
    "shell",
    "term",
    "colorterm",
    "window_width",
    "window_height",
    "font_family",
    "font_size",
    "cursor_style",
    "cursor_blink",
    "background_opacity",
    "background_blur",
    "padding_x",
    "padding_y",
    "mouse_scroll_multiplier",
    "scrollbar_visibility",
    "scrollbar_style",
    "scrollback_history",
    "scrollback",
    "inactive_tab_scrollback",
    "command_palette_show_keybinds",
    "keybind",
];

pub const VALID_SECTIONS: &[&str] = &["colors", "tab_title"];

pub const SHELL_DECIDE_THEME_ID: &str = "shell-decide";
