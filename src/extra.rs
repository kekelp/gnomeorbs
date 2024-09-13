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
