
pub fn strip_newline(input: &String) -> String
{
    input.strip_suffix("\r\n")
         .or(input.strip_suffix("\n"))
         .unwrap_or(&input)
         .to_string()
}

fn remove_whitespace(input: &str) -> String {
    input.chars().filter(|&c| !c.is_whitespace()).collect()
}