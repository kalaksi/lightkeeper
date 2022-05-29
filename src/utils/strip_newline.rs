
pub fn strip_newline(input: &String) -> String
{
    input.strip_suffix("\r\n")
         .or(input.strip_suffix("\n"))
         .unwrap_or(&input)
         .to_string()
}