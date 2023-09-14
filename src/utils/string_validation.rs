const DECIMAL_SEPARATOR: char = '.';

pub fn is_alphanumeric(string: &str) -> bool {
    // chars() also handles multibyte correctly.
    string.chars().all(|char| char.is_alphanumeric())
}

pub fn is_alphanumeric_with(string: &str, allowed_chars: &str) -> bool {
    string.chars().all(|char| char.is_alphanumeric() || allowed_chars.contains(char))
}

pub fn begins_with_dash(string: &str) -> bool {
    string.chars().next().unwrap_or(' ') == '-'
}

pub fn is_numeric(string: &str) -> bool {
    string.parse::<f64>().is_ok()
}

// Numeric value with unit (e.g. 100M, 1.5GB, 5 m). May contain a space between number and unit.
pub fn is_numeric_with_unit(string: &str, valid_units: &[String]) -> bool {
    let value_chars = string.chars().take_while(|char| char.is_numeric() || *char == DECIMAL_SEPARATOR).collect::<String>();
    if !is_numeric(&value_chars) {
        return false;
    }

    let unit_chars = string.chars().skip_while(|char| char.is_numeric() || *char == DECIMAL_SEPARATOR).collect::<String>();
    let unit_string = unit_chars.trim().to_string();
    !unit_string.is_empty() && valid_units.contains(&unit_string)
}
