
pub fn is_alphanumeric(string: &String) -> bool {
    // chars() also handles multibyte correctly.
    string.chars().all(|char| char.is_alphanumeric())
}

pub fn is_alphanumeric_with_dash(string: &String) -> bool {
    string.chars().all(|char| char.is_alphanumeric() || char == '-')
}