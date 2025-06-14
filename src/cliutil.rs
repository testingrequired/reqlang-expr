//! A set of utility functions to implement reglang-expr CLIs

use std::{
    error::Error,
    fs::read_to_string,
    io::{Read, stdin},
};

/// Unzip a vector of key-value pairs into separate vectors for keys and values.
///
/// ```
/// let keys_values: Vec<(String, String)> = vec![
///     ("key1".to_string(), "val1".to_string()),
///     ("key2".to_string(), "val2".to_string())
/// ];
///
/// let (keys, values): (Vec<String>, Vec<String>) = keys_values.into_iter().unzip();
///
/// assert_eq!(vec!["key1", "key2"], keys);
/// assert_eq!(vec!["val1", "val2"], values);
/// ```
///
/// ## Usage
///
/// This is can be used to accept in key-value pairs from command line arguments,
/// feeding the keys/values to the compile time and runtime envrionments
/// respectively.
///
/// ```ignore
/// let (var_keys, var_values) = unzip_key_values(args.vars);
/// let (prompt_keys, prompt_values) = unzip_key_values(args.prompts);
/// let (secret_keys, secret_values) = unzip_key_values(args.secrets);
///
/// let env = Env::new(var_keys, prompt_keys, secret_keys);
///
/// let runtime_env: RuntimeEnv = RuntimeEnv {
///     vars: var_values,
///     prompts: prompt_values,
///     secrets: secret_values,
/// };
/// ```
pub fn unzip_key_values(keys_values: Vec<(String, String)>) -> (Vec<String>, Vec<String>) {
    let (keys, values): (Vec<String>, Vec<String>) = keys_values.into_iter().unzip();

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
