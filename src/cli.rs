use std::{
    error::Error,
    fs::read_to_string,
    io::{Read, stdin},
};

pub fn split_key_values(input: &Vec<(String, String)>) -> (Vec<String>, Vec<String>) {
    let keys: Vec<String> = input.clone().into_iter().map(|(key, _)| key).collect();
    let values: Vec<String> = input.clone().into_iter().map(|(_, value)| value).collect();

    (keys, values)
}

/// Parse a single key-value pair
pub fn parse_key_val<T, U>(value: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let n = 2;

    let parts: Vec<&str> = value.splitn(n, '=').collect();

    if parts.len() != n {
        return Err(format!("should be formatted as key=value pair: `{value}`").into());
    }

    let key = parts[0].parse()?;
    let value = parts[1].parse()?;

    Ok((key, value))
}

pub fn read_in_source(path: Option<String>) -> String {
    match path {
        Some(path) => {
            //
            return read_to_string(path).expect("should be able to open file at path");
        }
        None => {
            let mut source = String::new();

            stdin().read_to_string(&mut source).unwrap();

            source
        }
    }
}
