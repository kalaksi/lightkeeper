
pub fn strip_newline<Stringable: ToString>(input: &Stringable) -> String
{
    let input = input.to_string();
    input.strip_suffix("\r\n")
         .or(input.strip_suffix("\n"))
         .unwrap_or(&input)
         .to_string()
}

pub fn remove_whitespace<Stringable: ToString>(input: &Stringable) -> String {
    input.to_string().chars().filter(|&c| !c.is_whitespace()).collect()
}