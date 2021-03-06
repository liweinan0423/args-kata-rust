#![allow(unused_imports)]
use core::fmt::Debug;
use std::{collections::HashMap, marker::PhantomData, str::FromStr};

fn token_to_kv(token: &str) -> Result<(&str, Box<dyn Args>), ParseErr> {
    match token.len() {
        0 => Err(ParseErr::InvalidSchema),
        1 => Ok((token, Box::new(BoolArg(false)))),
        _ => {
            let arg_name = &token[..=0];
            match &token[1..] {
                "*" => Ok((arg_name, Box::new(StringArg(None)))),
                "#" => Ok((arg_name, Box::new(NumberArg(None)))),
                "[*]" => Ok((arg_name, Box::new(StrArrayArg(vec![])))),
                "[#]" => Ok((arg_name, Box::new(NumberArrayArg(vec![])))),
                t => Err(ParseErr::UnsupportedArgType(t.to_string())),
            }
        }
    }
}

pub fn parse<'a>(
    schema: &'a str,
    input: &'a str,
) -> Result<HashMap<&'a str, Box<dyn Args>>, ParseErr> {
    let args: Result<HashMap<&str, Box<dyn Args>>, ParseErr> =
        schema.split(',').map(str::trim).map(token_to_kv).collect();
    args.and_then(|mut args| {
        for token in TokensIterator::from(input.to_string()) {
            if let Some(arg) = args.get_mut(&token.modifier[..]) {
                let result = arg.set(token.values);
                if result.is_err() {
                    return Err(result.unwrap_err());
                }
                
            } else {
                return Err(ParseErr::UnknownArg(token.modifier));
            }
        }
        Ok(args)
    })
}

struct TokensIterator {
    input: String,
    cursor: usize,
}

impl TokensIterator {
    fn from(input: String) -> Self {
        Self {
            input,
            cursor: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Token {
    modifier: String,
    values: Vec<String>,
}


impl Iterator for TokensIterator {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        for segment  in self.input.split('-').skip(self.cursor) {
            self.cursor += 1; //advance the cursor
            if segment.len() > 0 {
                let modifier = segment.split(' ').nth(0).expect("").to_string();
                let values: Vec<String> = segment.split(' ').skip(1).filter(|i| i.len() > 0).map(ToString::to_string).collect();
                return Some(Token {modifier, values});
            }
        }
        None
    }
}

pub trait Args {
    fn set(&mut self, tokens: Vec<String>) -> Result<(), ParseErr>;
    fn get(&self) -> Option<String>;
    fn as_number(&self) -> Option<isize> {
        self.get().and_then(|v| v.parse().ok())
    }
    fn as_bool(&self) -> Option<bool> {
        self.get().and_then(|v| v.parse().ok())
    }
    fn as_str_array(&self) -> Vec<String> {
        self.get().map(|v| v.split(',').map(ToString::to_string).collect()).unwrap_or(vec![])
    }
    fn as_num_array(&self) -> Vec<isize> {
        self.get().map(|v| v.split(',').filter_map(|v|v.parse().ok()).collect()).unwrap_or(vec![])
    }
}

#[derive(Debug)]
struct StringArg(Option<String>);
#[derive(Debug)]
struct BoolArg(bool);
#[derive(Debug)]
struct NumberArg(Option<isize>);
#[derive(Debug)]
struct StrArrayArg(Vec<String>);
#[derive(Debug)]
struct NumberArrayArg(Vec<isize>);

impl Args for NumberArrayArg {
    fn set(&mut self, tokens: Vec<String>) -> Result<(), ParseErr> {
        self.0.append(
            &mut tokens.into_iter()
                .filter_map(|t| t.parse().ok())
                .collect()
        );
        Ok(())
    }

    fn get(&self) -> Option<String> {
        Some(self.0.iter().map(ToString::to_string).collect::<Vec<String>>().join(","))
    }
}
impl Args for StrArrayArg {
    fn set(&mut self, mut tokens: Vec<String>) -> Result<(), ParseErr> {
        self.0.append(&mut tokens);
        Ok(())
    }

    fn get(&self) -> Option<String> {
        Some(self.0.join(","))
    }
}

impl Args for StringArg {
    fn set(&mut self, val: Vec<String>) -> Result<(), ParseErr> {
        self.0.replace(val.join(""));
        Ok(())
    }

    fn get(&self) -> Option<String> {
        self.0.to_owned()
    }
}
impl Args for BoolArg {
    fn set(&mut self, values: Vec<String>) -> Result<(), ParseErr> {
        if values.len() == 0 || values.join("").to_lowercase() == "true" {
            self.0 = true;
        } else {
            self.0 = false;
        }
        Ok(())
    }

    fn get(&self) -> Option<String> {
        Some(self.0.to_string())
    }
}
impl Args for NumberArg {
    fn set(&mut self, val: Vec<String>) -> Result<(), ParseErr> {
        match val.join("").parse() {
            Ok(val) => {
                self.0.replace(val);
                Ok(())
            }
            Err(_) => Err(ParseErr::NumberFormatErr(val.join(""))),
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
    mod token_iterator {
        use super::*;
        #[test]
        fn test_token_iter() {
            let tokens = TokensIterator::from("-d /var/logs -p 8080 -l".to_string());
            let mut iter = tokens.into_iter();
            assert_eq!(iter.next().unwrap(), Token {
                modifier: 'd'.to_string(),
                values: vec!["/var/logs".to_string()],
            });
            assert_eq!(iter.next().unwrap(), Token {
                modifier: 'p'.to_string(),
                values: vec!["8080".to_string()],
            });
            assert_eq!(iter.next().unwrap(), Token {
                modifier: 'l'.to_string(),
                values: vec![],
            });
            assert_eq!(iter.next(), None);
        }   
    }
    mod boolean_args {
        use super::*;
        #[test]
        fn parse_bool_arg_true() {
            let args = parse("l", "-l").unwrap();
            assert_eq!(args.get("l").unwrap().as_bool().unwrap(), true);
        }

        #[test]
        fn parse_explicit_true() {
            let args = parse("l", "-l true").unwrap();
            assert_eq!(args.get("l").unwrap().as_bool().unwrap(), true);
        }

        #[test]
        fn parse_explicit_true_case_insensitive() {
            let args = parse("l", "-l True").unwrap();
            assert_eq!(args.get("l").unwrap().as_bool().unwrap(), true);

            let args = parse("l", "-l TRUE").unwrap();
            assert_eq!(args.get("l").unwrap().as_bool().unwrap(), true);
        }

        #[test]
        fn parse_explicit_false() {
            let args = parse("l", "-l false").unwrap();
            assert_eq!(args.get("l").unwrap().as_bool().unwrap(), false);
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

        #[test]
        fn should_return_number_format_err() {
            let args = parse("p#", "-p foo");
            assert_eq!(args.unwrap_err(), ParseErr::NumberFormatErr("foo".to_string()));
        }
    }

    mod array_args {
        use super::*;

        #[test]
        fn parse_str_arr_arg() {
            let args = parse("s[*]", "-s this is an array");
            assert_eq!(args.unwrap().get("s").unwrap().get().unwrap(), "this,is,an,array");
            let args = parse("s[*]", "-s this is an array");
            assert_eq!(args.unwrap().get("s").unwrap().as_str_array(), vec!["this","is","an","array"]);
        }

        #[test]
        fn parse_number_arr_arg() {
            let args = parse("p[#]", "-p 1 2 3 4 5");
            assert_eq!(args.unwrap().get("p").unwrap().as_num_array(), vec![1,2,3,4,5]);
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ParseErr {
    InvalidSchema,
    UnsupportedArgType(String),
    UnknownArg(String),
    NumberFormatErr(String)
}
