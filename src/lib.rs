use std::collections::HashMap;

fn token_to_kv(token: &str) -> (&str, Box<dyn Args>) {
    match token.len() {
        1 => (token, Box::new(BoolArg(None))),
        2 => {
            let arg_name = &token[..=0];
            match &token[1..=1] {
                "*" => (arg_name, Box::new(StringArg(None))),
                "#" => (arg_name, Box::new(NumberArg(None))),
                t => panic!(format!("unknow arg type! - {}", t)),
            }
        }
        _ => panic!("invalid schema"),
    }
}

pub fn parse<'a>(schema: &'a str, input: &'a str) -> HashMap<&'a str, Box<dyn Args>> {
    let mut args: HashMap<&str, Box<dyn Args>> =
        schema.split(",").map(str::trim).map(token_to_kv).collect();

    let vec: Vec<&str> = input.split(" ").collect();
    for (index, token) in vec.iter().enumerate() {
        if token.starts_with("-") {
            let arg_name = &token[1..=1];
            args.get_mut(arg_name)
                .unwrap()
                .set(if index < vec.len() - 1 {
                    vec[index + 1]
                } else {
                    ""
                });
        }
    }
    return args;
}

pub trait Args {
    fn set(&mut self, val: &str);
    fn get(&self) -> String;
    fn as_number(&self) -> isize {
        self.get().parse().unwrap()
    }
    fn as_bool(&self) -> bool {
        self.get().parse().unwrap()
    }
}

struct StringArg(Option<String>);
struct BoolArg(Option<bool>);
struct NumberArg(Option<isize>);

impl<'a> Args for StringArg {
    fn set(&mut self, val: &str) {
        self.0.replace(val.to_string());
    }

    fn get(&self) -> String {
        self.0.as_ref().unwrap().to_string()
    }
}
impl Args for BoolArg {
    fn set(&mut self, _val: &str) {
        self.0.replace(true);
    }

    fn get(&self) -> String {
        match self.0 {
            Some(_) => "true".to_string(),
            None => "false".to_string(),
        }
    }
}
impl Args for NumberArg {
    fn set(&mut self, val: &str) {
        let val = val.parse().unwrap();
        self.0.replace(val);
    }

    fn get(&self) -> String {
        self.0.unwrap().to_string()
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[test]
    #[should_panic]
    fn no_args() {
        let args = parse("", "");
        assert!(args.get("d").is_none());
    }

    #[test]
    fn parses_single_arg() {
        let args = parse("d*", "-d /var/logs");
        assert_eq!(args.get("d").unwrap().get(), "/var/logs");
    }

    #[test]
    fn parse_single_arg_2() {
        let args = parse("n*", "-n foo");
        assert_eq!(args.get("n").unwrap().get(), "foo");
    }

    #[test]
    fn parses_multiple_args() {
        let args = parse("d*,n*", "-d /var/logs -n foo");
        assert_eq!(args.get("d").unwrap().get(), "/var/logs");
        assert_eq!(args.get("n").unwrap().get(), "foo");
    }

    #[test]
    fn parse_bool_arg_true() {
        let args = parse("l", "-l");
        assert_eq!(args.get("l").unwrap().as_bool(), true);
    }

    #[test]
    fn parse_bool_arg_false() {
        let args = parse("l", "");
        assert_eq!(args.get("l").unwrap().as_bool(), false);
    }

    #[test]
    fn parse_number_arg() {
        let args = parse("p#", "-p 8080");
        assert_eq!(args.get("p").unwrap().as_number(), 8080);
    }
}
