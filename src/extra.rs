pub trait DoublePushLine {
    fn pushln2(&mut self, string1: &str, string2: &str);
    fn pushln(&mut self, string: &str);
}
impl DoublePushLine for String {
    fn pushln2(&mut self, string1: &str, string2: &str) {
        self.push_str(string1);
        self.push_str(string2);
        self.push('\n');
    }

    fn pushln(&mut self, string: &str) {
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

pub fn misspellings(text: &str) -> Vec<String> {
    let mut results = Vec::<String>::new();

    // inverted letters
    let n = std::cmp::min(text.len() - 1, 4);
    for i in 0..n {
        results.push(invert_letters(text, i, i + 1));
    }

    // missing letters
    if n >= 3 {
        results.push(text[0..2].to_string() + &text[3..]);
    }
    if n >= 2 {
        results.push(text[0..1].to_string() + &text[2..]);
    }
    results.push(text[1..].to_string());

    return results;
}

pub fn invert_letters(text: &str, i1: usize, i2: usize) -> String {
    let characs: Vec<char> = text.chars().collect();

    let mut char_vec_1 = Vec::<char>::new();

    for (i, c) in characs.iter().enumerate() {
        let nc = match i {
            i if i == i1 => characs[i2],
            i if i == i2 => characs[i1],
            _ => *c,
        };
        char_vec_1.push(nc);
    }
    let result: String = char_vec_1.into_iter().collect();
    return result;
}
