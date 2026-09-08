use miniserde::json;
use std::fmt::Write;

const MC_COLORS: [&str; 16] = [
    "black",
    "dark_blue",
    "dark_green",
    "dark_aqua",
    "dark_red",
    "dark_purple",
    "gold",
    "gray",
    "dark_gray",
    "blue",
    "green",
    "aqua",
    "red",
    "light_purple",
    "yellow",
    "white",
];

/**
 * Converts legacy ampersand codes to section signs; `\\&` remains a literal ampersand.
 */
pub fn string(text: &str) -> String {
    text.replace("\\&", "\0")
        .replace('&', "§")
        .replace('\0', "&")
}

/**
 * Produces a modern Minecraft JSON component so styles work consistently in MOTDs and disconnects.
 */
pub fn component(text: &str) -> String {
    let mut output = String::from(r#"{"text":"","extra":["#);
    let mut style = String::new();
    let mut part = String::new();
    let formatted = string(text);
    let mut chars = formatted.chars();

    while let Some(character) = chars.next() {
        if character == '§'
            && let Some(code) = chars.next().filter(|code| is_format(*code))
        {
            push(&mut output, &part, &style);
            part.clear();
            apply(&mut style, code);
            continue;
        }
        part.push(character);
    }
    push(&mut output, &part, &style);
    output.push_str("]}");
    output
}

fn is_format(code: char) -> bool {
    code.is_ascii_hexdigit() || matches!(code.to_ascii_lowercase(), 'k'..='o' | 'r')
}

fn apply(style: &mut String, code: char) {
    if let Some(color) = code.to_digit(16) {
        style.clear();
        write!(style, ",\"color\":\"{}\"", MC_COLORS[color as usize]).unwrap();
        return;
    }
    match code.to_ascii_lowercase() {
        'k' => style.push_str(",\"obfuscated\":true"),
        'l' => style.push_str(",\"bold\":true"),
        'm' => style.push_str(",\"strikethrough\":true"),
        'n' => style.push_str(",\"underlined\":true"),
        'o' => style.push_str(",\"italic\":true"),
        'r' => style.clear(),
        _ => {}
    }
}

fn push(output: &mut String, text: &str, style: &str) {
    if text.is_empty() {
        return;
    }
    if !output.ends_with('[') {
        output.push(',');
    }
    write!(output, r#"{{"text":{}{style}}}"#, json::to_string(text)).unwrap();
}
