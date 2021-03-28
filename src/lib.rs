#![allow(unused_imports)]
use core::fmt::Debug;
use std::collections::HashMap;

fn token_to_kv(token: &str) -> Result<(&str, Box<dyn Args>), ParseErr> {
    match token.len() {
        1 => Ok((token, Box::new(BoolArg(None)))),
        2 => {
            let arg_name = &token[..=0];
            match &token[1..=1] {
                "*" => Ok((arg_name, Box::new(StringArg(None)))),
                "#" => Ok((arg_name, Box::new(NumberArg(None)))),
                t => Err(ParseErr::UnsupportedArgType(t.to_string())),
            }
        }
        _ => Err(ParseErr::InvalidSchema),
    }
}

pub fn parse<'a>(
    schema: &'a str,
    input: &'a str,
) -> Result<HashMap<&'a str, Box<dyn Args>>, ParseErr> {
    let args: Result<HashMap<&str, Box<dyn Args>>, ParseErr> =
        schema.split(',').map(str::trim).map(token_to_kv).collect();
    args.and_then(|mut args| {
        let vec: Vec<&str> = input.split(' ').collect();
        for (index, token) in vec.iter().enumerate() {
            if token.starts_with('-') {
                let arg_name = &token[1..=1];
                if let Some(arg) = args.get_mut(arg_name) {
                    arg.set(&vec[index..]);
                } else {
                    return Err(ParseErr::UnknownArg(arg_name.to_string()));
                }
            }
        }
        Ok(args)
    })
}

pub trait Args {
    fn set(&mut self, tokens: &[&str]);
    fn get(&self) -> Option<String>;
    fn as_number(&self) -> Option<isize> {
        self.get().and_then(|v| v.parse().ok())
    }
    fn as_bool(&self) -> Option<bool> {
        self.get().and_then(|v| v.parse().ok())
    }
}
#[derive(Debug)]
struct StringArg(Option<String>);
#[derive(Debug)]
struct BoolArg(Option<bool>);
#[derive(Debug)]
struct NumberArg(Option<isize>);

impl Args for StringArg {
    fn set(&mut self, val: &[&str]) {
        self.0.replace(val[1].to_string());
    }

    fn get(&self) -> Option<String> {
        self.0.to_owned()
    }
}
impl Args for BoolArg {
    fn set(&mut self, _: &[&str]) {
        self.0.replace(true);
    }

    fn get(&self) -> Option<String> {
        match self.0 {
            Some(_) => Some("true".to_string()),
            None => Some("false".to_string()),
        }
    }
}
impl Args for NumberArg {
    fn set(&mut self, val: &[&str]) {
        if let Ok(val) = val[1].parse() {
            self.0.replace(val);
        }
    }

    fn get(&self) -> Option<String> {
        self.0.map(|v| v.to_string())
    }
}

impl Debug for dyn Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod boolean_args {
        use super::*;
        #[test]
        fn parse_bool_arg_true() {
            let args = parse("l", "-l").unwrap();
            assert_eq!(args.get("l").unwrap().as_bool().unwrap(), true);
        }

        #[test]
        fn parse_bool_arg_false() {
            let args = parse("l", "").unwrap();
            assert_eq!(args.get("l").unwrap().as_bool().unwrap(), false);
        }
    }
    mod no_args {
        use super::*;
        #[test]
        #[should_panic]
        fn no_args() {
            let args = parse("", "").unwrap();
            assert!(args.get("d").is_none());
        }
    }
    mod str_args {
        use super::*;
        #[test]
        fn parses_single_arg() {
            let args = parse("d*", "-d /var/logs").unwrap();
            assert_eq!(args.get("d").unwrap().get().unwrap(), "/var/logs");
        }

        #[test]
        fn parse_single_arg_2() {
            let args = parse("n*", "-n foo").unwrap();
            assert_eq!(args.get("n").unwrap().get().unwrap(), "foo");
        }

        #[test]
        fn parses_multiple_args() {
            let args = parse("d*,n*", "-d /var/logs -n foo").unwrap();
            assert_eq!(args.get("d").unwrap().get().unwrap(), "/var/logs");
            assert_eq!(args.get("n").unwrap().get().unwrap(), "foo");
        }
    }
    mod number_args {
        use super::*;
        #[test]
        fn parse_number_arg() {
            let args = parse("p#", "-p 8080").unwrap();
            assert_eq!(args.get("p").unwrap().as_number().unwrap(), 8080);
        }
    }

    mod error_cases {
        use super::*;

        #[test]
        fn should_return_err_if_no_schema() {
            let args = parse("", "");
            assert_eq!(args.unwrap_err(), ParseErr::InvalidSchema);
        }

        #[test]
        fn should_return_invalid_arg_type_err() {
            let args = parse("p!", "-p 8080");
            assert_eq!(
                args.unwrap_err(),
                ParseErr::UnsupportedArgType("!".to_string())
            );
        }

        #[test]
        fn should_return_unknown_arg_err() {
            let args = parse("d*", "-p 8080");
            assert_eq!(args.unwrap_err(), ParseErr::UnknownArg("p".to_string()));
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ParseErr {
    InvalidSchema,
    UnsupportedArgType(String),
    UnknownArg(String),
}
