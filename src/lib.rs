use std::{string::String, path::PathBuf, str::FromStr};

pub trait AddManyThings {
    fn manypush(&mut self, parts: &[&str]);
}

impl AddManyThings for String {
    fn manypush(&mut self, parts: &[&str]) {
        for &part in parts {
            self.push_str(part);
        }
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