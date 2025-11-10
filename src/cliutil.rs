//! A set of utility functions to implement reglang-expr CLIs

use std::{
    error::Error,
    fs::read_to_string,
    io::{Read, stdin},
};

/// Unzip a vector of key-value pairs into separate vectors for keys and values.
///
/// ```
/// use reqlang_expr::cliutil::unzip_key_values;
///
/// let keys_values: Vec<(String, String)> = vec![
///     ("key1".to_string(), "val1".to_string()),
///     ("key2".to_string(), "val2".to_string())
/// ];
///
/// let (keys, values) = unzip_key_values(keys_values);
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

/// Parse a single key-value pair string. This is used to parse command line arguments.
///
/// ```
/// use reqlang_expr::cliutil::parse_key_val;
///
/// let a = parse_key_val::<String, String>("a=1");
///
/// assert_eq!(("a".to_string(), "1".to_string()), a.unwrap());
/// ```
///
/// Example of using parse_key_val with Clap
///
/// ```
/// use clap::Parser;
/// use reqlang_expr::cliutil::parse_key_val;
///
/// #[derive(Parser, Debug)]
/// struct Args {
///     #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser = parse_key_val::<String, String>)]
///     vars: Vec<(String, String)>
/// }
///
/// let args = Args::parse_from(["test", "--vars", "key=value", "another_key=another_value"]);
/// assert_eq!(args.vars, vec![("key".to_string(), "value".to_string()), ("another_key".to_string(), "another_value".to_string())]);
/// ```
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

/// Read source code from a file at the provided path or from standard input if no path is provided.
///
/// # Usage
///
/// ## From a file:
///
/// ```
/// use reqlang_expr::cliutil::read_in_source;
///
/// let source_code = read_in_source(Some("./spec/valid/call_id.expr".to_string()));
///
/// assert_eq!("(id (noop))", source_code);
/// ```
///
/// ## From stdin:
///
/// ```ignore
/// use reqlang_expr::cliutil::read_in_source;
///
/// // Assuming "(id (noop))" was passed to stdin...
///
/// let source_code = read_in_source(None);
///
/// assert_eq!("(id (noop))".to_string(), source_code);
/// ```
pub fn read_in_source(path: Option<String>) -> String {
    match path {
        Some(path) => {
            //
            read_to_string(path).expect("should be able to open file at path")
        }
        None => {
            let mut source = String::new();

            stdin().read_to_string(&mut source).unwrap();

            source
        }
    }
}

#[cfg(test)]
mod cliutil_tests {
    use clap::Parser;

    use crate::cliutil::{parse_key_val, read_in_source};

    #[test]
    fn read_in_source_from_file() {
        let result = read_in_source(Some("./spec/valid/call_id.expr".to_string()));

        assert_eq!("(id (noop))", result);
    }

    #[test]
    fn parse_key_val_valid_keyvalue_pair() {
        #[derive(Parser, Debug, PartialEq)]
        struct Args {
            #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser = parse_key_val::<String, String>)]
            vars: Vec<(String, String)>,
        }

        assert_eq!(
            Args {
                vars: vec![(String::from("key"), String::from("value"))]
            },
            Args::try_parse_from(["test", "--vars", "key=value"])
                .ok()
                .unwrap()
        );
    }

    #[test]
    fn parse_key_val_invalid_keyvalue_pair() {
        #[derive(Parser, Debug, PartialEq)]
        struct Args {
            #[arg(long, value_delimiter = ' ', num_args = 1.., value_parser = parse_key_val::<String, String>)]
            vars: Vec<(String, String)>,
        }

        assert_eq!(
            "error: invalid value 'key_without_value' for '--vars <VARS>...': should be formatted as key=value pair: `key_without_value`\n\nFor more information, try '--help'.\n",
            Args::try_parse_from(["test", "--vars", "key_without_value"])
                .err()
                .unwrap()
                .to_string()
        );
    }
}
