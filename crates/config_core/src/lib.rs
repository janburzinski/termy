mod color_keys;
mod constants;
mod diagnostics;
mod parser;
mod path;
mod render;
mod types;

pub use color_keys::{ColorEntryError, apply_color_entry, canonical_color_key};
pub use constants::{SHELL_DECIDE_THEME_ID, VALID_ROOT_KEYS, VALID_SECTIONS};
pub use diagnostics::{ConfigDiagnostic, ConfigDiagnosticKind, ConfigParseReport};
pub use parser::parse_theme_id;
pub use path::config_path;
pub use render::{DEFAULT_CONFIG_TEMPLATE, prettify_config_contents};
pub use types::{
    AppConfig, CursorStyle, CustomColors, KeybindConfigLine, Rgb8, TabCloseVisibility,
    TabTitleConfig, TabTitleMode, TabTitleSource, TabWidthMode, TerminalScrollbarStyle,
    TerminalScrollbarVisibility, ThemeId, WorkingDirFallback,
};

#[cfg(test)]
mod parser_tests;
