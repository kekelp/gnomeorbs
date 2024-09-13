pub trait AddManyThings {
    fn push_line(&mut self, parts: &str);
}

impl AddManyThings for String {
    fn push_line(&mut self, string: &str) {
        self.push_str(string);
        self.push('\n');
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
            return self.create(true).truncate(true);
        }
    }
}
