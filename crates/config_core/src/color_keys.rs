use crate::types::{CustomColors, Rgb8};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorEntryError {
    UnknownKey,
    InvalidValue,
}

#[derive(Clone, Copy)]
enum ColorSlot {
    Foreground,
    Background,
    Cursor,
    Ansi(usize),
}

pub fn canonical_color_key(key: &str) -> Option<&'static str> {
    match key.trim().to_ascii_lowercase().as_str() {
        "foreground" | "fg" => Some("foreground"),
        "background" | "bg" => Some("background"),
        "cursor" => Some("cursor"),
        "black" | "color0" => Some("black"),
        "red" | "color1" => Some("red"),
        "green" | "color2" => Some("green"),
        "yellow" | "color3" => Some("yellow"),
        "blue" | "color4" => Some("blue"),
        "magenta" | "color5" => Some("magenta"),
        "cyan" | "color6" => Some("cyan"),
        "white" | "color7" => Some("white"),
        "bright_black" | "brightblack" | "color8" => Some("bright_black"),
        "bright_red" | "brightred" | "color9" => Some("bright_red"),
        "bright_green" | "brightgreen" | "color10" => Some("bright_green"),
        "bright_yellow" | "brightyellow" | "color11" => Some("bright_yellow"),
        "bright_blue" | "brightblue" | "color12" => Some("bright_blue"),
        "bright_magenta" | "brightmagenta" | "color13" => Some("bright_magenta"),
        "bright_cyan" | "brightcyan" | "color14" => Some("bright_cyan"),
        "bright_white" | "brightwhite" | "color15" => Some("bright_white"),
        _ => None,
    }
}

pub fn apply_color_entry(
    colors: &mut CustomColors,
    key: &str,
    value: &str,
) -> Result<(), ColorEntryError> {
    let slot = color_slot(key).ok_or(ColorEntryError::UnknownKey)?;
    let color = Rgb8::from_hex(value).ok_or(ColorEntryError::InvalidValue)?;

    match slot {
        ColorSlot::Foreground => colors.foreground = Some(color),
        ColorSlot::Background => colors.background = Some(color),
        ColorSlot::Cursor => colors.cursor = Some(color),
        ColorSlot::Ansi(index) => colors.ansi[index] = Some(color),
    }

    Ok(())
}

fn color_slot(key: &str) -> Option<ColorSlot> {
    match canonical_color_key(key)? {
        "foreground" => Some(ColorSlot::Foreground),
        "background" => Some(ColorSlot::Background),
        "cursor" => Some(ColorSlot::Cursor),
        "black" => Some(ColorSlot::Ansi(0)),
        "red" => Some(ColorSlot::Ansi(1)),
        "green" => Some(ColorSlot::Ansi(2)),
        "yellow" => Some(ColorSlot::Ansi(3)),
        "blue" => Some(ColorSlot::Ansi(4)),
        "magenta" => Some(ColorSlot::Ansi(5)),
        "cyan" => Some(ColorSlot::Ansi(6)),
        "white" => Some(ColorSlot::Ansi(7)),
        "bright_black" => Some(ColorSlot::Ansi(8)),
        "bright_red" => Some(ColorSlot::Ansi(9)),
        "bright_green" => Some(ColorSlot::Ansi(10)),
        "bright_yellow" => Some(ColorSlot::Ansi(11)),
        "bright_blue" => Some(ColorSlot::Ansi(12)),
        "bright_magenta" => Some(ColorSlot::Ansi(13)),
        "bright_cyan" => Some(ColorSlot::Ansi(14)),
        "bright_white" => Some(ColorSlot::Ansi(15)),
        _ => None,
    }
}
