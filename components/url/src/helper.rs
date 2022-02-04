pub const SPECIAL_SCHEMES: [&str; 6] = ["ftp", "file", "http", "https", "ws", "wss"];

pub const SPECIAL_SCHEME_PORTS: [(&str, Option<u16>); 6] = [
    ("ftp", Some(21)),
    ("file", None),
    ("http", Some(80)),
    ("https", Some(443)),
    ("ws", Some(80)),
    ("wss", Some(443)),
];

pub fn is_normalized_window_drive_letter(input: &str) -> bool {
    let mut chars = input.chars();
    match (chars.next(), chars.next()) {
        (Some(c), Some(':')) if c.is_ascii_alphabetic() => true,
        _ => false
    }
}

pub fn is_window_drive_letter(input: &str) -> bool {
    let mut chars = input.chars();
    match (chars.next(), chars.next()) {
        (Some(c), Some(':')) if c.is_ascii_alphabetic() => true,
        (Some(c), Some('|')) if c.is_ascii_alphabetic() => true,
        _ => false
    }
}

pub fn is_double_dot_path_segment(input: &str) -> bool {
    input == ".." || match input.to_ascii_lowercase().as_str() {
        ".%2e" | "%2e." | "%2e%2e" => true,
        _ => false
    }
}

pub fn is_single_dot_path_segment(input: &str) -> bool {
    input == "." || input.to_ascii_lowercase() == "%2e"
}

// TODO: Implement the rest of this
pub fn is_url_c(c: char) -> bool {
    match c {
        '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' |
        ',' | '-' | '.' | '/' | ':' | ';' | '=' | '?' | '@' | '_' | '~' => true,
        c => c.is_ascii_alphanumeric()
    }
}

pub fn is_c0_control (n: u32) -> bool {
    match n {
        0x0000..=0x001F => true,
        _ => false
    }
}

pub fn is_c0_control_or_space(n: u32) -> bool {
    return is_c0_control(n) || n == 0x0020;
}

pub fn is_start_with_two_hex(input: &str) -> bool {
    let mut chars = input.chars();
    match (chars.next(), chars.next()) {
        (Some(a), Some(b)) => a.is_ascii_hexdigit() && b.is_ascii_hexdigit(),
        _ => false
    }
}

pub fn contains_forbidden_host_code_point(input: &str) -> bool {
    input.contains(|c| ['\0', '\t', '\r', '\n', ' ', '#', '/', ':', '<', '>', '?', '@', '[', '\\', ']', '^', '|'].contains(&c))
}
