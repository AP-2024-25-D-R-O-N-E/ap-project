use std::str::FromStr;

pub fn parse_string<T>(string: String) -> Result<Vec<T>, String>
where
    T: FromStr,
{
    for c in string.chars() {
        if !(c.is_digit(10) || c == ',' || c.is_whitespace()) {
            return Err("The string contains extraneous characters".to_string());
        }
    }
    let rv: Vec<T> = string
        .split(',')
        .filter_map(|s| s.trim().parse::<T>().ok())
        .collect();

    return Ok(rv);
}
