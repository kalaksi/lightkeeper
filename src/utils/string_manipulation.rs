pub fn strip_newline<Stringable: ToString>(input: &Stringable) -> String {
    let input = input.to_string();
    input.strip_suffix("\r\n").or(input.strip_suffix('\n')).unwrap_or(&input).to_string()
}

pub fn remove_whitespace<Stringable: ToString>(input: &Stringable) -> String {
    input.to_string().chars().filter(|&c| !c.is_whitespace()).collect()
}

pub fn get_string_between<Stringable: ToString>(input: &Stringable, start: &str, end: &str) -> String {
    let input = input.to_string();
    let start = input.find(start).unwrap() + start.len();
    let end = input.find(end).unwrap();
    input[start..end].to_string()
}

pub fn remove_quotes<Stringable: ToString>(input: &Stringable) -> String {
    let input = input.to_string();
    input
        .strip_prefix('"')
        .or(input.strip_prefix('\''))
        .and_then(|input| input.strip_suffix('"'))
        .or(input.strip_suffix('\''))
        .unwrap_or(&input)
        .to_string()
}
