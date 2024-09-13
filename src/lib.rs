use std::{string::String, path::{PathBuf, self}, error, env, str::FromStr, };
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;


pub fn turn_to_title_case(text: String) -> String {
    let text = text.replace("-", " ");
    let text = text.replace("_", " ");
    let mut characters: Vec<char> = text.chars().collect();

    let mut prev_character = ' ';
    for i in 0..characters.len() {
        if prev_character == ' ' {
            characters[i] = best_effort_uppercase(characters[i]);
        }
        prev_character = characters[i];
    }

    return characters.into_iter().collect();
}

// Does not support the german ß (will turn ß into a single capital S)
pub fn best_effort_uppercase(c: char) -> char {
    let best_effort_uppercase = c.to_uppercase().next();
    match best_effort_uppercase {
        Some(character) => {
            return character;
        }
        None => {
            return c;
        }
    }
}

pub trait AddManyThings {
    fn push_4(&mut self, strings_to_add: [&str; 4]) -> &mut String;
    fn push_3(&mut self, strings_to_add: [&str; 3]) -> &mut String;
    fn push_2(&mut self, strings_to_add: [&str; 2]) -> &mut String;
}

impl AddManyThings for String {
    fn push_4(self: &mut String, strings_to_add: [&str; 4]) -> &mut String {
        for s in strings_to_add {
            self.push_str(s);
        }
        return self;
    }
    fn push_3(self: &mut String, strings_to_add: [&str; 3]) -> &mut String {
        for s in strings_to_add {
            self.push_str(s);
        }
        return self;
    }
    fn push_2(self: &mut String, strings_to_add: [&str; 2]) -> &mut String {
        for s in strings_to_add {
            self.push_str(s);
        }
        return self;
    }
}

pub trait CreateConditional {
    fn create_conditional(&mut self, overwriting: bool) -> &mut Self;
}

impl CreateConditional for std::fs::OpenOptions {
    fn create_conditional(&mut self, overwriting: bool) -> &mut Self {
        if overwriting == false {
            return self.create_new(true);
        } else {
            return self
            .create (true)
            .truncate(true);
        }
    }
    
}

pub fn is_path_and_exists(string: &str) -> Option<PathBuf> {
    let path_result = PathBuf::from_str(&string);
    if path_result.is_err() {
        return None;
    }
    let path = path_result.unwrap();
    return match path.exists() {
        true => Some(path),
        false => None,
    };
}