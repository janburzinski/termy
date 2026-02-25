mod catppuccin_mocha;
mod dracula;
mod gruvbox_dark;
mod material_dark;
mod monokai;
mod nord;
mod oceanic_next;
mod one_dark;
mod palenight;
mod solarized_dark;
mod termy;
mod tokyo_night;
mod tomorrow_night;

use std::collections::HashSet;
use std::sync::{OnceLock, RwLock};
pub use termy_theme_core::{
    BUILTIN_THEME_IDS, Rgb8, ThemeColors, canonical_builtin_theme_id, normalize_theme_id,
};

pub trait ThemeProvider: Send + Sync {
    fn theme(&self, theme_id: &str) -> Option<ThemeColors>;

    fn theme_ids(&self) -> &'static [&'static str] {
        &[]
    }
}

#[derive(Default)]
pub struct ThemeRegistry {
    providers: Vec<Box<dyn ThemeProvider>>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        registry.register_provider(BuiltinThemeProvider);
        registry
    }

    pub fn register_provider<P>(&mut self, provider: P)
    where
        P: ThemeProvider + 'static,
    {
        self.providers.push(Box::new(provider));
    }

    pub fn resolve(&self, theme_id: &str) -> Option<ThemeColors> {
        for provider in self.providers.iter().rev() {
            if let Some(theme) = provider.theme(theme_id) {
                return Some(theme);
            }
        }
        None
    }

    pub fn theme_ids(&self) -> Vec<&'static str> {
        let mut seen = HashSet::new();
        let mut ids = Vec::new();
        for provider in &self.providers {
            for id in provider.theme_ids() {
                if seen.insert(*id) {
                    ids.push(*id);
                }
            }
        }
        ids
    }
}

pub struct BuiltinThemeProvider;

impl ThemeProvider for BuiltinThemeProvider {
    fn theme(&self, theme_id: &str) -> Option<ThemeColors> {
        builtin_theme(theme_id)
    }

    fn theme_ids(&self) -> &'static [&'static str] {
        BUILTIN_THEME_IDS
    }
}

static GLOBAL_THEME_REGISTRY: OnceLock<RwLock<ThemeRegistry>> = OnceLock::new();

fn global_theme_registry() -> &'static RwLock<ThemeRegistry> {
    GLOBAL_THEME_REGISTRY.get_or_init(|| RwLock::new(ThemeRegistry::with_builtins()))
}

pub fn register_theme_provider<P>(provider: P)
where
    P: ThemeProvider + 'static,
{
    global_theme_registry()
        .write()
        .expect("Theme registry lock poisoned")
        .register_provider(provider);
}

pub fn resolve_theme(theme_id: &str) -> Option<ThemeColors> {
    global_theme_registry()
        .read()
        .expect("Theme registry lock poisoned")
        .resolve(theme_id)
}

pub fn available_theme_ids() -> Vec<&'static str> {
    global_theme_registry()
        .read()
        .expect("Theme registry lock poisoned")
        .theme_ids()
}

pub fn builtin_theme(theme_id: &str) -> Option<ThemeColors> {
    match canonical_builtin_theme_id(theme_id)? {
        "termy" => Some(termy()),
        "tokyo-night" => Some(tokyo_night()),
        "catppuccin-mocha" => Some(catppuccin_mocha()),
        "dracula" => Some(dracula()),
        "gruvbox-dark" => Some(gruvbox_dark()),
        "nord" => Some(nord()),
        "solarized-dark" => Some(solarized_dark()),
        "one-dark" => Some(one_dark()),
        "monokai" => Some(monokai()),
        "material-dark" => Some(material_dark()),
        "palenight" => Some(palenight()),
        "tomorrow-night" => Some(tomorrow_night()),
        "oceanic-next" => Some(oceanic_next()),
        _ => None,
    }
}

pub fn tokyo_night() -> ThemeColors {
    tokyo_night::theme()
}

pub fn termy() -> ThemeColors {
    termy::theme()
}

pub fn catppuccin_mocha() -> ThemeColors {
    catppuccin_mocha::theme()
}

pub fn dracula() -> ThemeColors {
    dracula::theme()
}

pub fn gruvbox_dark() -> ThemeColors {
    gruvbox_dark::theme()
}

pub fn nord() -> ThemeColors {
    nord::theme()
}

pub fn solarized_dark() -> ThemeColors {
    solarized_dark::theme()
}

pub fn one_dark() -> ThemeColors {
    one_dark::theme()
}

pub fn monokai() -> ThemeColors {
    monokai::theme()
}

pub fn material_dark() -> ThemeColors {
    material_dark::theme()
}

pub fn palenight() -> ThemeColors {
    palenight::theme()
}

pub fn tomorrow_night() -> ThemeColors {
    tomorrow_night::theme()
}

pub fn oceanic_next() -> ThemeColors {
    oceanic_next::theme()
}

fn rgba(r: u8, g: u8, b: u8) -> Rgb8 {
    Rgb8::new(r, g, b)
}
