
pub fn is_alphanumeric(string: &String) -> bool {
    // chars() also handles multibyte correctly.
    string.chars().all(|char| char.is_alphanumeric())
}

pub fn is_alphanumeric_with(string: &String, allowed_chars: &str) -> bool {
    string.chars().all(|char| char.is_alphanumeric() || allowed_chars.contains(char))
}

pub fn begins_with_dash(string: &String) -> bool {
    string.chars().next().unwrap_or(' ') == '-'
}

pub fn has_whitespace(string: &String) -> bool {
    string.chars().any(|char| char.is_whitespace())
}