use std::{fs, path::Path};

use termy_config_core::{Rgb8, canonical_color_key, parse_theme_id};

use super::ConfigIoError;
use super::io::{ensure_config_file, notify_config_changed, write_atomic};

fn update_config_contents<R>(
    updater: impl FnOnce(&str) -> Result<(String, R), String>,
) -> Result<R, String> {
    let config_path = ensure_config_file().map_err(|error| error.to_string())?;
    let existing = fs::read_to_string(&config_path)
        .map_err(|source| ConfigIoError::ReadConfig {
            path: config_path.clone(),
            source,
        })
        .map_err(|error| error.to_string())?;
    let (updated, result) = updater(&existing)?;
    write_atomic(&config_path, &updated).map_err(|error| error.to_string())?;
    notify_config_changed();
    Ok(result)
}

fn upsert_root_assignment(contents: &str, key: &str, value: &str) -> String {
    let mut new_config = String::new();
    let mut replaced = false;
    let mut inserted_before_first_section = false;
    let mut in_root_section = true;

    for line in contents.lines() {
        let trimmed = line.trim();
        let is_section_header = trimmed.starts_with('[') && trimmed.ends_with(']');

        if is_section_header {
            if !replaced && !inserted_before_first_section {
                new_config.push_str(&format!("{} = {}\n", key, value));
                inserted_before_first_section = true;
                replaced = true;
            }
            in_root_section = false;
            new_config.push_str(line);
            new_config.push('\n');
            continue;
        }

        if in_root_section {
            let mut parts = trimmed.splitn(2, '=');
            let line_key = parts.next().unwrap_or("").trim();
            if line_key.eq_ignore_ascii_case(key) {
                if !replaced {
                    new_config.push_str(&format!("{} = {}\n", key, value));
                    replaced = true;
                }
                continue;
            }
        }

        new_config.push_str(line);
        new_config.push('\n');
    }

    if !replaced && !inserted_before_first_section {
        if !new_config.is_empty() && !new_config.ends_with('\n') {
            new_config.push('\n');
        }
        new_config.push_str(&format!("{} = {}\n", key, value));
    }

    new_config
}

fn append_section(out: &mut String, section_name: &str, section_lines: &[String]) {
    out.push_str(&format!("[{}]\n", section_name));
    for section_line in section_lines {
        out.push_str(section_line);
        out.push('\n');
    }
}

fn replace_or_insert_section(
    contents: &str,
    section_name: &str,
    section_lines: &[String],
) -> String {
    let mut new_config = String::new();
    let mut in_target_section = false;
    let mut target_section_inserted = false;
    let target_header = format!("[{}]", section_name);

    for line in contents.lines() {
        let trimmed = line.trim();
        let is_section_header = trimmed.starts_with('[') && trimmed.ends_with(']');
        if is_section_header {
            in_target_section = false;
            if trimmed.eq_ignore_ascii_case(&target_header) {
                if !target_section_inserted {
                    append_section(&mut new_config, section_name, section_lines);
                    target_section_inserted = true;
                }
                in_target_section = true;
                continue;
            }
        }

        if in_target_section {
            continue;
        }

        new_config.push_str(line);
        new_config.push('\n');
    }

    if !target_section_inserted {
        if !new_config.is_empty() {
            new_config.push('\n');
        }
        append_section(&mut new_config, section_name, section_lines);
    }

    new_config
}

pub fn import_colors_from_json(json_path: &Path) -> Result<String, String> {
    let contents =
        fs::read_to_string(json_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&contents).map_err(|e| format!("Invalid JSON: {}", e))?;

    let colors = json
        .as_object()
        .ok_or_else(|| "JSON must be an object".to_string())?;

    let mut color_lines = Vec::new();

    for (key, value) in colors {
        if key.starts_with('$') {
            continue;
        }

        let hex = value
            .as_str()
            .ok_or_else(|| format!("Color '{}' must be a hex string", key))?;

        if Rgb8::from_hex(hex).is_none() {
            return Err(format!("Invalid hex color for '{}': {}", key, hex));
        }

        let Some(config_key) = canonical_color_key(key) else {
            continue;
        };

        color_lines.push(format!("{} = {}", config_key, hex));
    }

    if color_lines.is_empty() {
        return Err("No valid colors found in JSON".to_string());
    }

    let color_count = color_lines.len();
    update_config_contents(|existing| {
        Ok((
            replace_or_insert_section(existing, "colors", &color_lines),
            (),
        ))
    })?;
    Ok(format!("Imported {} colors", color_count))
}

pub fn set_theme_in_config(theme_id: &str) -> Result<String, String> {
    let theme = parse_theme_id(theme_id).ok_or_else(|| "Invalid theme id".to_string())?;
    update_config_contents(|existing| {
        Ok((
            upsert_root_assignment(existing, "theme", &theme),
            format!("Theme set to {}", theme),
        ))
    })
}

pub fn set_config_value(key: &str, value: &str) -> Result<(), String> {
    update_config_contents(|existing| Ok((upsert_root_assignment(existing, key, value), ())))
}

#[cfg(test)]
mod tests {
    use super::{replace_or_insert_section, upsert_root_assignment};

    #[test]
    fn upsert_root_assignment_replaces_existing_root_value() {
        let input = "theme = termy\nfont_size = 14\n";
        let output = upsert_root_assignment(input, "theme", "nord");
        assert_eq!(output, "theme = nord\nfont_size = 14\n");
    }

    #[test]
    fn upsert_root_assignment_inserts_before_first_section_when_missing() {
        let input = "font_size = 14\n\n[colors]\nforeground = #ffffff\n";
        let output = upsert_root_assignment(input, "theme", "tokyo-night");
        assert_eq!(
            output,
            "font_size = 14\n\ntheme = tokyo-night\n[colors]\nforeground = #ffffff\n"
        );
    }

    #[test]
    fn upsert_root_assignment_handles_missing_trailing_newline() {
        let input = "font_size = 14";
        let output = upsert_root_assignment(input, "theme", "nord");
        assert_eq!(output, "font_size = 14\ntheme = nord\n");
    }

    #[test]
    fn upsert_root_assignment_preserves_leading_comments() {
        let input = "# default config\n# keep this\n\n[colors]\nforeground = #ffffff\n";
        let output = upsert_root_assignment(input, "theme", "tokyo-night");
        assert_eq!(
            output,
            "# default config\n# keep this\n\ntheme = tokyo-night\n[colors]\nforeground = #ffffff\n"
        );
    }

    #[test]
    fn upsert_root_assignment_collapses_duplicate_root_keys() {
        let input = "font_size = 12\nfont_size = 13\n[colors]\nforeground = #ffffff\n";
        let output = upsert_root_assignment(input, "font_size", "14");
        assert_eq!(output, "font_size = 14\n[colors]\nforeground = #ffffff\n");
    }

    #[test]
    fn replace_or_insert_section_replaces_existing_section_body() {
        let input = "theme = termy\n[colors]\nforeground = #ffffff\nbackground = #000000\n";
        let output = replace_or_insert_section(
            input,
            "colors",
            &[
                "foreground = #111111".to_string(),
                "cursor = #222222".to_string(),
            ],
        );

        assert_eq!(
            output,
            "theme = termy\n[colors]\nforeground = #111111\ncursor = #222222\n"
        );
    }

    #[test]
    fn replace_or_insert_section_matches_header_case_insensitively() {
        let input = "theme = termy\n[Colors]\nforeground = #ffffff\n";
        let output =
            replace_or_insert_section(input, "colors", &["foreground = #111111".to_string()]);
        assert_eq!(output, "theme = termy\n[colors]\nforeground = #111111\n");
    }

    #[test]
    fn replace_or_insert_section_collapses_duplicate_sections() {
        let input = "theme = termy\n[colors]\nforeground = #ffffff\n[other]\nvalue = 1\n[colors]\nbackground = #000000\n";
        let output = replace_or_insert_section(
            input,
            "colors",
            &[
                "foreground = #111111".to_string(),
                "background = #222222".to_string(),
            ],
        );
        assert_eq!(
            output,
            "theme = termy\n[colors]\nforeground = #111111\nbackground = #222222\n[other]\nvalue = 1\n"
        );
    }

    #[test]
    fn replace_or_insert_section_appends_missing_section() {
        let input = "theme = termy\nfont_size = 14\n";
        let output =
            replace_or_insert_section(input, "colors", &["foreground = #111111".to_string()]);

        assert_eq!(
            output,
            "theme = termy\nfont_size = 14\n\n[colors]\nforeground = #111111\n"
        );
    }
}
