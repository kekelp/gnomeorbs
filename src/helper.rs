
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
